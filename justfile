run-server:
    @echo "Running grpc-service server"
    cargo run --bin server

run-client:
    @echo "Running grpc-service client"
    cargo run --bin client
