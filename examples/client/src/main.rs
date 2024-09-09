use std::{env::args, time::Duration};

use futures::StreamExt;
use pluginx::client::{config::ClientConfig, ClientBuilder, StdioData};
use shared::{GetRequest, PutRequest};
use tokio::{process::Command, select};

async fn amain() {
    let path = args().nth(1).expect("specify the plugin path");

    let mut builder = ClientBuilder::new(ClientConfig {
        handshake_config: shared::HANDSHAKE_CONFIG,
        cmd: Command::new(path),
        broker_multiplex: false,
        port_range: None,
        startup_timeout: Duration::from_secs(1),
    })
    .await
    .unwrap();
    builder.add_plugin(shared::KvPlugin).await;

    let mut client = builder.build();

    if let Ok(mut stdio) = client.stdio().unwrap().read().await {
        tokio::spawn(async move {
            while let Some(msg) = stdio.next().await {
                match msg {
                    StdioData::Stdout(x) => println!("stdout: {}", String::from_utf8_lossy(&x)),
                    StdioData::Stderr(x) => println!("stderr: {}", String::from_utf8_lossy(&x)),
                    _ => println!("invalid"),
                }
            }
        });
    }

    let mut kv_client = client.dispense::<shared::KvPlugin>().unwrap();

    // 1. put a data
    kv_client
        .put(PutRequest {
            key: "aaa".into(),
            value: b"value".into(),
        })
        .await
        .unwrap();

    // 2. try read data
    loop {
        let resp = kv_client
            .get(GetRequest { key: "aaa".into() })
            .await
            .unwrap();
        let resp = resp.into_inner();

        println!("aaa = {}", String::from_utf8_lossy(&resp.value));

        // ctrlc or infinity loop sleep 1s
        select! {
            _ = tokio::signal::ctrl_c() => {
                client.shutdown().await;
                break;
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {}
        }
    }
}

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(amain());
}
