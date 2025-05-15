//! This module defines the layout elements of a `dataflow` application.
//! It can be viewed as the logical graph structure of the application.
//!
//! It's really important to be able to model the application as a graph
//! using this crate, before intending to use the `runtime` to run it.

pub(crate) mod flows;
pub(crate) mod layout;
pub(crate) mod node;
pub(crate) mod primitives;

/// This prelude contains everything you need to use this crate.
pub mod prelude {
    pub use crate::flows::*;
    pub use crate::layout::*;
    pub use crate::node::*;
    pub use crate::primitives::*;

    pub(crate) use thirdparty::*;

    pub mod thirdparty {
        pub use tokio;
        pub use uuid::Uuid;

        pub use eyre::{self, Context, OptionExt, Result};
    }
}
