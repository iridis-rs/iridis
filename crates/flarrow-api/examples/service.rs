use flarrow_api::prelude::*;

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
        let task_128 = tokio::spawn(async move {
            loop {
                if let Err(e) = compare_to_128
                    .on_demand_async(async |query| match query > 128 {
                        true => Ok("Greater than 128".to_string()),
                        false => Ok("Less than or equal to 128".to_string()),
                    })
                    .await
                {
                    return Err(e);
                }
            }
        });

        let mut compare_to_64 = self.compare_to_64;
        let task_64 = tokio::spawn(async move {
            loop {
                if let Err(e) = compare_to_64
                    .on_demand_async(async |query| match query > 64 {
                        true => Ok("Greater than 64".to_string()),
                        false => Ok("Less than or equal to 64".to_string()),
                    })
                    .await
                {
                    return Err(e);
                }
            }
        });

        task_128.await??;
        task_64.await??;

        Ok(())
    }
}
