use std::sync::Arc;

use flarrow_runtime::prelude::*;
use url::Url;

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
    ) -> Result<Box<dyn Node>>
    where
        Self: Sized,
    {
        Ok(Box::new(Self {
            ask_128: queries
                .with("ask_128")
                .await
                .wrap_err("Failed to create ask_128 queryable")?,
            ask_64: queries
                .with("ask_64")
                .await
                .wrap_err("Failed to create compare_to_64 queryable")?,
        }) as Box<dyn Node>)
    }

    async fn start(mut self: Box<Self>) -> Result<()> {
        let (_, answer) = self
            .ask_128
            .query_async(100)
            .await
            .wrap_err("Failed to query ask_128")?;

        println!("Answer: {}", answer);

        let (_, answer) = self
            .ask_128
            .query_async(200)
            .await
            .wrap_err("Failed to query ask_128")?;

        println!("Answer: {}", answer);

        let (_, answer) = self
            .ask_64
            .query_async(100)
            .await
            .wrap_err("Failed to query ask_64")?;

        println!("Answer: {}", answer);

        let (_, answer) = self
            .ask_64
            .query_async(2)
            .await
            .wrap_err("Failed to query ask_64")?;

        println!("Answer: {}", answer);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut layout = DataflowLayout::new();

    let (service, (compare_to_128, compare_to_64)) = layout
        .create_node(async |io: &mut NodeIO| {
            (
                io.open_queryable("compare_to_128"),
                io.open_queryable("compare_to_64"),
            )
        })
        .await;

    let (client, (ask_128, ask_64)) = layout
        .create_node(async |io: &mut NodeIO| (io.open_query("ask_128"), io.open_query("ask_64")))
        .await;

    let layout = Arc::new(layout);
    let flows = Flows::new(layout.clone(), async move |connector: &mut Connector| {
        connector.service(ask_128, compare_to_128, None)?;
        connector.service(ask_64, compare_to_64, None)?;

        Ok(())
    })
    .await?;

    let path = std::env::var("CARGO_MANIFEST_DIR")?;
    let examples = format!("file://{}/../../target/debug/examples", path);

    let runtime = DataflowRuntime::new(flows, None, async move |loader: &mut Loader| {
        loader
            .load_statically_linked::<MyClient>(client, serde_yml::Value::from(""))
            .await
            .wrap_err("Failed to load MyClient")?;

        let service_file = Url::parse(&format!("{}/libservice.so", examples))?;

        loader
            .load_from_url(service, service_file, serde_yml::Value::from(""))
            .await
            .wrap_err("Failed to load service")?;

        Ok(())
    })
    .await?;

    runtime.run().await
}
