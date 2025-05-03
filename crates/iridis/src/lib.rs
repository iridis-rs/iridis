pub(crate) mod loader;
pub(crate) mod runtime;

pub(crate) mod plugins;

pub mod prelude {
    pub use crate::loader::*;
    pub use crate::plugins::*;
    pub use crate::runtime::*;

    pub use iridis_api::{self, prelude::*};
    pub use iridis_builtins::{self, prelude::*};
    pub use iridis_file_ext::{self, prelude::*};
    pub use iridis_flows::{self, prelude::*};
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
