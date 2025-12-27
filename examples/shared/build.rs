fn main() {
    tonic_prost_build::configure()
        .build_transport(false)
        .compile_protos(&["proto/kv.proto"], &["proto"])
        .unwrap();
}
