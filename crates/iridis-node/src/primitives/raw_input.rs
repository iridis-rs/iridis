//! This module contains implementations for this primitive.

use crate::prelude::*;

/// Not typed Input to receive data from the dataflow
#[derive(Debug)]
pub struct RawInput {
    /// The receiver part of the MPSC channel
    pub rx: MessageReceiver,

    /// The source node layout, useful for debugging
    pub source: NodeID,
    /// The layout of the input, useful for debugging
    pub layout: InputID,
}

impl RawInput {
    /// Create a new RawInput instance
    pub fn new(rx: MessageReceiver, source: NodeID, layout: InputID) -> Self {
        Self { rx, source, layout }
    }

    /// Receive a message from the channel, asynchronously
    pub async fn recv(&mut self) -> Result<DataflowMessage> {
        let message = self
            .rx
            .recv()
            .await
            .ok_or_eyre(report_error_receiving(&self.source, &self.layout))?;

        Ok(message)
    }
}
