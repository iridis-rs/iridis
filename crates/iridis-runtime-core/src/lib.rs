//! This module defines the `core` elements common to crates related to the `runtime`.

pub(crate) mod node;

/// This prelude contains everything you need to use this crate.
pub mod prelude {
    pub use crate::node::*;

    pub use iridis_node::{self, prelude::*};

    pub(crate) use thirdparty::*;

    pub mod thirdparty {
        pub use libloading;

        pub use eyre::{self, Context, OptionExt, Result};
    }
}
