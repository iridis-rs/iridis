use std::{collections::HashMap, sync::Arc};

use eyre::{Context, OptionExt, bail};

use arrow_array::Array;
use arrow_data::ArrayData;

use crate::prelude::*;

pub struct RawQueryable {
    clock: Arc<uhlc::HLC>,

    pub tx: HashMap<QueryUUID, DataflowSender>,
    pub rx: DataflowReceiver,
}

impl RawQueryable {
    pub fn new(
        clock: Arc<uhlc::HLC>,
        tx: HashMap<QueryUUID, DataflowSender>,
        rx: DataflowReceiver,
    ) -> Self {
        Self { clock, tx, rx }
    }

    pub fn on_demand(
        &mut self,
        response: impl FnOnce(DataflowMessage) -> Result<ArrayData>,
    ) -> Result<()> {
        let message = self
            .rx
            .blocking_recv()
            .ok_or_eyre("Failed to receive from this input")?;

        let tx = &match message.header.source {
            Some(Source::Query(id)) => self.tx.get(&id).ok_or_eyre("Failed to find query")?,
            _ => bail!("Invalid source"),
        };

        let data = DataflowMessage {
            header: Header {
                timestamp: self.clock.new_timestamp(),
                node: None,
                source: None,
            },
            data: response(message).wrap_err("Failed to send response")?,
        };

        tx.blocking_send(data).wrap_err("Failed to send response")
    }

    pub async fn on_demand_async(
        &mut self,
        response: impl AsyncFnOnce(DataflowMessage) -> Result<ArrayData>,
    ) -> Result<()> {
        let message = self
            .rx
            .recv()
            .await
            .ok_or_eyre("Failed to receive from this input")?;

        let tx = &match message.header.source {
            Some(Source::Query(id)) => self.tx.get(&id).ok_or_eyre("Failed to find query")?,
            _ => bail!("Invalid source"),
        };

        let data = DataflowMessage {
            header: Header {
                timestamp: self.clock.new_timestamp(),
                node: None,
                source: None,
            },
            data: response(message)
                .await
                .wrap_err("Failed to send response")?,
        };

        tx.send(data).await.wrap_err("Failed to send response")
    }
}

pub struct Queryable<T: ArrowMessage, F: ArrowMessage> {
    pub raw: RawQueryable,

    _phantom: std::marker::PhantomData<(T, F)>,
}

impl<T: ArrowMessage, F: ArrowMessage> Queryable<T, F> {
    pub fn new(
        clock: Arc<uhlc::HLC>,
        tx: HashMap<QueryUUID, DataflowSender>,
        rx: DataflowReceiver,
    ) -> Self {
        Self {
            raw: RawQueryable::new(clock, tx, rx),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn on_demand(&mut self, response: impl FnOnce(T) -> Result<F>) -> Result<()> {
        self.raw.on_demand(move |message| {
            let result = response(
                T::try_from_arrow(message.data)
                    .wrap_err("Failed to convert arrow data to message")?,
            )?;

            Ok(result
                .try_into_arrow()
                .wrap_err("Failed to convert response to arrow data")?
                .into_data())
        })
    }

    pub async fn on_demand_async(
        &mut self,
        response: impl AsyncFnOnce(T) -> Result<F>,
    ) -> Result<()> {
        self.raw
            .on_demand_async(async move |message| {
                let result = response(
                    T::try_from_arrow(message.data)
                        .wrap_err("Failed to convert arrow data to message")?,
                )
                .await?;

                Ok(result
                    .try_into_arrow()
                    .wrap_err("Failed to convert response to arrow data")?
                    .into_data())
            })
            .await
    }
}

pub struct Queryables {
    node: NodeUUID,

    clock: Arc<uhlc::HLC>,

    senders: SharedMap<QueryableUUID, HashMap<QueryUUID, DataflowSender>>,
    receivers: SharedMap<QueryableUUID, DataflowReceiver>,
}

impl Queryables {
    pub fn new(
        node: NodeUUID,
        clock: Arc<uhlc::HLC>,
        senders: SharedMap<QueryableUUID, HashMap<QueryUUID, DataflowSender>>,
        receivers: SharedMap<QueryableUUID, DataflowReceiver>,
    ) -> Self {
        Self {
            node,
            clock,
            senders,
            receivers,
        }
    }

    pub async fn raw(&mut self, queryable: impl Into<String>) -> Result<RawQueryable> {
        let id = self.node.queryable(queryable);

        let sender = self
            .senders
            .lock()
            .await
            .remove(&id)
            .ok_or_eyre(format!("Input {} not found", id.0))?;

        let receiver = self
            .receivers
            .lock()
            .await
            .remove(&id)
            .ok_or_eyre(format!("Input {} not found", id.0))?;

        Ok(RawQueryable::new(self.clock.clone(), sender, receiver))
    }

    pub async fn with<T: ArrowMessage, F: ArrowMessage>(
        &mut self,
        queryable: impl Into<String>,
    ) -> Result<Queryable<T, F>> {
        let id = self.node.queryable(queryable);

        let sender = self
            .senders
            .lock()
            .await
            .remove(&id)
            .ok_or_eyre(format!("Input {} not found", id.0))?;

        let receiver = self
            .receivers
            .lock()
            .await
            .remove(&id)
            .ok_or_eyre(format!("Input {} not found", id.0))?;

        Ok(Queryable::new(self.clock.clone(), sender, receiver))
    }
}
