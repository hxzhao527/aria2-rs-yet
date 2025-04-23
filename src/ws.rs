use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{timeout, Duration};
use tokio_tungstenite::tungstenite;
use std::ops::Deref;
use std::sync::Arc;

use crate::call::Call;
use crate::error::Error;
use crate::jsonrpc;
use crate::Result;

type WSMessage = tokio_tungstenite::tungstenite::Message;
type WSStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Debug, Clone)]
pub enum Notification {
    DownloadStart(String),
    DownloadPause(String),
    DownloadStop(String),
    DownloadComplete(String),
    DownloadError(String),
    BtDownloadComplete(String),
}

impl Notification {
    pub fn new(method: &str, gid: String) -> Self {
        match method {
            "aria2.onDownloadStart" => Self::DownloadStart(gid),
            "aria2.onDownloadPause" => Self::DownloadPause(gid),
            "aria2.onDownloadStop" => Self::DownloadStop(gid),
            "aria2.onDownloadComplete" => Self::DownloadComplete(gid),
            "aria2.onDownloadError" => Self::DownloadError(gid),
            "aria2.onBtDownloadComplete" => Self::BtDownloadComplete(gid),
            _ => unreachable!(),
        }
    }
}

#[derive(serde::Deserialize)]
struct NotificationParam {
    gid: String,
}

struct RPCRequest {
    params: Option<serde_json::Value>,
    method: &'static str,
    handler: oneshot::Sender<RPCReponse>,
}

enum RPCReponse {
    Success(serde_json::Value),
    Error(jsonrpc::Error),
}

pub struct ConnectionMeta {
    pub url: String,
    pub token: Option<String>,
}

impl ConnectionMeta{
    pub fn new(url: &str, token: Option<&str>) -> Self {
        Self {
            url: url.to_string(),
            token: token.map(|s| format!("token:{}", s)),
        }
    }
}

impl tungstenite::client::IntoClientRequest for &ConnectionMeta{
    fn into_client_request(self) -> tungstenite::Result<tungstenite::handshake::client::Request> {
        // add header here if needed
        self.url.as_str().into_client_request()
    }
}

#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

impl Deref for Client {
    type Target = ClientInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Client {
    pub async fn connect(meta: ConnectionMeta) -> Result<(Self, mpsc::UnboundedReceiver<Notification>)>{
        let (inner, notify_rx) = ClientInner::connect(meta).await?;
        let client = Client {
            inner: Arc::new(inner),
        };
        Ok((client, notify_rx))
    }
}

pub struct ClientInner {
    message_tx: mpsc::Sender<RPCRequest>,
    token: Option<String>,
    _drop_rx: oneshot::Receiver<()>,
}

impl ClientInner {
    async fn connect(
        meta: ConnectionMeta,
    ) -> Result<(Self, mpsc::UnboundedReceiver<Notification>)> {
        let (ws, _) = tokio_tungstenite::connect_async(&meta)
            .await
            .map_err(Error::Connect)?;
        let (message_tx, message_rx) = mpsc::channel(32);
        let (notification_tx, notification_rx) = mpsc::unbounded_channel();
        let (drop_tx, _drop_rx) = oneshot::channel();
        let token = meta.token.clone();
        tokio::spawn(Self::background(
            ws,
            meta,
            message_rx,
            drop_tx,
            notification_tx,
        ));
        Ok((
            Self {
                message_tx,
                token,
                _drop_rx,
            },
            notification_rx,
        ))
    }

    pub async fn call<C: Call>(&self, call: C) -> Result<C::Response> {
        let (tx, rx) = oneshot::channel();

        let method = call.method();
        let params = match call.to_params(self.token.as_ref().map(AsRef::as_ref)) {
            Some(params) => Some(serde_json::to_value(params).map_err(Error::Encode)?),
            None => None,
        };

        let request = RPCRequest {
            params,
            method,
            handler: tx,
        };
        self.message_tx
            .send(request)
            .await
            .map_err(|_| Error::ChannelSend)?;
        match rx.await.map_err(Error::ChannelRecv)? {
            RPCReponse::Success(value) => {
                serde_json::from_value(value).map_err(Error::Decode)
            }
            RPCReponse::Error(err) => Err(err.into()),
        }
    }

