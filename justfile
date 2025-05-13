set quiet

fix:
    cargo clippy --fix --allow-dirty

build:
    cargo build

client: build
    cargo build --example client --no-default-features --features cdylib

io_layout:
    cargo run --example io_layout

io_layout_thr:
    cargo run --example io_layout_thr

io_runtime: sink source
    cargo run --example io_runtime

message_complex:
    cargo run --example message_complex

message_derive:
    cargo run --example message_derive

message_enum_derive:
    cargo run --example message_enum_derive

message_enum_impl:
    cargo run --example message_enum_impl

message_impl:
    cargo run --example message_impl

service: build
    cargo build --example service --no-default-features --features cdylib

service_layout:
    cargo run --example service_layout

service_runtime: client service
    cargo run --example service_runtime

sink: build
    cargo build --example sink --no-default-features --features cdylib

source: build
    cargo build --example source --no-default-features --features cdylib
