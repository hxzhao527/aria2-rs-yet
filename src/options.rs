#[derive(Debug, serde::Serialize, Default)]
#[serde(rename_all = "kebab-case")]
#[serde_with::skip_serializing_none]
pub struct Aria2Options {
    // == basic
    pub dir: Option<String>,
    // == http_ftp_sftp
    pub out: Option<String>,
    // == http specific
    pub referer: Option<String>,
    pub user_agent: Option<String>,
}
