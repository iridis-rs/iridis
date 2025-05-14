use std::{collections::HashMap, sync::Arc};

use crate::prelude::{thirdparty::tokio::task::JoinSet, *};

/// Create a new runtime instance.
pub struct Runtime {
    pub clock: Arc<HLC>,

    pub file_ext: Arc<FileExtManager>,
    pub url_scheme: Arc<UrlSchemeManager>,

    pub nodes: HashMap<NodeID, RuntimeNode>,
}

impl Runtime {
    /// Create a new runtime instance with plugins.
    pub async fn new(
        plugins: impl AsyncFnOnce(&mut FileExtLoader, &mut UrlSchemeLoader) -> Result<()>,
    ) -> Result<Self> {
        let mut file_ext = FileExtLoader::new().await?;
        let mut url_scheme = UrlSchemeLoader::new().await?;

        file_ext.load_statically_linked_plugin::<DefaultFileExtPlugin>()?;
        url_scheme.load_statically_linked_plugin::<DefaultUrlSchemePlugin>()?;

        plugins(&mut file_ext, &mut url_scheme).await?;

        Ok(Self {
            clock: Arc::new(HLC::default()),
            file_ext: Arc::new(FileExtManager::new(file_ext.finish().await?)),
            url_scheme: Arc::new(UrlSchemeManager::new(url_scheme.finish().await?)),
            nodes: HashMap::new(),
        })
    }

    /// Load all nodes with the flows provided and run them all.
    pub async fn run(
        mut self,
        layout: Arc<DataflowLayout>,
        nodes: impl AsyncFnOnce(&mut Loader) -> Result<()>,
    ) -> Result<()> {
        let flows = RuntimeFlows::new(layout)?;

        let mut node_loader =
            Loader::new(self.file_ext, self.url_scheme, self.clock.clone(), flows);

        nodes(&mut node_loader).await?;

        self.nodes.extend(node_loader.finish().await?);

        let mut tasks = JoinSet::new();
        for (layout, node) in self.nodes {
            tasks.spawn(async move {
                node.run().await.wrap_err(format!(
                    "Node '{}' (uuid: {}) failed",
                    layout.label, layout.uuid,
                ))
            });
        }

        let join_all = tokio::spawn(async move {
            let mut reports: Vec<eyre::Report> = Vec::new();

            let mut is_ok = true;

            while let Some(result) = tasks.join_next().await {
                let result = result?;

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
