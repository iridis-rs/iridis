//! This module contains the `iridis` runtime. It can be used to
//! load and run a `DataflowLayout`

pub(crate) mod flows;
pub(crate) mod loader;
pub(crate) mod report;
pub(crate) mod runtime;

pub(crate) mod plugins;

/// This prelude contains everything you need to use this crate.
pub mod prelude {
    pub use crate::flows::*;
    pub use crate::loader::*;
    pub use crate::plugins::*;
    pub use crate::runtime::*;

    pub(crate) use crate::report::*;

    pub use iridis_builtins::{self, prelude::*};
    pub use iridis_file_ext::{self, prelude::*};
    pub use iridis_url_scheme::{self, prelude::*};

    pub(crate) use thirdparty::*;

    pub mod thirdparty {
        pub use libloading;
        pub use serde_yml;
        pub use tokio;
        pub use uhlc::HLC;
        pub use url::Url;

        pub use eyre::{self, Context, OptionExt, Result};
    }
}
