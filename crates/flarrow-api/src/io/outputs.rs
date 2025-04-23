use eyre::{Context, OptionExt, eyre};
use std::sync::Arc;

use arrow_array::Array;
use arrow_data::ArrayData;

use crate::prelude::*;

pub struct RawOutput {
    clock: Arc<uhlc::HLC>,

    pub tx: Vec<DataflowSender>,
}

impl RawOutput {
    pub fn new(clock: Arc<uhlc::HLC>, tx: Vec<DataflowSender>) -> Self {
        Self { clock, tx }
    }

    pub fn send(&self, data: ArrayData) -> Result<()> {
        let data = DataflowMessage {
            header: Header {
                timestamp: self.clock.new_timestamp(),
                node: None,
                source: None,
            },
            data,
        };

        let results: Vec<Result<()>> = self
            .tx
            .iter()
            .map(|tx| {
                tx.blocking_send(data.clone())
                    .map_err(eyre::Report::msg)
                    .wrap_err("Failed to send the message")
            })
            .collect();

        if results.iter().all(|r| r.is_ok()) {
            Ok(())
        } else {
            let combined_report: eyre::Report = results
                .into_iter()
                .filter(Result::is_err)
                .map(Result::unwrap_err)
                .fold(eyre!("encountered multiple errors"), |report, e| {
                    report.wrap_err(e)
                });
            Err(combined_report)
        }
    }

    pub async fn send_async(&self, data: ArrayData) -> Result<()> {
        let data = DataflowMessage {
            header: Header {
                timestamp: self.clock.new_timestamp(),
                node: None,
                source: None,
            },
            data,
        };

        let mut tasks = Vec::new();

        for tx in &self.tx {
            let tx = tx.clone();
            let data = data.clone();
            tasks.push(tokio::spawn(async move {
                tx.send(data)
                    .await
                    .map_err(eyre::Report::msg)
                    .wrap_err("Failed to send the message")
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
                .fold(eyre!("encountered multiple errors"), |report, e| {
                    report.wrap_err(e)
                });

            Err(combined_report)
        }
    }
}

pub struct Output<T: ArrowMessage> {
    pub raw: RawOutput,

    _phantom: std::marker::PhantomData<T>,
}

impl<T: ArrowMessage> Output<T> {
    pub fn new(clock: Arc<uhlc::HLC>, tx: Vec<DataflowSender>) -> Self {
        Self {
            raw: RawOutput::new(clock, tx),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn send(&self, data: T) -> Result<()> {
        self.raw.send(
            data.try_into_arrow()
                .wrap_err("Failed to convert arrow 'data' to message T")?
                .into_data(),
        )
    }

    pub async fn send_async(&self, data: T) -> Result<()> {
        self.raw
            .send_async(
                data.try_into_arrow()
                    .wrap_err("Failed to convert arrow 'data' to message T")?
                    .into_data(),
            )
            .await
    }
}

pub struct Outputs {
    node: NodeUUID,
    clock: Arc<uhlc::HLC>,
    senders: SharedMap<OutputUUID, Vec<DataflowSender>>,
}

impl Outputs {
    #[allow(clippy::type_complexity)]
    pub fn new(
        node: NodeUUID,
        clock: Arc<uhlc::HLC>,
        senders: SharedMap<OutputUUID, Vec<DataflowSender>>,
    ) -> Self {
        Self {
            node,
            clock,
            senders,
        }
    }

    pub async fn raw(&mut self, output: impl Into<String>) -> Result<RawOutput> {
        let id = self.node.output(output);

        let sender = self
            .senders
            .lock()
            .await
            .remove(&id)
            .ok_or_eyre(format!("Output {} not found", id.0))?;

        Ok(RawOutput::new(self.clock.clone(), sender))
    }

    pub async fn with<T: ArrowMessage>(&mut self, output: impl Into<String>) -> Result<Output<T>> {
        let id = self.node.output(output);

        let sender = self
            .senders
            .lock()
            .await
            .remove(&id)
            .ok_or_eyre(format!("Output {} not found", id.0))?;

        Ok(Output::new(self.clock.clone(), sender))
    }
}
