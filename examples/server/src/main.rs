use std::{collections::HashMap, time::Duration};

use pluginx::{
    meta_plugin::StdioType,
    server::{config::ServerConfig, Server},
    Request, Response, Status,
};
use shared::{kv_server::KvServer, Empty, GetRequest, GetResponse, PutRequest};
use tokio::sync::Mutex;

struct KvImpl(Mutex<HashMap<String, Vec<u8>>>);

#[pluginx::async_trait]
impl shared::kv_server::Kv for KvImpl {
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        match self.0.lock().await.get(&request.get_ref().key).to_owned() {
            Some(value) => Ok(Response::new(GetResponse {
                value: value.to_owned(),
            })),
            None => Err(Status::not_found("key not found")),
        }
    }

    async fn put(&self, request: Request<PutRequest>) -> Result<Response<Empty>, Status> {
        self.0.lock().await.insert(
            request.get_ref().key.to_owned(),
            request.get_ref().value.to_owned(),
        );
        Ok(Response::new(Empty {}))
    }
}

async fn amain() {
    let mut server = Server::new(ServerConfig {
        handshake_config: shared::HANDSHAKE_CONFIG,
    })
    .await
    .unwrap();

    server
        .add_plugin(KvServer::new(KvImpl(Default::default())))
        .await;

    let stdio = server.stdio_handler();
    let stdio_cloned = stdio.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await;
        stdio_cloned
            .write(StdioType::Stdout, b"hello".to_vec())
            .await
            .unwrap();
    });

    server.run().await.unwrap();
}

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(amain());
}
