pub(crate) mod file_ext;
pub(crate) mod plugin;

pub mod prelude {
    pub use crate::file_ext::*;
    pub use crate::plugin::*;

    pub use iridis_file_ext_derive::*;

    pub use iridis_api::{self, prelude::*};
    pub use iridis_runtime_core::{self, prelude::*};

    pub(crate) use thirdparty::*;

    pub mod thirdparty {
        pub use libloading;
        pub use serde_yml;
        pub use tokio;

        pub use eyre::{self, Context, OptionExt, Result};
    }
}
