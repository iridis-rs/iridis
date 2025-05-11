## Node

A node is always represented as a `rust` `struct` that derives the `Node` procedure and implements the `Node` trait. Here are some details about those procedures:

## The macros

### `#[derive(Node)]`

The `#[derive(Node)]` procedure will do two things:

- Create symbols if the `cdylib` feature is enabled, so that the `runtime` is able to load the node dynamically.

- Implement a default `tokio` `Runtime` that will automatically reuse the context `runtime` if the node has been statically linked, or create a new one if the node is dynamically linked.

The generated code is as follow:

```rust
#[cfg(feature = "cdylib")]
#[doc(hidden)]
#[unsafe(no_mangle)]
pub static IRIDIS_NODE: iridis_node::prelude::DynamicallyLinkedNodeInstance = |inputs, outputs, queries, queryables, configuration| {
    <#name>::new(inputs, outputs, queries, queryables, configuration)
};

static DEFAULT_TOKIO_RUNTIME: std::sync::LazyLock<iridis_node::prelude::thirdparty::tokio::runtime::Runtime> =
    std::sync::LazyLock::new(|| iridis_node::prelude::thirdparty::tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));

fn default_runtime<T: Send + 'static>(
    task: impl Future<Output = T> + Send + 'static,
) -> iridis_node::prelude::thirdparty::tokio::task::JoinHandle<T> {
    match iridis_node::prelude::thirdparty::tokio::runtime::Handle::try_current() {
        Ok(handle) => handle.spawn(task),
        Err(_) => DEFAULT_TOKIO_RUNTIME.spawn(task)
    }
}
```

### `#[node(runtime = "default_runtime")] impl Node`

First of all, the `Node` trait implements the following methods:

```rust
pub trait Node: Send + Sync {
    fn new(
        inputs: Inputs,
        outputs: Outputs,
        queries: Queries,
        queryables: Queryables,
        configuration: serde_yml::Value,
    ) -> tokio::task::JoinHandle<Result<Box<dyn Node>>>
    where
        Self: Sized;

    fn start(self: Box<Self>) -> tokio::task::JoinHandle<Result<()>>;
}
```

So each node must implement the `new` and `start` methods. See below for more details.
As you can see each method returns a `tokio::task::JoinHandle`, which means that the node is executed in a `tokio` task. But writing the `new` method is a bit tricky, because the `Node` trait is not `async`, so you can't use `async fn` directly. Instead, you have to use a `runtime` (the `default_runtime` for example) function to spawn the task. This is why we use the procedure `#[node(runtime = "default_runtime")]`.

It transforms your code so that:

```rust
async fn new(
    _: Inputs,
    _: Outputs,
    _: Queries,
    _: Queryables,
    _: serde_yml::Value,
) -> Result<Self> {
    Ok(Self {})
}
```

Will be transformed to that:

```rust
fn new (
    _: Inputs,
    _: Outputs,
    _: Queries,
    _: Queryables,
    _: serde_yml::Value,
) -> JoinHandle<Result<Box<dyn Node>>> {
    default_runtime(async {
        Ok(Box::new(Self {}) as Box<dyn Node>)
    })
}
```

## IOs

To better understand what are the parameters of the `new` method, we must understand the application pipeline:

- You create a `DataflowLayout` that describes the number of nodes, their names and their inputs/outputs

- You create the connections (the flows) for this layout

- You then pass this to the `runtime` that will load the nodes according to the layout and the flows.

This is when the `new` method is called: each node can retrieve the IO objects that are passed to it by the `runtime`. To retrieve it, simply use the `with` function:

```rust
Self {
    output: outputs
        .with("out")
        .await
        .wrap_err("Failed to create output")?,
}
```

Be careful, the name `out` must match the one provided inside the `DataflowLayout`. Also, the objects are typed by default. Which means that each IO has to precise the `ArrowMessage` type that it will receive/send or query:

```rust
use iridis_node::prelude::{thirdparty::*, *};

#[derive(Node)]
pub struct MySource {
    pub output: Output<String>,
}
```

See the [Messages](./message.md) section for more details about the `ArrowMessage` type.

In case of Query/Queryables, two `ArrowMessage` types must be provided, one for the request message, and one for the response message. For example:

```rust
pub compare_to_128: Queryable<u8, String>,
```

Here the request is an `u8` and the response is a `String`.

**Note:** if you don't want to manipulate typed IOs, you can use the `RawInput`, `RawOutput`, `RawQuery` and `RawQueryable` alternatives, together with the `with_raw` function. This will allow you to manipulate `ArrayData` directly with no serialization/deserialization.

## Configuration

Each node also receives a `configuration` parameter. This is a `serde_yaml::Value` object that can be used to pass configuration parameters to the node. This is useful if you want to pass some parameters to the node at runtime, so you can use the same node implementation for different configurations.

## Start

The `start` method is called once all nodes have been loaded. It consumes the node and so, when the function returns, the node is no longer available, it will be dropped. You have to take care of the loop yourself. For example, you can have a loop that sends messages every second:

```rust
async fn start(self: Box<Self>) -> Result<()> {
    while let Ok(()) = self
        .output
        .send("tick".to_string())
        .await
        .wrap_err("Failed to send message")
    {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
```

Here we use a `while let Ok(())` loop instead of just a `loop`, because when aborting the node the `send` function will eventually return an `channel closed` error. So we can use this to stop the loop and exit the node gracefully.

The `send` method takes an `ArrowMessage` as a parameter, it will then serialize it as an `ArrayData` without copy, add a `Header` and send it to the output channel. The `input` that will receives the message, will receives this `DataflowMessage` object
as a tuple containing the `Header` and the reserialized `ArrowMessage` object.

The same logic applies for `Query/Queryable`. See the [examples](./examples) section for more details.
