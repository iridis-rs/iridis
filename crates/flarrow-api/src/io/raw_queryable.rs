use std::{collections::HashMap, sync::Arc};

use crate::prelude::*;
use thirdparty::arrow_data::ArrayData;

/// Not typed Queryable to receive data from the dataflow
pub struct RawQueryable {
    /// The sender part of the MPSC channel with the Query
    pub tx: HashMap<Uuid, MessageSender>,
    /// The receiver part of the MPSC channel with the Query
    pub rx: MessageReceiver,
    /// Shared clock with the runtime
    pub clock: Arc<HLC>,

    /// The source node layout, useful for debugging
    pub source: NodeLayout,
    /// The layout of the queryable, useful for debugging
    pub layout: QueryableLayout,
}

impl RawQueryable {
    /// Create a new RawQueryable instance
    pub fn new(
        tx: HashMap<Uuid, MessageSender>,
        rx: MessageReceiver,
        clock: Arc<HLC>,
        source: NodeLayout,
        layout: QueryableLayout,
    ) -> Self {
        Self {
            tx,
            rx,
            clock,
            source,
            layout,
        }
    }

    /// Let the queryable handle a message
    pub fn blocking_on_demand(
        &mut self,
        response: impl FnOnce(DataflowMessage) -> Result<ArrayData>,
    ) -> Result<()> {
        let message = self
            .rx
            .blocking_recv()
            .ok_or_eyre(report_error_receiving(&self.source, &self.layout))?;

        let tx = self
            .tx
            .get(&message.header.source.1)
            .ok_or_eyre(report_io_not_found(&self.source, &self.layout))?;

        let data = DataflowMessage {
            header: Header {
                timestamp: self.clock.new_timestamp(),
                source: (self.source.uuid, self.layout.uuid),
            },
            data: response(message).wrap_err(report_error_sending(&self.source, &self.layout))?,
        };

        tx.blocking_send(data)
            .wrap_err(report_error_sending(&self.source, &self.layout))
    }

    /// Let the queryable handle a message asynchronously
    pub async fn on_demand(
        &mut self,
        response: impl AsyncFnOnce(DataflowMessage) -> Result<ArrayData>,
    ) -> Result<()> {
        let message = self
            .rx
            .recv()
            .await
            .ok_or_eyre(report_error_receiving(&self.source, &self.layout))?;

        let tx = self
            .tx
            .get(&message.header.source.1)
            .ok_or_eyre(report_io_not_found(&self.source, &self.layout))?;

        let data = DataflowMessage {
            header: Header {
                timestamp: self.clock.new_timestamp(),
                source: (self.source.uuid, self.layout.uuid),
            },
            data: response(message)
                .await
                .wrap_err(report_error_sending(&self.source, &self.layout))?,
        };

        tx.send(data)
            .await
            .wrap_err(report_error_sending(&self.source, &self.layout))
    }
}
