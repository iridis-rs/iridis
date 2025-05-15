//! This module defines the `Manager` and `Loader` associated with the `FileExtPlugin` trait.
//! It lets you load `FileExtPlugin`, store them and then load files according to their extension.

use std::{collections::HashMap, mem::ManuallyDrop, path::PathBuf, sync::Arc};

use crate::prelude::{
    thirdparty::{libloading, tokio::task::JoinSet},
    *,
};

/// Use this struct to load files according to their extension.
pub struct FileExtManager {
    pub plugins: HashMap<String, Arc<RuntimeFileExt>>,
}

/// Use this struct to load and store the plugins.
pub struct FileExtLoader {
    pub plugins: JoinSet<Result<(Vec<String>, RuntimeFileExt)>>,
}

impl FileExtManager {
    /// Create a new `FileExtManager` with the given plugins.
    pub fn new(plugins: HashMap<String, Arc<RuntimeFileExt>>) -> Self {
        Self { plugins }
    }

    /// Load a file according to its extension. It's instantiating the `Node`, and so needs all
    /// the primitives and configuration. This will `await` for the `Node` to be instantiated.
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
    /// Create a new `FileExtLoader` with an empty set of plugins.
    pub async fn new() -> Result<Self> {
        Ok(FileExtLoader {
            plugins: JoinSet::new(),
        })
    }

    /// Load a statically linked plugin by calling the `new` method of the plugin. This function
    /// is not `async`, so it will not `await` for the plugin to be loaded. It will spawn a new
    /// task to load the plugin and return immediately.
    ///
    /// Before using any loaded plugin, the `FileExtLoader::finish` method must be called to
    /// `await` for all the plugins to be loaded and return them.
    pub fn load_statically_linked_plugin<T: FileExtPlugin + 'static>(&mut self) {
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
    }

    /// Load a dynamically linked plugin by calling the `new` method of the plugin. This function
    /// is not `async`, so it will not `await` for the plugin to be loaded. It will spawn a new
    /// task to load the plugin and return immediately.
    ///
    /// Before using any loaded plugin, the `FileExtLoader::finish` method must be called to
    /// `await` for all the plugins to be loaded and return them.
    pub fn load_dynamically_linked_plugin(&mut self, path: PathBuf) {
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
                            DynamicallyLinkedFileExtPlugin::new(
                                (constructor)().await?.wrap_err(format!(
                                    "Failed to load dynamically linked plugin '{:?}'",
                                    path,
                                ))?,
                                library,
                            ),
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
    }

    /// Finish loading all the plugins. This function will `await` for all the plugins to be loaded
    /// and return them.
    pub async fn finish(mut self) -> Result<HashMap<String, Arc<RuntimeFileExt>>> {
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

/// This struct represents a dynamically linked Plugin.
/// It loads the plugin from a shared library at runtime, storing the handle as a `Box<dyn FileExtPlugin>`.
/// It's really important to store the library as well, because once the library is dropped the handle will be invalid.
///
/// While for the `Node` struct we don't care about the order of the library and the handle, because by design the node will be dropped
/// before the library, it's not the case here. And so we need either to use `ManuallyDrop` or to order the fields in a way that the library is dropped last.
pub struct DynamicallyLinkedFileExtPlugin {
    pub handle: ManuallyDrop<Box<dyn FileExtPlugin>>,

    #[cfg(not(target_family = "unix"))]
    pub library: ManuallyDrop<libloading::Library>,
    #[cfg(target_family = "unix")]
    pub library: ManuallyDrop<libloading::os::unix::Library>,
}

impl DynamicallyLinkedFileExtPlugin {
    /// Create a new `DynamicallyLinkedFileExtPlugin` with the given handle and library.
    /// Use this function to make it easier to create a new `DynamicallyLinkedFileExtPlugin`.
    pub fn new(
        handle: Box<dyn FileExtPlugin>,
        #[cfg(not(target_family = "unix"))] library: libloading::Library,
        #[cfg(target_family = "unix")] library: libloading::os::unix::Library,
    ) -> Self {
        Self {
            handle: ManuallyDrop::new(handle),
            library: ManuallyDrop::new(library),
        }
    }
}

impl Drop for DynamicallyLinkedFileExtPlugin {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.handle);
            ManuallyDrop::drop(&mut self.library);
        }
    }
}

/// This is the main enum of this module. It represents a plugin that can be either statically linked or dynamically linked,
/// allowing the runtime to use either type of plugin interchangeably.
pub enum RuntimeFileExt {
    StaticallyLinked(Box<dyn FileExtPlugin>),
    DynamicallyLinked(DynamicallyLinkedFileExtPlugin),
}

impl RuntimeFileExt {
    /// Returns the target of the plugin. This is used to determine which plugin to use for a given file.
    pub fn target(&self) -> Vec<String> {
        match self {
            RuntimeFileExt::StaticallyLinked(plugin) => plugin.target(),
            RuntimeFileExt::DynamicallyLinked(plugin) => plugin.handle.target(),
        }
    }

    /// Load a `Node` based on the `PathBuf` to the correct file. This will `await` for the `Node` to be loaded.
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
