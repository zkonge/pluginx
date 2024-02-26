fn main() {
    tonic_build::configure()
        .compile(
            &[
                "proto/grpc_broker.proto",
                "proto/grpc_controller.proto",
                "proto/grpc_stdio.proto",
            ],
            &["proto"],
        )
        .unwrap();
}
