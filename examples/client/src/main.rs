use std::time::Duration;

use shared::{GetRequest, PutRequest};
use tokio::{process::Command, select};

async fn amain() {
    let mut builder = pluginx::client::ClientBuilder::new(pluginx::client::config::ClientConfig {
        handshake_config: shared::HANDSHAKE_CONFIG,
        cmd: Command::new("/root/code/pluginx/server"),
        broker_multiplex: false,
        port_range: None,
        startup_timeout: Duration::from_secs(1),
    })
    .await;
    builder.add_plugin(shared::KvPlugin).await;
    let client = builder.build();

    let mut client = client.dispense::<shared::KvPlugin>().unwrap();
    dbg!(client
        .put(PutRequest {
            key: "aaa".to_string(),
            value: b"aaa".to_vec()
        })
        .await
        .unwrap());
    loop {
        let resp = client
            .get(GetRequest {
                key: "aaa".to_string(),
            })
            .await
            .unwrap();
        dbg!(resp);

        // ctrlc or infinity loop sleep 1s
        select! {
            _ = tokio::signal::ctrl_c() => {
                break;
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {}

        }
    }
}

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(amain());
}
