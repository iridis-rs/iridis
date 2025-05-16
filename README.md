# `iridis`

`iridis` is a framework that allows you to define and build dataflow applications with ease.

It consists of two main APIs:

* `iridis-node`: the primary API used to implement each node in the dataflow graph.
* `iridis`: the `runtime` API responsible for loading all nodes and launching the application.

In addition, we provide two plugin APIs:

* `iridis-file-ext`: the plugin API for handling file extensions. It defines how the runtime should load files with specific extensions.
* `iridis-url-scheme`: the plugin API for handling URL schemes. It defines how the runtime should load URLs with specific schemes.

Each plugin can be loaded into the `iridis` runtime upon initialization.

See [*the official wiki*](https://iridis-rs.github.io/iridis) for more information.

## Usage

In a `lib` crate, you can define a `node` like this:

```rust
use iridis_node::prelude::{thirdparty::*, *};

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

Next, create a `layout` and define the `flows` for the application:

```rust
use iridis::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    let layout = DataflowLayout::empty();

    let (source, output) = layout
        .node("source", async |builder: &mut NodeLayout| {
            builder.output("out")
        })
        .await;

    let (operator, (op_in, op_out)) = layout
        .node("operator", async |builder: &mut NodeLayout| {
            (builder.input("in"), builder.output("out"))
        })
        .await;

    let (sink, input) = layout
        .node("sink", async |builder: &mut NodeLayout| builder.input("in"))
        .await;

    let layout = layout
        .finish(async |flows| {
            flows.connect(op_in, output)?;
            flows.connect(input, op_out)?;

            Ok(())
        })
        .await?;

    Ok(())
}
```

Finally, create a runtime, load your plugins, and load a `node implementation` for each node in the layout:

```rust
let runtime = Runtime::new(
    async |_file_ext: &mut FileExtLoader, _url_scheme: &mut UrlSchemeLoader| {
        Ok(())
    },
)
.await?;

runtime
    .run(layout, async move |loader: &mut Loader| {
        loader
            .load::<Timer>(source, serde_yml::from_str("frequency: 1.0")?)?;

        loader
            .load::<Transport>(operator, serde_yml::from_str("")?)?;

        loader
            .load::<Printer>(sink, serde_yml::from_str("")?)?;

        Ok(())
    })
    .await
}
```

In this example, three nodes are loaded as statically linked libraries. However, it’s also possible to load a node dynamically from a URL. The node must be compiled as a `cdylib` with the `cdylib` feature flag enabled:

```rust
loader.load_url(Url::parse("file:///path/to/timer.so")?, source, serde_yml::from_str("frequency: 1.0")?)?;
```

For a complete example of a project with multiple nodes—both statically linked and dynamically loaded—see [iridis-benchmark](https://github.com/iridis-rs/iridis-benchmark).

## Examples

Multiple examples can be found in [this directory](crates/iridis-examples) and can be launched with `just`:

### Example of full layouts, ready to be put inside the runtime

```bash
just io_layout
just service_layout
```

### Example of message definitions

```bash
just message_complex
just message_derive
just message_enum_derive
just message_enum_impl
just message_impl
```

### Example of nodes implementation

```bash
just sink
just source
just client
just service
```

### Example of applications

```bash
just io_runtime
just service_runtime
```

## Python

It's possible to write your nodes in python. You will need to add the `PythonFileExt` plugin into your runtime to be able to load a `.py` file. See
[pyridis](https://github.com/iridis-rs/pyridis) for a detailed description of the python support.

## Benchmark

See [iridis-benchmark](https://github.com/iridis-rs/iridis-benchmark) for a detailed description of the benchmark.

<div align="center">
  <img src="https://raw.githubusercontent.com/iridis-rs/iridis-benchmark/main/bench/benchmark_latency.svg" alt="Benchmark Latency">
</div>