    async fn background(
        ws: WSStream,
        meta: ConnectionMeta,
        mut message_rx: mpsc::Receiver<RPCRequest>,
        mut drop_tx: oneshot::Sender<()>,
        notification_tx: mpsc::UnboundedSender<Notification>,
    ) {
        let (mut ws_tx, mut ws_rx) = ws.split();
        let mut shutdown = tokio::spawn({
            let notification_tx = notification_tx.clone();
            async move {
                tokio::join!(drop_tx.closed(), notification_tx.closed());
            }
        });

        let mut request_id = 1i64;
        let mut pending_requests = std::collections::HashMap::new();

        loop {
            loop {
                if notification_tx.is_closed() && message_rx.is_closed() {
                    tracing::info!("background task shutdown");
                    return;
                }
                tokio::select! {
                    _ = &mut shutdown => {
                        tracing::info!("background task shutdown");
                        return;
                    }
                    Some(msg) = message_rx.recv() => {
                        request_id += 1;
                        pending_requests.insert(request_id, msg.handler);

                        if let Err(e) = timeout(
                            Duration::from_secs(10),
                           Self::send_request(&mut ws_tx, request_id, msg.method, msg.params,)
                        ).await {
                            tracing::error!("send request error: {e}");
                            break;
                        }
                    }
                    Some(msg) = ws_rx.next() => {
                        let text = match msg {
                            Ok(WSMessage::Text(text)) => text,
                            Ok(WSMessage::Close(_)) => {
                                tracing::info!("websocket closed");
                                break;
                            }
                            Ok(_) => {
                                continue;
                            }
                            Err(e) => {
                                tracing::error!("websocket error: {e}");
                                break;
                            }
                        };
                        Self::handle_response(&text, &mut pending_requests, notification_tx.clone());
                    }
                }
            }
            pending_requests.clear();

            // reconnect
            loop {
                if notification_tx.is_closed() && message_rx.is_closed() {
                    tracing::info!("background task shutdown");
                    return;
                }
                match timeout(
                    Duration::from_secs(10),
                    tokio_tungstenite::connect_async(&meta),
                )
                .await
                {
                    Err(e) => {
                        tracing::error!("reconnect error: {e}, will retry in 10 seconds");
                        tokio::time::sleep(Duration::from_secs(10)).await;
                    }
                    Ok(Err(e)) => {
                        tracing::error!("reconnect timeout: {e}, will retry in 10 seconds");
                        tokio::time::sleep(Duration::from_secs(10)).await;
                    }
                    Ok(Ok((new_ws, _))) => {
                        let (tx, rx) = new_ws.split();
                        ws_tx = tx;
                        ws_rx = rx;
                        break;
                    }
                }
            }
        }
    }

    async fn send_request(
        sink: &mut SplitSink<WSStream, WSMessage>,
        id: i64,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<()> {
        let rpc_req = jsonrpc::Request {
            id: Some(id),
            jsonrpc: "2.0",
            method,
            params,
        };
        sink.send(WSMessage::Text(
            serde_json::to_string(&rpc_req)
                .map_err(Error::Encode)?
                .into(),
        ))
        .await
        .map_err(Error::Websocket)
    }

    fn handle_response(
        text: &str,
        pending_requests: &mut std::collections::HashMap<i64, oneshot::Sender<RPCReponse>>,
        notification_tx: mpsc::UnboundedSender<Notification>,
    ) {
        if let Ok(resp) = serde_json::from_str::<
            jsonrpc::Response<i64, serde_json::Value, Vec<NotificationParam>>,
        >(text)
        {
            match resp {
                jsonrpc::Response::Err { id, error } => {
                    if let Some(tx) = pending_requests.remove(&id) {
                        let _ = tx.send(RPCReponse::Error(error));
                    }
                }
                jsonrpc::Response::Resp { id, result } => {
                    if let Some(tx) = pending_requests.remove(&id) {
                        let _ = tx.send(RPCReponse::Success(result));
                    }
                }
                jsonrpc::Response::Notification { method, params } => {
                    tokio::spawn(async move {
                        let method = method;
                        for param in params {
                            if notification_tx.send(Notification::new(&method, param.gid)).is_err()
                            {
                                break;
                            }
                        }
                    });
                }
            }
        }
    }
}
