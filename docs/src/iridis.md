# `iridis`

`iridis` is a rust library for building dataflow applications. It provides simple APIs to:

- Make a standalone node in Rust
- Assemble nodes into a dataflow graph
- Customize the runtime with plugins

`iridis` is inspired by both [`dora-rs`](https://github.com/dora-rs/dora) and [`zenoh-flow`](https://github.com/eclipse-zenoh-flow/zenoh-flow), two projects that focus on dataflow programming. It aims to provide a more lightweight, up to date and extensible solution for building dataflow applications in Rust.

# Features

- Fast builds: `iridis` is designed to be simple and easy to use. It tries to lower the number of dependencies and the complexity of the code so that each time you want to try your application you don't have to spend dozen of seconds to compile it!

- Fast, that's all: `iridis` design is based on `shared-library` loading. So that each node is in fact the same global `process`, allowing for sharing data effectively.

- Async: `iridis` is built on top of `tokio`, a powerful async runtime for Rust. This allows you to build highly concurrent applications with ease.

- Extensible: `iridis` provides a plugin system that allows you to extend the runtime with custom plugins. This allows you to customize the behavior of the runtime to suit your needs. (see [`pyridis`](https://github.com/iridis-rs/pyridis) for a example of a plugin that allows you to use `python` as a node in the dataflow graph)

# Related projects

- [`pyridis`](https://github.com/iridis-rs/pyridis) - A plugin for `iridis` that allows you to use `python` as a node in the dataflow graph.
- [`distore`](https://github.com/iridis-rs/distore) - A store repository that allows you to use already made nodes in your application.
- [`iris`](https://github.com/iridis-rs/iris) - A CLI tool that allows you to create applicatios directly from a YAML file.
