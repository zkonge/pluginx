fn main() {
    tonic_build::configure()
        .build_transport(false)
        .compile(&["proto/example.proto"], &["proto"])
        .unwrap();
}
