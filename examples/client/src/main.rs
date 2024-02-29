use std::time::Duration;

use shared::{GetRequest, PutRequest};
use tokio::{process::Command, select};

async fn amain() {
    let builder = pluginx::client::ClientBuilder::new(pluginx::client::config::ClientConfig {
        handshake_config: shared::HANDSHAKE_CONFIG,
        cmd: Command::new("/root/code/pluginx/server"),
        broker_multiplex: false,
        port_range: None,
        startup_timeout: Duration::from_secs(1),
    })
    .await
    .unwrap();
    let client = builder.add_plugin(shared::KvPlugin).await.build();

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
        let resp = kv_client.get(GetRequest { key: "aaa".into() }).await.unwrap();
        let resp = resp.into_inner();

        println!("aaa = {:?}", resp.value);

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
