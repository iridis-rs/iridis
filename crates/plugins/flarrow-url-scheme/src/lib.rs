pub(crate) mod plugin;
pub(crate) mod url_scheme;
pub(crate) mod url_scheme_default;

pub mod prelude {
    pub use crate::plugin::*;
    pub use crate::url_scheme::*;
    pub use crate::url_scheme_default::*;

    pub(crate) use flarrow_api::prelude::*;
    pub(crate) use flarrow_file_ext::prelude::*;
    pub(crate) use flarrow_runtime_core::prelude::*;

    pub(crate) use thirdparty::*;

    pub mod thirdparty {
        pub use libloading;
        pub use serde_yml;
        pub use tokio;
        pub use url::Url;

        pub use eyre::{self, Context, OptionExt, Result};
    }
}
