//! This module contains everything to write an `iridis` node.

pub(crate) mod message;
pub(crate) mod node;
pub(crate) mod primitives;
pub(crate) mod report;

/// This prelude contains everything you need to use this crate.
pub mod prelude {
    pub use crate::message::*;
    pub use crate::node::*;
    pub use crate::primitives::*;

    pub use iridis_node_derive::*;

    pub(crate) use crate::report::*;

    pub use iridis_layout::{self, prelude::*};
    pub use iridis_message::{self, prelude::*};

    pub(crate) use thirdparty::*;

    pub mod thirdparty {
        pub use arrow_array;
        pub use arrow_data;
        pub use serde_yml;
        pub use tokio;
        pub use uhlc::{self, HLC};
        pub use uuid::Uuid;

        pub use eyre::{self, Context, OptionExt, Result};
    }
}
