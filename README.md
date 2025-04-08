# flarrow

Flarrow (flow + arrow) is a rust runtime/framework for building dataflow applications. It lets you define logical layouts for your application and then loading nodes into the runtime. Flarrow provides a simple and intuitive API for defining dataflow applications, making it easy to build complex systems with minimal effort.

# Features

- Async tokio runtime everywhere so you can easily use your async code.
-

# How it works

## Layout

The first thing to do is to define the layout of your application. This is done by creating a `DataflowLayout` instance and then adding nodes to it. When creating a node the user provides a closure that takes a mutable reference to a `NodeIO` and returns optionally a future that resolves to a tuple of input and output streams. This is `async` compatible, so you can use `async` code inside the closure if you want to perform additional asynchronous operations.

```rust
#[tokio::main]
async fn main() {
    let mut layout = DataflowLayout::new();

    let (source, output) = layout
        .create_node(async |io: &mut NodeIO| io.open_output("out"))
        .await;

    let (operator, (op_in, op_out)) = layout
        .create_node(async |io: &mut NodeIO| (io.open_input("in"), io.open_output("out")))
        .await;

    let (sink, input) = layout
        .create_node(async |io: &mut NodeIO| io.open_input("in"))
        .await;

    Ok(())
}
```

## Node API

You must then create the implementation of your nodes. You can either make a rust library or a `cdylib` to be passed to the `flarrow-runtime`. It relies on a `tokio` runtime: it will choose the current one if there is one available (`rlib`) or create a new one if none is available (`cdylib`). You can totally control the runtime by passing a custom function instead of `default_runtime`.

```rust
use flarrow_api::prelude::*;

#[derive(Node)]
pub struct MySink {
    pub input: Input<String>,
}

#[node(runtime = "default_runtime")]
impl Node for MySink {
    async fn new(mut inputs: Inputs, _: Outputs, _: serde_yml::Value) -> Result<Box<dyn Node>>
    where
        Self: Sized,
    {
        Ok(Box::new(Self {
            input: inputs.with("in").await.wrap_err("Failed to create input")?,
        }) as Box<dyn Node>)
    }

    async fn start(mut self: Box<Self>) -> Result<()> {
        while let Ok((_, message)) = self.input.recv_async().await {
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

## Runtime

Now you've created a layout and implemented your nodes, you can create the connections between the nodes and load the implementation for each one. This is done by creating a `Flows` struct first, and then creating a `DataflowRuntime` instance. As you can see, the flows creation are defined in an `async` closure so you can use `async` code in it. You can also see that you can load the implementation for each node using the either `load_statically_linked` or `load_from_url`. The first one is intented to be used with `rlib` nodes, and the second one is intented to be used with `cdylib` or `builtin` nodes (which are `rlib` nodes already integrated into the runtime).

```rust
let flows = Flows::new(layout.clone(), async move |connector: &mut Connector| {
    connector.connect(op_in, output)?;
    connector.connect(input, op_out)?;

    Ok(())
})
.await?;

let runtime = DataflowRuntime::new(
    flows,
    Some(
        RuntimeUrlPlugin::new_statically_linked::<UrlDefaultPlugin>()
            .await
            .wrap_err("Failed to load URL plugin")?,
    ),
    async move |loader: &mut Loader| {
        loader
            .load_statically_linked::<MyOperator>(operator, serde_yml::Value::from(""))
            .await
            .wrap_err("Failed to load MyOperator")?;

        let source_file = Url::parse("builtin:///timer")?;
        let sink_file = Url::parse(&format!("{}/libsink.so", examples))?;

        loader
            .load_from_url(source, source_file, serde_yml::from_str("frequency: 5.0")?)
            .await
            .wrap_err("Failed to load source")?;
        loader
            .load_from_url(sink, sink_file, serde_yml::Value::from(""))
            .await
            .wrap_err("Failed to load sink")?;

        Ok(())
    },
)
.await?;
```

Finally you can start the runtime by calling `runtime.run().await`

## A word about `load_from_url`

To use `load_from_url`, you need to provide a URL Plugin that implements the `UrlPlugin` trait. It can be either a statically linked `rlib` or a dynamically linked `cdylib`. The URL Plugin is responsible for loading the node implementation from the provided URL. You can create your own URL Plugin to support various protocols and formats, such as HTTP, HTTPS, FTP or .py files.

The provided `UrlDefaultPlugin` only works with the `builtin://` and `file://` schemes.

# Examples

You can run the examples provided in this repository. Start by building the all:

```
cargo build --examples
```

And then you can run the `runtime` example:

```
cargo run --example runtime
```
