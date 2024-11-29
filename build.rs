fn main() {
    tonic_build::configure()
        .compile_protos(
            &[
                "proto/grpc_broker.proto",
                "proto/grpc_controller.proto",
                "proto/grpc_stdio.proto",
            ],
            &["proto"],
        )
        .unwrap();
}
