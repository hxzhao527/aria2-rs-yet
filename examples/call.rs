use aria2_rs_yet::{Client, ConnectionMeta, Result};
use aria2_rs_yet::call::{SystemListMethods, GetVersion};

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
    // drop(client); // uncomment this line to see the client disconnecting
    println!("waiting for ctrl-c");
    tokio::signal::ctrl_c().await.unwrap();
    Ok(())
}