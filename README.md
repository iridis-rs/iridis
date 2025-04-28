# `flarrow-runtime`

Flarrow (flow + arrow) is a rust framework for building dataflow applications. The `flarrow-runtime` repo defines crates that lets you define logical layouts for your application, load nodes into the runtime and execute it. Flarrow provides a simple and intuitive API for defining dataflow applications, making it easy to build complex systems with minimal effort.

# Features

- Async tokio runtime everywhere so you can easily use your async code to determine each step of the application.
- Message passing between nodes with tokio channels.
- Message passing without copy thanks to the Apache `arrow` format.
- Service system with Query/Queryable primitive.
- 100% pluggable!

# How it works

## Layout

The first thing to do is to define the layout of your application. This is done by creating a `DataflowLayout` instance and then adding nodes to it. When creating a node the user provides a closure that takes a mutable reference to a `NodeIOBuilder` and returns optionally a future that resolves to a tuple of input and output streams. This is `async` compatible, so you can use `async` code inside the closure if you want to perform additional asynchronous operations.

```rust
use flarrow_layout::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() {
    let mut layout = DataflowLayout::new();

    let (source, output) = layout
        .node("source", async |builder: &mut NodeIOBuilder| {
            builder.output("out")
        })
        .await;

    let (operator, (op_in, op_out)) = layout
        .node("operator", async |builder: &mut NodeIOBuilder| {
            (builder.input("in"), builder.output("out"))
        })
        .await;

    let (sink, input) = layout
        .node("sink", async |builder: &mut NodeIOBuilder| {
            builder.input("in")
        })
        .await;

    let layout = layout.build();

    Ok(())
}
```

## Node API

You must then create the implementation of your nodes. You can either make a rust library or a `cdylib` to be passed to the `flarrow-runtime`. It relies on a `tokio` runtime: it will choose the current one if there is one available (`crate`) or create a new one if none is available (`cdylib`). You can totally control the runtime by passing a custom function instead of `default_runtime`.

```rust
use flarrow_api::prelude::{thirdparty::*, *};

#[derive(Node)]
pub struct MySink {
    pub input: Input<String>,
}

#[node(runtime = "default_runtime")]
impl Node for MySink {
    async fn new(
        mut inputs: Inputs,
        _: Outputs,
        _: Queries,
        _: Queryables,
        _: serde_yml::Value,
    ) -> Result<Self> {
        Ok(Self {
            input: inputs.with("in").await.wrap_err("Failed to create input")?,
        })
    }

    async fn start(mut self: Box<Self>) -> Result<()> {
        while let Ok((_, message)) = self.input.recv().await {
            println!("Received message: {}", message);
        }

        Ok(())
    }
}
```

Where `default_runtime` is:

```rust
static DEFAULT_TOKIO_RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> =
    std::sync::LazyLock::new(|| tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));

fn default_runtime<T: Send + 'static>(
    task: impl Future<Output = T> + Send + 'static,
) -> tokio::task::JoinHandle<T> {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle.spawn(task),
        Err(_) => DEFAULT_TOKIO_RUNTIME.spawn(task)
    }
}
```

**Note**: The code above is automatically added when you use `#[derive(Node)]`.

**Note**: The `crate` or `cdylib` version can have the same code! But in order to get a usable `libX.so` you need to compile your crate with the `--features cdylib` flag. If enabled it will generate the symbols for your node.

## Flows

Now you've created a layout and implemented your nodes, you can create the connections between the nodes. This is done by creating a `Flows` struct. As you can see, the flows creation are defined in an `async` closure so you can use `async` code in it.

```rust
use flarrow_layout::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() {
    let mut layout = DataflowLayout::new();

    let (source, output) = layout
        .node("source", async |builder: &mut NodeIOBuilder| {
            builder.output("out")
        })
        .await;

    let (operator, (op_in, op_out)) = layout
        .node("operator", async |builder: &mut NodeIOBuilder| {
            (builder.input("in"), builder.output("out"))
        })
        .await;

    let (sink, input) = layout
        .node("sink", async |builder: &mut NodeIOBuilder| {
            builder.input("in")
        })
        .await;

    let layout = layout.build();

    let flows = Flows::new(layout.clone(), async move |builder: &mut FlowsBuilder| {
        builder.connect(op_in, output, None)?;
        builder.connect(input, op_out, None)?;

        Ok(())
    })
    .await?;

    Ok(())
}
```

## Runtime

This part is the most complex one. The `flarrow-runtime` is pluggable, and so you need to understand what kind of plugin one can provide.

### `RuntimePlugin`

The first kind of plugin is the `RuntimePlugin`. You can provide the `flarrow-runtime` multiple `RuntimePlugin` and they will all be launched together with the classic runtime and they will be able to interact with it, manipulate your nodes, expose them to the internet etc...

There is only one official `RuntimePlugin` backed by `flarrow-rs`:

- `FlarrowZenohPlugin`: this plugin will expose all your nodes to the `zenoh` network, making it possible to monitor your flows and even send messages to specific nodes

### Plugin when loading nodes

When loading nodes to the `flarrow-runtime`, you can either linked them statically (when you use the Cargo `crate` directly), or load them via an `url` in this format: `scheme://some_address`.

#### `SchemePlugin`

A scheme plugin is responsible of handling all `url` starting with the `scheme` they are targeting. By default, with no additional plugins, the `flarrow-runtime` can handle `file:///path/to/file.{dylib}` schemes and `builtin://name/of/a/builtin/node`.

You can then add your own plugin to handle other custom schemes (`http://`, `ssh://`, etc...)

**How it works**: Usually, a `SchemePlugin` handles an `url` by doing some work and then returning a `PathBuf` that will be handled by the `ExtensionPlugin` corresponding to this file extension.

#### `ExtensionPlugin`

An extension plugin is responsible of handling a `PathBuf` with the file extension it's targeting. By default, with no additional plugins, the `flarrow-runtime` can hanle `.so`, `.dylib` and `.dll` file extensions.

There is only one official `ExtensionPlugin` backed by `flarrow-rs`:

- `PythonPlugin`: this plugin handles `.py` files

# Examples

You can run the examples provided in this repository. For convenience use the `justfile` provided:

- `just simple_runtime`, this launches a simple dataflow (the one in this README)
- `just service_runtime`, this launches a dataflow with a service node and a client node
