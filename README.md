# `iridis`

`iridis` is a framework that lets you define and build dataflow applications with ease.

It consists on 2 APIs:

- `iridis-api`: the main API for implementing each node in the dataflow graph.
- `iridis`: the `runtime` API that will load all the nodes and launch the application.

Additionally we provide 2 other APIs:

- `iridis-file-ext`: the Plugin API for handling file extensions (it defines how the runtime should load a file with a specific extension).
- `iridis-url-scheme`: the Plugin API for handling URL schemes (it defines how the runtime should load an url with a specific URL scheme).

Each plugin can be loaded into the `iridis` runtime uppon initialization.

## Usage

In a `lib` crate we can define a `node`:

```rust
use iridis_api::prelude::{thirdparty::*, *};

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

Then we can create a `layout` and the `flows` that apply to the application:

```rust
use iridis::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
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

And finally we can create a Runtime, load all our plugins, and for each node in the layout, we can load a `node implementation`:

```rust
let runtime = Runtime::new(
    async |_file_ext: &mut FileExtManagerBuilder, _url_scheme: &mut UrlSchemeManagerBuilder| {
        Ok(())
    },
)
.await?;

runtime
    .run(flows, async move |loader: &mut NodeLoader| {
        loader
            .load::<Timer>(source, serde_yml::from_str("frequency: 1.0")?)
            .await?;

        loader
            .load::<Transport>(operator, serde_yml::from_str("")?)
            .await?;

        loader
            .load::<Printer>(sink, serde_yml::from_str("")?)
            .await?;

        Ok(())
    })
    .await
}
```

In this example we load 3 nodes as statically linked libraries. But it's possible to load a node dynamically from an URL. The node must have been
compiled as a `cdylib` with the feature flag `cdylib` enabled.

```rust
loader.load_url(Url::parse("file:///path/to/timer.so")?, source, serde_yml::from_str("frequency: 1.0")?)
    .await?;
```

See [iridis-benchmark](https://github.com/iridis-rs/iridis-benchmark) for a complete example of a project with multiple nodes, both statically linked and dynamically loaded.

## Benchmark

See [iridis-benchmark](https://github.com/iridis-rs/iridis-benchmark) for a full description of the benchmark.

<div align="center">
  <img src="https://raw.githubusercontent.com/iridis-rs/iridis-benchmark/main/bench/benchmark_latency.svg" alt="Benchmark Latency">
</div>
