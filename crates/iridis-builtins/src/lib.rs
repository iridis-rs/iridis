//! This module contains some builtins nodes that can be used directly
//! by the user.

pub(crate) mod enumeration;

pub(crate) mod printer;
pub(crate) mod timer;
pub(crate) mod transport;

/// This prelude contains everything you need to use this crate.
pub mod prelude {
    pub use crate::enumeration::*;
    pub use crate::printer::*;
    pub use crate::timer::*;
    pub use crate::transport::*;

    pub use iridis_node::{self, prelude::*};

    pub(crate) use thirdparty::*;

    pub mod thirdparty {
        pub use serde_yml;

        pub use eyre::{self, Context, OptionExt, Result};
    }
}
