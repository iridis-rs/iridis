# Examples

See the [example folder](https://github.com/iridis-rs/iridis/tree/main/crates/iridis-examples/examples)

## Messages

```rust
use iridis_message::prelude::{
    thirdparty::{arrow_array::*, arrow_data::*, *},
    *,
};

#[derive(Debug, ArrowMessage)]
enum Encoding {
    RGB8,
    RGBA8,
    BGR8,
    BGRA8,
}

#[derive(Debug, ArrowMessage)]
struct Metadata {
    name: Option<String>,
    width: u32,
    height: u32,
    encoding: Option<Encoding>,
}

#[derive(Debug, ArrowMessage)]
struct Image {
    data: UInt8Array,
    metadata: Option<Metadata>,
}
```

## Node

```rust
use iridis_node::prelude::{thirdparty::*, *};

#[derive(Node)]
pub struct MyClient {
    pub ask_128: Query<u8, String>,
    pub ask_64: Query<u8, String>,
}

#[node(runtime = "default_runtime")]
impl Node for MyClient {
    async fn new(
        _: Inputs,
        _: Outputs,
        mut queries: Queries,
        _: Queryables,
        _: serde_yml::Value,
    ) -> Result<Self> {
        Ok(Self {
            ask_128: queries
                .with("ask_128")
                .await
                .wrap_err("Failed to create ask_128 queryable")?,
            ask_64: queries
                .with("ask_64")
                .await
                .wrap_err("Failed to create compare_to_64 queryable")?,
        })
    }

    async fn start(mut self: Box<Self>) -> Result<()> {
        let (_, answer) = self
            .ask_128
            .query(100)
            .await
            .wrap_err("Failed to query ask_128")?;

        println!("Answer: {}", answer);

        let (_, answer) = self
            .ask_128
            .query(200)
            .await
            .wrap_err("Failed to query ask_128")?;

        println!("Answer: {}", answer);

        let (_, answer) = self
            .ask_64
            .query(100)
            .await
            .wrap_err("Failed to query ask_64")?;

        println!("Answer: {}", answer);

        let (_, answer) = self
            .ask_64
            .query(2)
            .await
            .wrap_err("Failed to query ask_64")?;

        println!("Answer: {}", answer);

        Ok(())
    }
}

use iridis_node::prelude::{thirdparty::*, *};

#[derive(Node)]
pub struct MyService {
    pub compare_to_128: Queryable<u8, String>,
    pub compare_to_64: Queryable<u8, String>,
}

#[node(runtime = "default_runtime")]
impl Node for MyService {
    async fn new(
        _: Inputs,
        _: Outputs,
        _: Queries,
        mut queryables: Queryables,
        _: serde_yml::Value,
    ) -> Result<Self> {
        Ok(Self {
            compare_to_128: queryables
                .with("compare_to_128")
                .await
                .wrap_err("Failed to create compare_to_128 queryable")?,
            compare_to_64: queryables
                .with("compare_to_64")
                .await
                .wrap_err("Failed to create compare_to_64 queryable")?,
        })
    }

    async fn start(self: Box<Self>) -> Result<()> {
        let mut compare_to_128 = self.compare_to_128;
        let task_128: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
            while let Ok(()) = compare_to_128
                .on_query(async |query| match query > 128 {
                    true => Ok(format!("{} is greater than 128", query).to_string()),
                    false => Ok(format!("{} is less than or equal to 128", query).to_string()),
                })
                .await
            {}

            Ok(())
        });

        let mut compare_to_64 = self.compare_to_64;
        let task_64: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
            while let Ok(()) = compare_to_64
                .on_query(async |query| match query > 64 {
                    true => Ok(format!("{} is greater than 64", query).to_string()),
                    false => Ok(format!("{} is less than or equal to 64", query).to_string()),
                })
                .await
            {}

            Ok(())
        });

        task_128.await??;
        task_64.await??;

        Ok(())
    }
}

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

use std::time::Duration;

use iridis_node::prelude::{thirdparty::*, *};

#[derive(Node)]
pub struct MySource {
    pub output: Output<String>,
}

#[node(runtime = "default_runtime")]
impl Node for MySource {
    async fn new(
        _: Inputs,
        mut outputs: Outputs,
        _: Queries,
        _: Queryables,
        _: serde_yml::Value,
    ) -> Result<Self> {
        Ok(Self {
            output: outputs
                .with("out")
                .await
                .wrap_err("Failed to create output")?,
        })
    }

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
}
```

## Runtime

```rust
use iridis::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut layout = DataflowLayout::new();

    let (source, output) = layout
        .node("source", async |builder: &mut Builder| {
            builder.output("out")
        })
        .await;

    let (operator, (op_in, op_out)) = layout
        .node("operator", async |builder: &mut Builder| {
            (builder.input("in"), builder.output("out"))
        })
        .await;

    let (sink, input) = layout
        .node("sink", async |builder: &mut Builder| builder.input("in"))
        .await;

    let layout = layout.build();

    let flows = Flows::new(layout.clone(), async move |flows: &mut Connector| {
        flows.connect(op_in, output, None)?;
        flows.connect(input, op_out, None)?;

        Ok(())
    })
    .await?;

    let runtime = Runtime::new(
        async |_file_ext: &mut FileExtLoader, _url_scheme: &mut UrlSchemeLoader| Ok(()),
    )
    .await?;

    runtime
        .run(flows, async move |loader: &mut Loader| {
            loader
                .load_url(
                    iridis_examples::dylib("source", None)?,
                    source,
                    serde_yml::from_str("")?,
                )
                .await?;

            loader
                .load::<Transport>(operator, serde_yml::from_str("")?)
                .await?;

            loader
                .load_url(
                    iridis_examples::dylib("sink", None)?,
                    sink,
                    serde_yml::from_str("")?,
                )
                .await?;

            Ok(())
        })
        .await
}

use iridis::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut layout = DataflowLayout::new();

    let (service, (compare_to_128, compare_to_64)) = layout
        .node("service", async |builder: &mut Builder| {
            (
                builder.queryable("compare_to_128"),
                builder.queryable("compare_to_64"),
            )
        })
        .await;

    let (client, (ask_128, ask_64)) = layout
        .node("client", async |builder: &mut Builder| {
            (builder.query("ask_128"), builder.query("ask_64"))
        })
        .await;

    let layout = layout.build();

    let flows = Flows::new(layout.clone(), async move |flows: &mut Connector| {
        flows.connect(ask_128, compare_to_128, None)?;
        flows.connect(ask_64, compare_to_64, None)?;

        Ok(())
    })
    .await?;

    let runtime = Runtime::new(
        async |_file_ext: &mut FileExtLoader, _url_scheme: &mut UrlSchemeLoader| Ok(()),
    )
    .await?;

    runtime
        .run(flows, async move |loader: &mut Loader| {
            loader
                .load_url(
                    iridis_examples::dylib("service", None)?,
                    service,
                    serde_yml::from_str("")?,
                )
                .await?;

            loader
                .load_url(
                    iridis_examples::dylib("client", None)?,
                    client,
                    serde_yml::from_str("")?,
                )
                .await?;

            Ok(())
        })
        .await
}
```
