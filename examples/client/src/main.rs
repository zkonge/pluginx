fn main() {
    let client = pluginx::Client::new(pluginx::ClientConfig {
        handshake_config: shared::HANDSHAKE_CONFIG,
    });
}
