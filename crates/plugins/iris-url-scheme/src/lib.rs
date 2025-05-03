pub(crate) mod plugin;
pub(crate) mod url_scheme;

pub mod prelude {
    pub use crate::plugin::*;
    pub use crate::url_scheme::*;

    pub use iris_url_scheme_derive::*;

    pub use iris_api::{self, prelude::*};
    pub use iris_file_ext::{self, prelude::*};

    pub(crate) use thirdparty::*;

    pub mod thirdparty {
        pub use libloading;
        pub use serde_yml;
        pub use tokio;
        pub use url::Url;

        pub use eyre::{self, Context, OptionExt, Result};
    }
}
