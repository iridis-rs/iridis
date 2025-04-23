use arrow_data::ArrayData;

use crate::prelude::*;

use uhlc::Timestamp;

#[derive(Debug, PartialEq, Clone)]
pub enum Source {
    Output(OutputUUID),
    Query(QueryUUID),
    Queryable(QueryableUUID),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Header {
    pub timestamp: Timestamp,

    pub node: Option<NodeUUID>,
    pub source: Option<Source>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DataflowMessage {
    pub header: Header,
    pub data: ArrayData,
}
