use crate::jsonrpc;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Rpc error {0}")]
    Rpc(RpcError),
    #[error("Decode error {0}")]
    Decode(serde_json::Error),
    #[error("Decode error {0}")]
    Encode(serde_json::Error),
    #[error("Connect error {0}")]
    Connect(tokio_tungstenite::tungstenite::error::Error),
    #[error("Request send error")]
    ChannelSend,
    #[error("Response send error {0}")]
    ChannelRecv(#[from] tokio::sync::oneshot::error::RecvError),
    #[error("Websocket error {0}")]
    Websocket(#[from] tokio_tungstenite::tungstenite::Error),
}


#[derive(serde::Deserialize, Debug, Clone)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RpcError: {{\"code\": {}, \"message\": \"{}\"}}",
            self.code, self.message
        )
    }
}
impl std::error::Error for RpcError {}

impl From<jsonrpc::Error> for Error {
    fn from(err: jsonrpc::Error) -> Self {
        Error::Rpc(RpcError {
            code: err.code,
            message: err.message,
        })
    }
}