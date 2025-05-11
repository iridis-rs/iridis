# Installation

## Prerequisites

To use `iridis`, you need to have the following installed:

- `rustup`
- `cargo`

Because of the `shared-library` design of `iridis`, you need to use the same `rust` toolchain for all the nodes, plugins and applications. This project is usig the latest stable version of `rust`, so you should use the same version for your own projects. See [rust-toolchain.toml](https://github.com/iridis-rs/iridis/blob/main/rust-toolchain.toml).

## For an application

Each application that uses `iridis` is a `Cargo` binary crate. You can create a new `Cargo` binary crate using the following command:

```bash
cargo new --bin my_app
```

Then, add `iridis` as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
iridis = "0.3"
```

Or add it with `cargo add`:

```bash
cargo add iridis
```

## For a node

Each node that uses `iridis` is a `Cargo` library crate (optionally `cdylib`). You can create a new `Cargo` library crate using the following command:

```bash
cargo new --lib my_node
```

Then, add `iridis-node` as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
iridis-node = "0.3"
```

Or add it with `cargo add`:

```bash
cargo add iridis-node
```

Because a node can be compiled as a `cdylib`, you need to add the following lines in your `Cargo.toml` file:

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[features]
cdylib = []
```

**Note**: If you only intend to use the node as a static library, you can remove the `cdylib` field from the `crate-type` list. However, keep the `cdylib` field in the `features` section, otherwise you will get a warning.

**Note:** The `iridis-node` crate is included in the `iridis` crate, so you don't need to add it as a dependency if you are using `iridis` directly.

## For a plugin

Two different kind of plugins are available in `iridis` as for now:

- `FileExtPlugin`: this plugin allows you to load a file with a specific extension. For example, if you want to load a file with the `.py` extension, you need to add the `PythonFileExt` plugin in your runtime.

- `UrlSchemePlugin`: this plugin allows you to load a different kind of `url` than the default `file://` scheme. For example, if you want to load a node from the `http://` scheme, you would need to add some kind of `http` plugin. For now, only the `file://` and `builtin://` schemes are available in `iridis`, but you can create your own plugin to load a node from a different scheme.

The installation procedure is the same as for a node, but the crate to add is either `iridis-file-ext` or `iridis-url-scheme`.

**Note:** The `iridis-file-ext` crate and `iridis-url-scheme` crate are included in the `iridis` crate, so you don't need to add them as a dependency if you are using `iridis` directly.

## For a message

Each message used in `iridis` is an `Arrow` data-format message defined by the `ArrowMessage` trait in the `iridis-message` crate. If you want to create custom messages, you need to add the `iridis-message` crate as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
iridis-message = "0.3"
```

Or add it with `cargo add`:

```bash
cargo add iridis-message
```

**Note:** The `iridis-message` crate is included in the `iridis` crate, so you don't need to add it as a dependency if you are using `iridis` directly.
