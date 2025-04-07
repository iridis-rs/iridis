python-dev:
    uv sync --directory crates/flarrow-message-python --extra tests

fix:
    cargo fix --workspace --allow-dirty
    cargo clippy --fix --allow-dirty

format:
    cargo fmt
    uv run --directory crates/flarrow-message-python ruff format
