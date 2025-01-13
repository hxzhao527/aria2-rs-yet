use serde::ser::{SerializeSeq, Serializer};

use crate::options::Aria2Options;

pub trait Call {
    type Response: serde::de::DeserializeOwned;

    fn method(&self) -> &'static str;
    fn serialize_params<S: SerializeSeq>(&self, _serializer: &mut S) -> Result<(), S::Error>{
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

impl<'a, T> Aria2Params<'a, T>
{
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
            Self: Sized, {
        None
    }
}

#[derive(Debug)]
pub struct AddUri {
    pub uris: Vec<String>,
    pub options: Option<Aria2Options>,
    pub position: Option<i32>,
}

impl AddUri{
    pub fn new<S: Into<String>>(uris: Vec<S>, options: Option<Aria2Options>, position: Option<i32>) -> Self {
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