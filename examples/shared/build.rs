fn main() {
    tonic_build::configure()
        .build_transport(false)
        .compile_protos(&["proto/kv.proto"], &["proto"])
        .unwrap();
}
