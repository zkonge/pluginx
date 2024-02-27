fn main() {
    tonic_build::configure()
        .build_transport(false)
        .compile(&["proto/kv.proto"], &["proto"])
        .unwrap();
}
