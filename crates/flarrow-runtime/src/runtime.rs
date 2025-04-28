use std::{collections::HashMap, sync::Arc};

use crate::prelude::*;

/// Create a new runtime instance.
pub struct Runtime {
    pub clock: Arc<HLC>,

    pub file_ext: Arc<FileExtManager>,
    pub url_scheme: Arc<UrlSchemeManager>,

    pub nodes: HashMap<NodeLayout, RuntimeNode>,
}

impl Runtime {
    /// Create a new runtime instance with plugins.
    pub async fn new(
        plugins: impl AsyncFnOnce(
            &mut FileExtManagerBuilder,
            &mut UrlSchemeManagerBuilder,
        ) -> Result<()>,
    ) -> Result<Self> {
        let mut file_ext = FileExtManagerBuilder::new().await?;
        let mut url_scheme = UrlSchemeManagerBuilder::new().await?;

        plugins(&mut file_ext, &mut url_scheme).await?;

        Ok(Self {
            clock: Arc::new(HLC::default()),
            file_ext: Arc::new(FileExtManager::new(file_ext.plugins)),
            url_scheme: Arc::new(UrlSchemeManager::new(url_scheme.plugins)),
            nodes: HashMap::new(),
        })
    }

    /// Load all nodes with the flows provided and run them all.
    pub async fn run(
        mut self,
        flows: Flows,
        nodes: impl AsyncFnOnce(&mut NodeLoader) -> Result<()>,
    ) -> Result<()> {
        let mut node_loader =
            NodeLoader::new(self.file_ext, self.url_scheme, self.clock.clone(), flows);

        nodes(&mut node_loader).await?;

        self.nodes.extend(node_loader.nodes);

        println!("Starting runtime... (press Ctrl+C to stop)");

        let mut tasks = Vec::new();
        for (layout, node) in self.nodes {
            tasks.push(tokio::spawn(async move {
                node.run().await.wrap_err(format!(
                    "Node '{}' (uuid: {}) failed",
                    layout.label, layout.uuid,
                ))
            }));
        }

        let join_all = tokio::spawn(async move {
            let mut reports: Vec<eyre::Report> = Vec::new();

            let mut is_ok = true;

            for task in tasks {
                let result = task.await?;

                if let Err(report) = result {
                    is_ok = false;
                    reports.push(report);
                }
            }

            let reports: eyre::Report = {
                let report_str: String = reports.iter().fold(
                    "The runtime encountered multiple errors:".to_string(),
                    |acc, report| format!("{}\n\n{:?}", acc, report),
                );

                eyre::Report::msg(report_str)
            };

            match is_ok {
                true => Ok(()),
                false => Err(reports),
            }
        });

        tokio::select! {
            _ = tokio::signal::ctrl_c() => { Ok(()) },
            results = join_all => { results? }
        }
    }
}
