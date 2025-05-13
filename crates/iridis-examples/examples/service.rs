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
                .on_query(async |query| match query.data > 128 {
                    true => Ok(format!("{} is greater than 128", query.data).to_string()),
                    false => Ok(format!("{} is less than or equal to 128", query.data).to_string()),
                })
                .await
            {}

            Ok(())
        });

        let mut compare_to_64 = self.compare_to_64;
        let task_64: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
            while let Ok(()) = compare_to_64
                .on_query(async |query| match query.data > 64 {
                    true => Ok(format!("{} is greater than 64", query.data).to_string()),
                    false => Ok(format!("{} is less than or equal to 64", query.data).to_string()),
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
