fn main() {
    let protos = [
        "proto/grpc_broker.proto",
        "proto/grpc_controller.proto",
        "proto/grpc_stdio.proto",
    ];

    tonic_prost_build::configure()
        .compile_protos(&protos, &["proto"])
        .unwrap();
}
