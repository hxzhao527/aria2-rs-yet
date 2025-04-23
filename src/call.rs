use serde::ser::{SerializeSeq, Serializer};
use serde_with::{serde_as, DisplayFromStr};

use crate::options::Aria2Options;

pub trait Call {
    type Response: serde::de::DeserializeOwned;

    fn method(&self) -> &'static str;
    fn serialize_params<S: SerializeSeq>(&self, _serializer: &mut S) -> Result<(), S::Error> {
        Ok(())
    }
    fn to_params(self, token: Option<&str>) -> Option<Aria2Params<'_, Self>>
    where
        Self: Sized,
    {
        Some(Aria2Params::new(token, self))
    }
}

macro_rules! option_element {
    ($opt: expr, $serializer: expr) => {
        if let Some(ref value) = $opt {
            $serializer.serialize_element(value)?;
        }
    };
}

/// https://aria2.github.io/manual/en/html/aria2c.html#rpc-authorization-secret-token
#[derive(Debug)]
pub struct Aria2Params<'a, T> {
    token: Option<&'a str>,
    params: T,
}

impl<'a, T> Aria2Params<'a, T> {
    /// token with prefix
    pub fn new(token: Option<&'a str>, params: T) -> Self {
        Self { token, params }
    }
}

impl<T> serde::Serialize for Aria2Params<'_, T>
where
    T: Call,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        option_element!(self.token, seq);
        self.params.serialize_params(&mut seq)?;
        seq.end()
    }
}

#[derive(Debug)]
pub struct SystemListMethods;
impl Call for SystemListMethods {
    type Response = Vec<String>;

    fn method(&self) -> &'static str {
        "system.listMethods"
    }

    fn to_params(self, _: Option<&str>) -> Option<Aria2Params<'_, Self>>
    where
        Self: Sized,
    {
        None
    }
}

#[derive(Debug)]
pub struct AddUri {
    pub uris: Vec<String>,
    pub options: Option<Aria2Options>,
    pub position: Option<i32>,
}

impl AddUri {
    pub fn new<S: Into<String>>(
        uris: Vec<S>,
        options: Option<Aria2Options>,
        position: Option<i32>,
    ) -> Self {
        Self::uris(uris).options(options).position(position)
    }

    pub fn uris<S: Into<String>>(uris: Vec<S>) -> Self {
        Self {
            uris: uris.into_iter().map(|s| s.into()).collect(),
            options: None,
            position: None,
        }
    }

    pub fn options(mut self, options: Option<Aria2Options>) -> Self {
        self.options = options;
        self
    }

    pub fn position(mut self, position: Option<i32>) -> Self {
        self.position = position;
        self
    }
}

impl Call for AddUri {
    type Response = GidReply;

