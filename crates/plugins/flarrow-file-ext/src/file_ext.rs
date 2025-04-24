use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::prelude::{thirdparty::libloading::Library, *};

pub struct FileExtManager {
    pub plugins: HashMap<String, Arc<RuntimeFileExt>>,
}

pub struct FileExtManagerBuilder {
    pub plugins: HashMap<String, Arc<RuntimeFileExt>>,
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

impl FileExtManagerBuilder {
    pub async fn new() -> Result<Self> {
        let mut manager = FileExtManagerBuilder {
            plugins: HashMap::new(),
        };

        manager
            .load_statically_linked_plugin::<DefaultFileExtPlugin>()
            .await?;

        Ok(manager)
    }

    pub async fn load_statically_linked_plugin<T: FileExtPlugin + 'static>(
        &mut self,
    ) -> Result<()> {
        let plugin = T::new().await?.wrap_err(format!(
            "Failed to load static plugin '{}'",
            std::any::type_name::<T>(),
        ))?;

        let plugin = Arc::new(RuntimeFileExt::StaticallyLinked(plugin));

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
                                .get::<*mut DynamicallyLinkedFileExtPluginInstance>(
                                    b"FLARROW_FILE_EXT_PLUGIN",
                                )
                                .wrap_err(format!(
                                    "Failed to load symbol 'FLARROW_FILE_EXT_PLUGIN' from cdylib {:?}",
                                    path_buf
                                ))?
                                .read()
                            };

                            Ok::<_, eyre::Report>((library, constructor))
                        })
                        .await??;

                    let plugin = Arc::new(RuntimeFileExt::DynamicallyLinked(
                        DynamicallyLinkedFileExtPlugin {
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

pub struct DynamicallyLinkedFileExtPlugin {
    pub handle: Box<dyn FileExtPlugin>,
    pub _library: Library, // Order is important !!! TODO: change to ManuallyDrop
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
