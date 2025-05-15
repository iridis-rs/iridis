//! This module defines the `Manager` and `Loader` associated with the `UrlSchemePlugin` trait.
//! It lets you load `UrlSchemePlugin`, store them and then load files according to their url.

use std::{collections::HashMap, mem::ManuallyDrop, path::PathBuf, sync::Arc};

use crate::prelude::{
    thirdparty::{libloading, tokio::task::JoinSet},
    *,
};

/// Use this struct to load files according to their url.
pub struct UrlSchemeManager {
    pub plugins: HashMap<String, Arc<RuntimeUrlScheme>>,
}

/// Use this struct to load and store the plugins.
pub struct UrlSchemeLoader {
    pub plugins: JoinSet<Result<(Vec<String>, RuntimeUrlScheme)>>,
}

impl UrlSchemeManager {
    /// Create a new `UrlSchemeManager` with the given plugins.
    pub fn new(plugins: HashMap<String, Arc<RuntimeUrlScheme>>) -> Self {
        UrlSchemeManager { plugins }
    }

    /// Load a file according to its url. It's instantiating the `Node`, and so needs all
    /// the primitives and configuration. This will `await` for the `Node` to be instantiated.
    ///
    /// If needed the `UrlSchemeManager` can fallback to the `FileExtManager` to load the file.
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
    /// Create a new `UrlSchemeLoader` with no plugins.
    pub async fn new() -> Result<Self> {
        Ok(Self {
            plugins: JoinSet::new(),
        })
    }

    /// Load a statically linked plugin by calling the `new` method of the plugin. This function
    /// is not `async`, so it will not `await` for the plugin to be loaded. It will spawn a new
    /// task to load the plugin and return immediately.
    ///
    /// Before using any loaded plugin, the `UrlSchemeLoader::finish` method must be called to
    /// `await` for all the plugins to be loaded and return them.
    pub fn load_statically_linked_plugin<T: UrlSchemePlugin + 'static>(&mut self) {
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
    }

    /// Load a dynamically linked plugin by calling the `new` method of the plugin. This function
    /// is not `async`, so it will not `await` for the plugin to be loaded. It will spawn a new
    /// task to load the plugin and return immediately.
    ///
    /// Before using any loaded plugin, the `UrlSchemeLoader::finish` method must be called to
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
                            DynamicallyLinkedUrlSchemePlugin::new(
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
    pub async fn finish(mut self) -> Result<HashMap<String, Arc<RuntimeUrlScheme>>> {
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
/// It loads the plugin from a shared library at runtime, storing the handle as a `Box<dyn UrlSchemePlugin>`.
/// It's really important to store the library as well, because once the library is dropped the handle will be invalid.
///
/// While for the `Node` struct we don't care about the order of the library and the handle, because by design the node will be dropped
/// before the library, it's not the case here. And so we need either to use `ManuallyDrop` or to order the fields in a way that the library is dropped last.
pub struct DynamicallyLinkedUrlSchemePlugin {
    pub handle: ManuallyDrop<Box<dyn UrlSchemePlugin>>,

    #[cfg(not(target_family = "unix"))]
    pub library: ManuallyDrop<libloading::Library>,
    #[cfg(target_family = "unix")]
    pub library: ManuallyDrop<libloading::os::unix::Library>,
}

impl DynamicallyLinkedUrlSchemePlugin {
    /// Create a new `DynamicallyLinkedUrlSchemePlugin` with the given handle and library.
    /// Use this function to make it easier to create a new `DynamicallyLinkedUrlSchemePlugin`.
    pub fn new(
        handle: Box<dyn UrlSchemePlugin>,
        #[cfg(not(target_family = "unix"))] library: libloading::Library,
        #[cfg(target_family = "unix")] library: libloading::os::unix::Library,
    ) -> Self {
        Self {
            handle: ManuallyDrop::new(handle),
            library: ManuallyDrop::new(library),
        }
    }
}

impl Drop for DynamicallyLinkedUrlSchemePlugin {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.handle);
            ManuallyDrop::drop(&mut self.library);
        }
    }
}

/// This is the main enum of this module. It represents a plugin that can be either statically linked or dynamically linked,
/// allowing the runtime to use either type of plugin interchangeably.
pub enum RuntimeUrlScheme {
    StaticallyLinked(Box<dyn UrlSchemePlugin>),
    DynamicallyLinked(DynamicallyLinkedUrlSchemePlugin),
}

impl RuntimeUrlScheme {
    /// Return the target of the plugin. This is the URL scheme that the plugin supports.
    pub fn target(&self) -> Vec<String> {
        match self {
            RuntimeUrlScheme::StaticallyLinked(plugin) => plugin.target(),
            RuntimeUrlScheme::DynamicallyLinked(plugin) => plugin.handle.target(),
        }
    }

    /// Load a `Node` based on the URL. This will `await` for the `Node` to be instantiated.
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