    fn method(&self) -> &'static str {
        "aria2.addUri"
    }

    fn serialize_params<S: SerializeSeq>(&self, serializer: &mut S) -> Result<(), S::Error> {
        serializer.serialize_element(&self.uris)?;
        option_element!(self.options, serializer);
        option_element!(self.position, serializer);
        Ok(())
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct GidReply(pub String);

impl From<GidReply> for String {
    fn from(gid: GidReply) -> Self {
        gid.0
    }
}

#[derive(Debug)]
pub struct GetVersion;
impl Call for GetVersion {
    type Response = VersionReply;

    fn method(&self) -> &'static str {
        "aria2.getVersion"
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct VersionReply {
    pub version: String,
    #[serde(rename = "enabledFeatures")]
    pub enabled_features: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TellStatusField {
    Gid,
    Status,
    TotalLength,
    CompletedLength,
    UploadedLength,
    BitField,
    DownloadSpeed,
    UploadSpeed,
    InfoHash,
    NumSeeders,
    Seeder,
    PieceLength,
    NumPieces,
    Connections,
    ErrorCode,
    ErrorMessage,
    FollowedBy,
    Following,
    BelongsTo,
    Dir,
    Files,
    // bt nested fields not supported now
    // Bittorrent,
    VerifiedLength,
    VeriyIntegrityPending,
}

impl TryFrom<&str> for TellStatusField {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "gid" => Ok(Self::Gid),
            "status" => Ok(Self::Status),
            "totalLength" => Ok(Self::TotalLength),
            "completedLength" => Ok(Self::CompletedLength),
            "uploadedLength" => Ok(Self::UploadedLength),
            "bitField" => Ok(Self::BitField),
            "downloadSpeed" => Ok(Self::DownloadSpeed),
            "uploadSpeed" => Ok(Self::UploadSpeed),
            "infoHash" => Ok(Self::InfoHash),
            "numSeeders" => Ok(Self::NumSeeders),
            "seeder" => Ok(Self::Seeder),
            "pieceLength" => Ok(Self::PieceLength),
            "numPieces" => Ok(Self::NumPieces),
            "connections" => Ok(Self::Connections),
            "errorCode" => Ok(Self::ErrorCode),
            "errorMessage" => Ok(Self::ErrorMessage),
            "followedBy" => Ok(Self::FollowedBy),
            "following" => Ok(Self::Following),
            "belongsTo" => Ok(Self::BelongsTo),
            "dir" => Ok(Self::Dir),
            "files" => Ok(Self::Files),
            // Bittorrent
            "verifiedLength" => Ok(Self::VerifiedLength),
            "veriyIntegrityPending" => Ok(Self::VeriyIntegrityPending),
            _ => Err("Invalid TellStatusField"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Active,
    Waiting,
    Paused,
    Error,
    Complete,
    Removed,
}

/// https://aria2.github.io/manual/en/html/aria2c.html#aria2.tellStatus
#[derive(Debug)]
pub struct TellStatus {
    pub gid: String,
    pub keys: Option<Vec<TellStatusField>>,
}

impl TellStatus {
    /// create a new TellStatus
    pub fn new<G: Into<String>>(gid: G) -> Self {
        Self {
            gid: gid.into(),
            keys: None,
        }
    }

    /// replace all fields wanted to be returned
    pub fn keys<I, F>(mut self, keys: Option<I>) -> Result<Self, F::Error>
    where
        I: IntoIterator<Item = F>,
        F: TryInto<TellStatusField, Error = &'static str>,
    {
        let mut temp = vec![];
        if let Some(keys) = keys {
            for field in keys.into_iter() {
                match field.try_into() {
                    Ok(f) => temp.push(f),
                    Err(e) => return Err(e),
                }
            }
        }

        self.keys = Some(temp);
        Ok(self)
    }

    /// add key
    /// ignore invalid key
    pub fn key<F>(self, key: F) -> Result<Self, F::Error>
    where
        F: TryInto<TellStatusField, Error = &'static str>,
    {
        let field = key.try_into()?;
        Ok(self.field(field))
    }

    pub fn new_with_fields<G, I, F>(gid: G, fields: I) -> Self
    where
        G: Into<String>,
        I: IntoIterator<Item = F>,
        F: Into<TellStatusField>,
    {
        Self {
            gid: gid.into(),
            keys: Some(fields.into_iter().map(|f| f.into()).collect()),
        }
    }

    pub fn field<F>(mut self, field: F) -> Self
    where
        F: Into<TellStatusField>,
    {
        if let Some(ref mut keys) = self.keys {
            keys.push(field.into());
        } else {
            self.keys = Some(vec![field.into()]);
        }
        self
    }
}

#[serde_as]
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TellStatusReply {
    pub gid: Option<String>,
    pub status: Option<TaskStatus>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub total_length: Option<u64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub completed_length: Option<u64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub upload_length: Option<u64>,
    pub bitfield: Option<String>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub download_speed: Option<u64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub upload_speed: Option<u64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub piece_length: Option<u64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub num_pieces: Option<u64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub connections: Option<u64>,
    pub dir: Option<String>,
    pub files: Option<Vec<TellStatusReplyFile>>,
}

#[serde_as]
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TellStatusReplyFile {
    pub index: String,
    #[serde_as(as = "DisplayFromStr")]
    pub length: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub completed_length: u64,
    pub path: String,
    #[serde_as(as = "DisplayFromStr")]
    pub selected: bool,
    pub uris: Vec<TellStatusReplyUri>,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum URIStatus {
    Used,
    Waiting,
}
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TellStatusReplyUri {
    pub status: URIStatus,
    pub uri: String,
}

impl Call for TellStatus {
    type Response = TellStatusReply;

    fn method(&self) -> &'static str {
        "aria2.tellStatus"
    }

    fn serialize_params<S: SerializeSeq>(&self, serializer: &mut S) -> Result<(), S::Error> {
        serializer.serialize_element(&self.gid)?;
        option_element!(self.keys, serializer);
        Ok(())
    }
}