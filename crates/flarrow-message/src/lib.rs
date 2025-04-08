pub(crate) mod helper;
pub(crate) mod traits;

pub mod prelude {
    pub use arrow_array;
    pub use arrow_data;
    pub use arrow_schema;

    pub use crate::{helper::*, traits::message::ArrowMessage};

    pub use flarrow_message_derive::ArrowMessage;

    pub type ArrowResult<T> = std::result::Result<T, arrow_schema::ArrowError>;
}
