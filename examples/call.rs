use aria2_rs_yet::{Client, ConnectionMeta, Result};
use aria2_rs_yet::call::{SystemListMethods, GetVersion, AddUri};
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

    let methods = client.call(SystemListMethods).await?;
    println!("{:?}", methods);

    let version = client.call(GetVersion).await?;
    println!("{:?}", version);

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

    // drop(client); // uncomment this line to see the client disconnecting
    println!("waiting for ctrl-c");
    tokio::signal::ctrl_c().await.unwrap();
    Ok(())
}