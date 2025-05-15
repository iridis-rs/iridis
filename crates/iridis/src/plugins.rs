//! This module contains the two default plugins for the `iridis` runtime.

pub(crate) mod file_ext_default;
pub(crate) mod url_scheme_default;

pub use file_ext_default::*;
pub use url_scheme_default::*;
