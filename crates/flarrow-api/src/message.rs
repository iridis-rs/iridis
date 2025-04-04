use crate::prelude::*;

use uhlc::Timestamp;

#[derive(Debug, PartialEq, Clone)]
pub struct Header {
    pub timestamp: Timestamp,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DataflowMessage {
    pub header: Header,
    pub data: ArrayData,
}
