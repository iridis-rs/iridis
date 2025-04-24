fix:
    cargo fix --workspace --allow-dirty
    cargo clippy --fix --allow-dirty

build:
    cargo build
    cargo build --examples
    cargo build --example sink --features cdylib
    cargo build --example service --features cdylib
    cargo build --example client --features cdylib

simple_runtime: build
    cargo run --example simple_runtime

service_runtime: build
    cargo run --example service_runtime
