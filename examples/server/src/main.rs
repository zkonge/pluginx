use std::collections::HashMap;

use pluginx::{server::config::ServerConfig, Request, Response, Status};
use shared::{Empty, GetRequest, GetResponse, PutRequest};
use tokio::sync::Mutex;

struct KvImpl(Mutex<HashMap<String, Vec<u8>>>);

#[pluginx::async_trait]
impl shared::kv_server::Kv for KvImpl {
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        match self.0.lock().await.get(&request.get_ref().key).to_owned() {
            Some(value) => Ok(Response::new(GetResponse {
                value: value.to_owned(),
            })),
            None => Err(Status::not_found("Key not found")),
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
    let mut server = pluginx::server::Server::new(ServerConfig {
        handshake_config: shared::HANDSHAKE_CONFIG,
    })
    .unwrap();

    server
        .add_plugin(shared::kv_server::KvServer::new(KvImpl(Default::default())))
        .await;

    server.run().await.unwrap()
}

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(amain());
}
