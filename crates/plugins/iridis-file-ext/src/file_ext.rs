use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::prelude::{
    thirdparty::{libloading, tokio::task::JoinSet},
    *,
};

pub struct FileExtManager {
    pub plugins: HashMap<String, Arc<RuntimeFileExt>>,
}

pub struct FileExtLoader {
    pub plugins: JoinSet<Result<(Vec<String>, RuntimeFileExt)>>,
}

impl FileExtManager {
    pub fn new(plugins: HashMap<String, Arc<RuntimeFileExt>>) -> Self {
        Self { plugins }
    }

    pub async fn load(
        &self,
        path: PathBuf,
        inputs: Inputs,
        outputs: Outputs,
        queries: Queries,
        queryables: Queryables,
        configuration: serde_yml::Value,
    ) -> Result<RuntimeNode> {
        let ext = path
            .extension()
            .ok_or_eyre(format!("No extension found for path '{:?}'", path))?
            .to_str()
            .ok_or_eyre("Invalid extension")?;

        let plugin = self
            .plugins
            .get(ext)
            .ok_or_eyre(format!("Plugin not found for extension '{}'", ext))?;

        plugin
            .load(path, inputs, outputs, queries, queryables, configuration)
            .await
    }
}

impl FileExtLoader {
    pub async fn new() -> Result<Self> {
        Ok(FileExtLoader {
            plugins: JoinSet::new(),
        })
    }

    pub fn load_statically_linked_plugin<T: FileExtPlugin + 'static>(&mut self) -> Result<()> {
        self.plugins.spawn(async move {
            let plugin = T::new().await?.wrap_err(format!(
                "Failed to load static plugin '{}'",
                std::any::type_name::<T>(),
            ))?;

            let plugin = RuntimeFileExt::StaticallyLinked(plugin);

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
                                    .get::<*mut DynamicallyLinkedFileExtPluginInstance>(
                                        b"IRIDIS_FILE_EXT_PLUGIN",
                                    )
                                    .wrap_err(format!(
                                        "Failed to load symbol 'IRIDIS_FILE_EXT_PLUGIN' from cdylib {:?}",
                                        path_buf
                                    ))?
                                    .read()
                            };

                            Ok::<_, eyre::Report>((library, constructor))
                        })
                        .await??;

                        let plugin = RuntimeFileExt::DynamicallyLinked(
                            DynamicallyLinkedFileExtPlugin {
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

    pub async fn finish(&mut self) -> Result<HashMap<String, Arc<RuntimeFileExt>>> {
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

pub struct DynamicallyLinkedFileExtPlugin {
    pub handle: Box<dyn FileExtPlugin>,

    // Order is important !!! TODO: change to ManuallyDrop
    #[cfg(not(target_family = "unix"))]
    pub _library: libloading::Library,
    #[cfg(target_family = "unix")]
    pub _library: libloading::os::unix::Library,
}

pub enum RuntimeFileExt {
    StaticallyLinked(Box<dyn FileExtPlugin>),
    DynamicallyLinked(DynamicallyLinkedFileExtPlugin),
}

impl RuntimeFileExt {
    pub fn target(&self) -> Vec<String> {
        match self {
            RuntimeFileExt::StaticallyLinked(plugin) => plugin.target(),
            RuntimeFileExt::DynamicallyLinked(plugin) => plugin.handle.target(),
        }
    }

    pub async fn load(
        &self,
        path: PathBuf,
        inputs: Inputs,
        outputs: Outputs,
        queries: Queries,
        queryables: Queryables,
        configuration: serde_yml::Value,
    ) -> Result<RuntimeNode> {
        match self {
            RuntimeFileExt::StaticallyLinked(plugin) => {
                plugin
                    .load(path, inputs, outputs, queries, queryables, configuration)
                    .await?
            }
            RuntimeFileExt::DynamicallyLinked(plugin) => {
                plugin
                    .handle
                    .load(path, inputs, outputs, queries, queryables, configuration)
                    .await?
            }
        }
    }
}
