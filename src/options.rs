#[derive(Debug, serde::Serialize)]
pub struct BasicOptions{
    pub dir: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct XXTPOptions{
    out: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct HTTPOptions{
    pub referer: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct Aria2Options {
    #[serde(flatten)]
    basic: BasicOptions,
    #[serde(flatten)]
    http_ftp_sftp: XXTPOptions,
    #[serde(flatten)]
    http: HTTPOptions,
}