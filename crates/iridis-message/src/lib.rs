//! This module defines `ArrowMessage` trait and its implementation for structs and enums.

pub(crate) mod helper;
pub(crate) mod traits;

/// This prelude contains everything you need to use this crate.
pub mod prelude {
    pub use crate::helper::*;
    pub use crate::traits::*;

    pub use iridis_message_derive::*;

    pub(crate) use thirdparty::*;

    pub mod thirdparty {
        pub use arrow_array;
        pub use arrow_buffer;
        pub use arrow_data;
        pub use arrow_schema;

        pub use eyre::{self, Context, OptionExt, Result};
    }
}
