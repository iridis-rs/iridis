use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::prelude::{thirdparty::libloading::Library, *};

pub struct UrlSchemeManager {
    pub plugins: HashMap<String, Arc<RuntimeUrlScheme>>,
}

pub struct UrlSchemeManagerBuilder {
    pub plugins: HashMap<String, Arc<RuntimeUrlScheme>>,
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

impl UrlSchemeManagerBuilder {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            plugins: HashMap::new(),
        })
    }

    pub async fn load_statically_linked_plugin<T: UrlSchemePlugin + 'static>(
        &mut self,
    ) -> Result<()> {
        let plugin = T::new().await?.wrap_err(format!(
            "Failed to load static plugin '{}'",
            std::any::type_name::<T>(),
        ))?;

        let plugin = Arc::new(RuntimeUrlScheme::StaticallyLinked(plugin));

        for ext in &plugin.target() {
            self.plugins.insert(ext.to_string(), plugin.clone());
        }

        tracing::debug!(
            "Loaded statically linked plugin: {}",
            std::any::type_name::<T>()
        );

        Ok(())
    }

    pub async fn load_dynamically_linked_plugin(&mut self, path: PathBuf) -> Result<()> {
        match path.extension() {
            Some(ext) => {
                if ext == std::env::consts::DLL_EXTENSION {
                    let path_buf = path.clone();
                    let (library, constructor) = tokio::task::spawn_blocking(move || {
                            let library = unsafe {
                                Library::new(path_buf.clone())
                                    .wrap_err(format!("Failed to load path {:?}", path_buf))?
                            };

                            let constructor = unsafe {
                                library
                                .get::<*mut DynamicallyLinkedUrlSchemePluginInstance>(
                                    b"FLARROW_URL_SCHEME_PLUGIN",
                                )
                                .wrap_err(format!(
                                    "Failed to load symbol 'FLARROW_URL_SCHEME_PLUGIN' from cdylib {:?}",
                                    path_buf
                                ))?
                                .read()
                            };

                            Ok::<_, eyre::Report>((library, constructor))
                        })
                        .await??;

                    let plugin = Arc::new(RuntimeUrlScheme::DynamicallyLinked(
                        DynamicallyLinkedUrlSchemePlugin {
                            _library: library,
                            handle: (constructor)().await?.wrap_err(format!(
                                "Failed to load dynamically linked plugin '{:?}'",
                                path,
                            ))?,
                        },
                    ));

                    for ext in &plugin.target() {
                        self.plugins.insert(ext.to_string(), plugin.clone());
                    }

                    tracing::debug!(
                        "Loaded dynamically linked plugin from path: {}",
                        path.display()
                    );

                    Ok(())
                } else {
                    Err(eyre::eyre!("Extension '{:?}' is not supported", ext))
                }
            }
            _ => Err(eyre::eyre!("Unsupported path '{:?}'", path)),
        }
    }
}

pub struct DynamicallyLinkedUrlSchemePlugin {
    pub handle: Box<dyn UrlSchemePlugin>,
    pub _library: Library, // Order is important !!! TODO: change to ManuallyDrop
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
