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
        let TypedDataflowMessage {
            header: _,
            data: answer,
        } = self
            .ask_128
            .query(100)
            .await
            .wrap_err("Failed to query ask_128")?;

        println!("Answer: {}", answer);

        let TypedDataflowMessage {
            header: _,
            data: answer,
        } = self
            .ask_128
            .query(200)
            .await
            .wrap_err("Failed to query ask_128")?;

        println!("Answer: {}", answer);

        let TypedDataflowMessage {
            header: _,
            data: answer,
        } = self
            .ask_64
            .query(100)
            .await
            .wrap_err("Failed to query ask_64")?;

        println!("Answer: {}", answer);

        let TypedDataflowMessage {
            header: _,
            data: answer,
        } = self
            .ask_64
            .query(2)
            .await
            .wrap_err("Failed to query ask_64")?;

        println!("Answer: {}", answer);

        Ok(())
    }
}
