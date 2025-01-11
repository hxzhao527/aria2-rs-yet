use tokio::signal;

use aria2_rs_yet::{Client, ConnectionMeta, Result};


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();
    
    let (_, mut rx) = Client::connect(
        ConnectionMeta::new(
            "ws://localhost:6800/jsonrpc",
            Some("<rpc-secret>"),
        )
    ).await?;

    println!("waiting for ctrl-c");
    loop{
        tokio::select! {
            _ = signal::ctrl_c() => {
                break;
            }
            notification = rx.recv() => {
                println!("{:?}", notification);
            }
        }
    }
    
    Ok(())
}