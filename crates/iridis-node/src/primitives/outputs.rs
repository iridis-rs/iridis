use std::{collections::HashMap, sync::Arc};

use crate::prelude::*;

use thirdparty::tokio::sync::Mutex;

type SharedMap<K, V> = Arc<Mutex<HashMap<K, V>>>;
type Senders = SharedMap<Uuid, Vec<MessageSender>>;

/// Outputs let you manage output connections during a node *implementation*
pub struct Outputs {
    senders: Senders,
    clock: Arc<uhlc::HLC>,

    source: NodeID,
}

impl Outputs {
    /// Creates a new instance of `Outputs`
    pub fn new(senders: Senders, clock: Arc<uhlc::HLC>, source: NodeID) -> Self {
        Self {
            senders,
            clock,
            source,
        }
    }

    async fn compute(
        &mut self,
        output: impl Into<String>,
    ) -> Result<(Vec<MessageSender>, OutputID)> {
        let label: String = output.into();
        let layout = self.source.output(&label);

        let senders = self
            .senders
            .lock()
            .await
            .remove(&layout.uuid)
            .ok_or_eyre(report_io_not_found(&self.source, &layout))?;

        Ok((senders, layout))
    }

    /// Creates a new raw Output, this raw output has no type information so you have
    /// to manually transform it
    pub async fn raw(&mut self, output: impl Into<String>) -> Result<RawOutput> {
        let (senders, layout) = self.compute(output).await?;

        tracing::debug!(
            "Creating new raw output '{}' (uuid: {}) for node '{}' (uuid: {})",
            layout.label,
            layout.uuid,
            self.source.label,
            self.source.uuid
        );

        Ok(RawOutput::new(
            senders,
            self.clock.clone(),
            self.source.clone(),
            layout,
        ))
    }

    /// Creates a new typed Output, this output has type information so you don't have
    /// to manually transform it
    pub async fn with<T: ArrowMessage>(&mut self, output: impl Into<String>) -> Result<Output<T>> {
        let (senders, layout) = self.compute(output).await?;

        tracing::debug!(
            "Creating new raw output '{}' (uuid: {}) for node '{}' (uuid: {})",
            layout.label,
            layout.uuid,
            self.source.label,
            self.source.uuid
        );

        Ok(Output::new(
            senders,
            self.clock.clone(),
            self.source.clone(),
            layout,
        ))
    }
}
