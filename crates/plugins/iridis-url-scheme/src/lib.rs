//! This module defines the `UrlSchemePlugin` trait that must be implemented
//! in order to make a plugin compatible with the `iridis_url_scheme` crate.
//! It also defines the `Manager` struct, which is used to manage a set of plugins.

pub(crate) mod plugin;
pub(crate) mod url_scheme;

/// This prelude contains everything you need to use this crate.
pub mod prelude {
    pub use crate::plugin::*;
    pub use crate::url_scheme::*;

    pub use iridis_url_scheme_derive::*;

    pub use iridis_file_ext::{self, prelude::*};

    pub(crate) use thirdparty::*;

    pub mod thirdparty {
        pub use libloading;
        pub use serde_yml;
        pub use tokio;
        pub use url::Url;

        pub use eyre::{self, Context, OptionExt, Result};
    }
}
