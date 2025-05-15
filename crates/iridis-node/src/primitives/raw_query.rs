//! This module contains implementations for this primitive.

use std::sync::Arc;

use crate::prelude::{thirdparty::arrow_data::ArrayData, *};

/// Not typed Query to receive data from the dataflow
pub struct RawQuery {
    /// The sender part of the MPSC channel with the Queryable
    pub tx: MessageSender,
    /// The receiver part of the MPSC channel with the Queryable
    pub rx: MessageReceiver,
    /// Shared clock with the runtime
    pub clock: Arc<HLC>,

    /// The source node layout, useful for debugging
    pub source: NodeID,
    /// The layout of the query, useful for debugging
    pub layout: QueryID,
}

impl RawQuery {
    /// Create a new RawQuery instance
    pub fn new(
        tx: MessageSender,
        rx: MessageReceiver,
        clock: Arc<HLC>,
        source: NodeID,
        layout: QueryID,
    ) -> Self {
        Self {
            tx,
            rx,
            clock,
            source,
            layout,
        }
    }

    /// Query a message to a queryable
    pub async fn query(&mut self, data: ArrayData) -> Result<DataflowMessage> {
        let data = DataflowMessage {
            header: Header {
                timestamp: self.clock.new_timestamp(),
                source: (self.source.uuid, self.layout.uuid),
            },
            data,
        };

        self.tx
            .send(data)
            .await
            .wrap_err(report_error_sending(&self.source, &self.layout))?;

        let message = self
            .rx
            .recv()
            .await
            .ok_or_eyre(report_error_receiving(&self.source, &self.layout))?;

        Ok(message)
    }
}
