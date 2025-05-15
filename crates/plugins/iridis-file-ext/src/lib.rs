//! This module defines the `FileExtPlugin` trait that must be implemented
//! in order to make a plugin compatible with the `iridis_file_ext` crate.
//! It also defines the `Manager` struct, which is used to manage a set of plugins.

pub(crate) mod file_ext;
pub(crate) mod plugin;

/// This prelude contains everything you need to use this crate.
pub mod prelude {
    pub use crate::file_ext::*;
    pub use crate::plugin::*;

    pub use iridis_file_ext_derive::*;

    pub use iridis_node::{self, prelude::*};
    pub use iridis_runtime_core::{self, prelude::*};

    pub(crate) use thirdparty::*;

    pub mod thirdparty {
        pub use libloading;
        pub use serde_yml;
        pub use tokio;

        pub use eyre::{self, Context, OptionExt, Result};
    }
}
