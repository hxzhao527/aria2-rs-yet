pub mod call;
mod error;
mod options;
mod ws;

/// https://www.jsonrpc.org/specification
mod jsonrpc {
    #[derive(serde::Serialize)]
    pub struct Request<'a, I, S> {
        pub jsonrpc: &'a str, // jsonrpc must be "2.0"
        pub method: &'a str,  // A String containing the name of the method to be invoked.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id: Option<I>, // An identifier established by the Client that MUST contain a String, Number, or NULL value if included.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub params: Option<S>, // A Structured value that holds the parameter values to be used during the invocation of the method.
    }

    #[derive(serde::Deserialize, Debug, Clone)]
    pub struct Error {
        pub code: i64,
        pub message: String,
    }

    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    pub enum Response<I, R, N> {
        Resp { id: I, result: R },
        Notification { method: String, params: N },
        Err { id: I, error: Error },
    }
}




pub use error::Error;
pub use ws::{Client, ConnectionMeta};

pub type Result<T> = std::result::Result<T, Error>;

