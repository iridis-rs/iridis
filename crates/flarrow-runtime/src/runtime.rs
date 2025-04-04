use std::{collections::HashMap, sync::Arc};

use crate::prelude::*;

pub struct DataflowRuntime {
    pub clock: Arc<uhlc::HLC>,
    pub nodes: HashMap<NodeID, RuntimeNode>,
}

impl DataflowRuntime {
    pub async fn new(
        flows: Flows,
        load: impl AsyncFn(&mut Loader) -> Result<()>,
    ) -> eyre::Result<Self> {
        let clock = Arc::new(uhlc::HLC::default());
        let mut loader = Loader::new(flows, clock.clone());

        load(&mut loader).await.wrap_err("Failed to load flows")?;

        Ok(Self {
            clock,
            nodes: loader.nodes,
        })
    }

    pub async fn run(self) -> Result<()> {
        let mut ids = Vec::new();
        let mut futures = Vec::new();

        for (id, node) in self.nodes {
            ids.push(id);
            futures.push(tokio::spawn(async move { node.run().await }));
        }

        let join_all = tokio::spawn(async move {
            let mut results = Vec::new();

            for future in futures {
                results.push(future.await?);
            }

            Ok::<Vec<_>, eyre::Report>(results)
        });

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            results = join_all => {
                let results = results??;
                for result in results {
                    if let Err(error) = result {
                        eprintln!("{:?}", error);
                    }
                }
            }
        }

        Ok(())
    }
}
