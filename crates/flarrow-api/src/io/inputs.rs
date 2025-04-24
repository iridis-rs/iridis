use std::{collections::HashMap, sync::Arc};

use crate::prelude::*;

use thirdparty::tokio::sync::Mutex;

type SharedMap<K, V> = Arc<Mutex<HashMap<K, V>>>;
type Receivers = SharedMap<Uuid, MessageReceiver>;

/// Inputs let you manage input connections during a node *implementation*
#[derive(Debug)]
pub struct Inputs {
    receivers: Receivers,

    source: NodeLayout,
}

impl Inputs {
    /// Creates a new Inputs instance.
    pub fn new(receivers: Receivers, source: NodeLayout) -> Self {
        tracing::debug!(
            "Creating Inputs entry for node '{}' (uuid: {})",
            source.label,
            source.uuid
        );

        Self { receivers, source }
    }

    async fn compute(
        &mut self,
        input: impl Into<String>,
    ) -> Result<(MessageReceiver, InputLayout)> {
        let label: String = input.into();
        let layout = self.source.input(&label);

        let receiver = self
            .receivers
            .lock()
            .await
            .remove(&layout.uuid)
            .ok_or_eyre(report_io_not_found(&self.source, &layout))?;

        Ok((receiver, layout))
    }

    /// Creates a new raw Input, this raw input has no type information so you have
    /// to manually transform it
    pub async fn raw(&mut self, input: impl Into<String>) -> Result<RawInput> {
        let (receiver, layout) = self.compute(input).await?;

        tracing::debug!(
            "Creating new raw input '{}' (uuid: {}) for node '{}' (uuid: {})",
            layout.label,
            layout.uuid,
            self.source.label,
            self.source.uuid
        );

        Ok(RawInput::new(receiver, self.source.clone(), layout))
    }

    /// Creates a new Input, this input has type information so it can be directly transformed
    pub async fn with<T: ArrowMessage>(&mut self, input: impl Into<String>) -> Result<Input<T>> {
        let (receiver, layout) = self.compute(input).await?;

        tracing::debug!(
            "Creating new input '{}' (uuid: {}) for node '{}' (uuid: {})",
            layout.label,
            layout.uuid,
            self.source.label,
            self.source.uuid
        );

        Ok(Input::new(receiver, self.source.clone(), layout))
    }
}
