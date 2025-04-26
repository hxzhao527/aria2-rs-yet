# Aria2-rs
[![Crates.io](https://img.shields.io/crates/v/aria2-rs-yet.svg)](https://crates.io/crates/aria2-rs-yet)

Yet Another Aria2 JSON-RPC Client. Inspired by [aria2-rs](https://github.com/ihciah/aria2-rs)

## Features
- [x] Simple direct call via websocket.
- [x] Notification from websocket.

## example

download a file and print taks status.

```rust
use aria2_rs_yet::{Client, ConnectionMeta, Result};
use aria2_rs_yet::call::{AddUri, TellStatus, TellStatusField};
use aria2_rs_yet::options::Aria2Options;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();
    
    let (client, _) = Client::connect(
        ConnectionMeta::new(
            "ws://localhost:6800/jsonrpc",
            Some("<rpc-secret>"),
        )
    ).await?;

    let gid = client.call(
        AddUri::new(
            vec!["https://github.com/hxzhao527/aria2-rs-yet/archive/refs/heads/master.zip"],
            Some(Aria2Options{
                dir: Some("/tmp".to_string()),
                out: Some("aria2-rs-yet.zip".to_string()),
                ..Default::default()
            }),
            None,
        )
    ).await?;
    println!("{:?}", gid);

    let status = client.call(
        TellStatus::new(gid).fields(Some([TellStatusField::Status, TellStatusField::Gid]))
    ).await?;
    println!("{:?}", status);

    // drop(client); // uncomment this line to see the client disconnecting
    println!("waiting for ctrl-c");
    tokio::signal::ctrl_c().await.unwrap();
    Ok(())
}
```

## License
MIT License