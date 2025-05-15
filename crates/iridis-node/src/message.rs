//! This module defines a message type for the dataflow communication

use arrow_data::ArrayData;
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

use crate::prelude::*;

use uhlc::Timestamp;

/// Header for a dataflow message
#[derive(Debug, PartialEq, Clone)]
pub struct Header {
    /// Timestamp of the message, representing when the message was created by the runtime (sender side)
    pub timestamp: Timestamp,

    /// Identifier of the message, representing the source node uuid and the IO it's coming from (output, query or queryable)
    pub source: (Uuid, Uuid),
}

/// Dataflow message. Cheap to clone
#[derive(Debug, PartialEq, Clone)]
pub struct DataflowMessage {
    pub header: Header,
    pub data: ArrayData,
}

/// MPSC Message sender. Can be cloned, cheap to clone
pub type MessageSender = Sender<DataflowMessage>;

/// MPSC Message receiver. Cannot be cloned
pub type MessageReceiver = Receiver<DataflowMessage>;

/// Typed dataflow message
#[derive(Debug, PartialEq, Clone)]
pub struct TypedDataflowMessage<T: ArrowMessage> {
    pub header: Header,
    pub data: T,
}

impl<T> TryFrom<DataflowMessage> for TypedDataflowMessage<T>
where
    T: ArrowMessage,
{
    type Error = eyre::Report;

    fn try_from(value: DataflowMessage) -> Result<Self> {
        let data = T::try_from_arrow(value.data)?;

        Ok(Self {
            header: value.header,
            data,
        })
    }
}
