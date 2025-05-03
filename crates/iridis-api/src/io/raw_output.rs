use std::sync::Arc;

use crate::prelude::*;
use thirdparty::arrow_data::ArrayData;

/// Not typed Output to receive data from the dataflow
pub struct RawOutput {
    /// The senders parts of MPSC channels
    pub tx: Vec<MessageSender>,
    /// The shared clock of the runtime
    pub clock: Arc<HLC>,

    /// The source node layout, useful for debugging
    pub source: NodeLayout,
    /// The layout of the output, useful for debugging
    pub layout: OutputLayout,
}

impl RawOutput {
    /// Create a new RawOutput instance
    pub fn new(
        tx: Vec<MessageSender>,
        clock: Arc<HLC>,
        source: NodeLayout,
        layout: OutputLayout,
    ) -> Self {
        Self {
            tx,
            clock,
            source,
            layout,
        }
    }

    /// Send a message asynchronously to all connected nodes.
    pub async fn send(&self, data: ArrayData) -> Result<()> {
        let data = DataflowMessage {
            header: Header {
                timestamp: self.clock.new_timestamp(),
                source: (self.source.uuid, self.layout.uuid),
            },
            data,
        };

        let mut tasks = Vec::new();

        for tx in &self.tx {
            let tx = tx.clone();
            let data = data.clone();

            let source = self.source.clone();
            let layout = self.layout.clone();

            tasks.push(tokio::spawn(async move {
                tx.send(data)
                    .await
                    .map_err(eyre::Report::msg)
                    .wrap_err(report_error_sending(&source, layout))
            }));
        }

        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(err) => results.push(Err(err.into())),
            }
        }

        if results.iter().all(|r| r.is_ok()) {
            Ok(())
        } else {
            let combined_report: eyre::Report = results
                .into_iter()
                .filter(Result::is_err)
                .map(Result::unwrap_err)
                .fold(
                    eyre::eyre!(
                        "Node '{}' (uuid: {}) encountered multiple errors",
                        self.source.label,
                        self.source.uuid
                    ),
                    |report, e| e.wrap_err(report),
                );

            Err(combined_report)
        }
    }
}
