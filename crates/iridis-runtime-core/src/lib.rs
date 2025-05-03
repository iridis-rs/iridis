pub(crate) mod node;

pub mod prelude {
    pub use crate::node::*;

    pub use iridis_api::{self, prelude::*};

    pub(crate) use thirdparty::*;

    pub mod thirdparty {
        pub use libloading;

        pub use eyre::{self, Context, OptionExt, Result};
    }
}
