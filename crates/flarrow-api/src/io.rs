pub(crate) mod inputs;
pub(crate) mod outputs;
pub(crate) mod queries;
pub(crate) mod queryables;

use std::{collections::HashMap, sync::Arc};

pub use inputs::*;
pub use outputs::*;
pub use queries::*;
pub use queryables::*;
use tokio::sync::Mutex;

pub type SharedMap<T, F> = Arc<Mutex<HashMap<T, F>>>;
