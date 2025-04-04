install-ubuntu:
    sudo apt-get install mold clang

rustup:
    rustup component add rustc-codegen-cranelift-preview --toolchain nightly
    rustup component add rust-analyzer

python-dev:
    uv sync --directory crates/flarrow-message-python --extra tests

fix:
    cargo fix --workspace --allow-dirty
    cargo clippy --fix --allow-dirty

format:
    uv run --directory crates/flarrow-message-python ruff format
