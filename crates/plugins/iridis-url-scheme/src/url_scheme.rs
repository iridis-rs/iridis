use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::prelude::{
    thirdparty::{libloading, tokio::task::JoinSet},
    *,
};

pub struct UrlSchemeManager {
    pub plugins: HashMap<String, Arc<RuntimeUrlScheme>>,
}

pub struct UrlSchemeLoader {
    pub plugins: JoinSet<Result<(Vec<String>, RuntimeUrlScheme)>>,
}

impl UrlSchemeManager {
    pub fn new(plugins: HashMap<String, Arc<RuntimeUrlScheme>>) -> Self {
        UrlSchemeManager { plugins }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn load(
        &self,
        url: Url,
        inputs: Inputs,
        outputs: Outputs,
        queries: Queries,
        queryables: Queryables,
        configuration: serde_yml::Value,
        file_ext: Arc<FileExtManager>,
    ) -> Result<RuntimeNode> {
        let scheme = url.scheme();

        let plugin = self
            .plugins
            .get(scheme)
            .ok_or_eyre(format!("Plugin not found for scheme '{}'", scheme))?;

        plugin
            .load(
                url,
                inputs,
                outputs,
                queries,
                queryables,
                configuration,
                file_ext,
            )
            .await
    }
}

impl UrlSchemeLoader {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            plugins: JoinSet::new(),
        })
    }

    pub fn load_statically_linked_plugin<T: UrlSchemePlugin + 'static>(&mut self) -> Result<()> {
        self.plugins.spawn(async move {
            let plugin = T::new().await?.wrap_err(format!(
                "Failed to load static plugin '{}'",
                std::any::type_name::<T>(),
            ))?;

            let plugin = RuntimeUrlScheme::StaticallyLinked(plugin);

            tracing::debug!(
                "Loaded statically linked plugin: {}",
                std::any::type_name::<T>()
            );

            Ok((plugin.target(), plugin))
        });

        Ok(())
    }

    pub fn load_dynamically_linked_plugin(&mut self, path: PathBuf) -> Result<()> {
        self.plugins.spawn(async move {
            match path.extension() {
                Some(ext) => {
                    if ext == std::env::consts::DLL_EXTENSION {
                        let path_buf = path.clone();
                        let (library, constructor) = tokio::task::spawn_blocking(move || {
                                let library = unsafe {
                                    #[cfg(target_family = "unix")]
                                    let library = libloading::os::unix::Library::open(
                                        Some(path_buf.clone()),
                                        libloading::os::unix::RTLD_NOW | libloading::os::unix::RTLD_GLOBAL,
                                    )
                                    .wrap_err(format!("Failed to load path {:?}", path_buf))?;

                                    #[cfg(not(target_family = "unix"))]
                                    let library = Library::new(path_buf.clone())
                                        .wrap_err(format!("Failed to load path {:?}", path_buf))?;

                                    library
                                };

                                let constructor = unsafe {
                                    library
                                    .get::<*mut DynamicallyLinkedUrlSchemePluginInstance>(
                                        b"IRIDIS_URL_SCHEME_PLUGIN",
                                    )
                                    .wrap_err(format!(
                                        "Failed to load symbol 'IRIDIS_URL_SCHEME_PLUGIN' from cdylib {:?}",
                                        path_buf
                                    ))?
                                    .read()
                                };

                                Ok::<_, eyre::Report>((library, constructor))
                            })
                            .await??;

                        let plugin = RuntimeUrlScheme::DynamicallyLinked(
                            DynamicallyLinkedUrlSchemePlugin {
                                _library: library,
                                handle: (constructor)().await?.wrap_err(format!(
                                    "Failed to load dynamically linked plugin '{:?}'",
                                    path,
                                ))?,
                            },
                        );

                        tracing::debug!(
                            "Loaded dynamically linked plugin from path: {}",
                            path.display()
                        );

                        Ok((plugin.target(), plugin))
                    } else {
                        Err(eyre::eyre!("Extension '{:?}' is not supported", ext))
                    }
                }
                _ => Err(eyre::eyre!("Unsupported path '{:?}'", path)),
            }
        });

        Ok(())
    }

    pub async fn finish(&mut self) -> Result<HashMap<String, Arc<RuntimeUrlScheme>>> {
        let mut plugins = HashMap::new();

        while let Some(result) = self.plugins.join_next().await {
            let (targets, plugin) = result??;

            let plugin = Arc::new(plugin);

            for target in targets {
                plugins.insert(target, plugin.clone());
            }
        }

        Ok(plugins)
    }
}

pub struct DynamicallyLinkedUrlSchemePlugin {
    pub handle: Box<dyn UrlSchemePlugin>,

    // Order is important !!! TODO: change to ManuallyDrop
    #[cfg(not(target_family = "unix"))]
    pub _library: libloading::Library,
    #[cfg(target_family = "unix")]
    pub _library: libloading::os::unix::Library,
}

pub enum RuntimeUrlScheme {
    StaticallyLinked(Box<dyn UrlSchemePlugin>),
    DynamicallyLinked(DynamicallyLinkedUrlSchemePlugin),
}

impl RuntimeUrlScheme {
    pub fn target(&self) -> Vec<String> {
        match self {
            RuntimeUrlScheme::StaticallyLinked(plugin) => plugin.target(),
            RuntimeUrlScheme::DynamicallyLinked(plugin) => plugin.handle.target(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn load(
        &self,
        url: Url,
        inputs: Inputs,
        outputs: Outputs,
        queries: Queries,
        queryables: Queryables,
        configuration: serde_yml::Value,
        file_ext: Arc<FileExtManager>,
    ) -> Result<RuntimeNode> {
        match self {
            RuntimeUrlScheme::StaticallyLinked(plugin) => {
                plugin
                    .load(
                        url,
                        inputs,
                        outputs,
                        queries,
                        queryables,
                        configuration,
                        file_ext,
                    )
                    .await?
            }
            RuntimeUrlScheme::DynamicallyLinked(plugin) => {
                plugin
                    .handle
                    .load(
                        url,
                        inputs,
                        outputs,
                        queries,
                        queryables,
                        configuration,
                        file_ext,
                    )
                    .await?
            }
        }
    }
}
