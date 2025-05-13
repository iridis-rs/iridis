use std::collections::HashSet;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct FlowLayout {
    pub connections: HashSet<(Uuid, Uuid)>, // Send -> Receive
}

impl FlowLayout {
    fn connect_output_input(&mut self, output: impl AsRef<Uuid>, input: impl AsRef<Uuid>) {
        self.connections.insert((*output.as_ref(), *input.as_ref()));
    }

    fn connect_queryable_query(&mut self, queryable: impl AsRef<Uuid>, query: impl AsRef<Uuid>) {
        self.connections
            .insert((*queryable.as_ref(), *query.as_ref()));

        self.connections
            .insert((*query.as_ref(), *queryable.as_ref()));
    }

    pub fn connect(&mut self, a: impl Into<PrimitiveID>, b: impl Into<PrimitiveID>) -> Result<()> {
        let (a, b) = (a.into(), b.into());

        match (a, b) {
            (PrimitiveID::Input(input), PrimitiveID::Output(output)) => {
                self.connect_output_input(output.uuid, input.uuid);
                Ok(())
            }
            (PrimitiveID::Query(query), PrimitiveID::Queryable(queryable)) => {
                self.connect_queryable_query(queryable.uuid, query.uuid);
                Ok(())
            }
            (PrimitiveID::Output(output), PrimitiveID::Input(input)) => {
                self.connect_output_input(output.uuid, input.uuid);
                Ok(())
            }
            (PrimitiveID::Queryable(queryable), PrimitiveID::Query(query)) => {
                self.connect_queryable_query(queryable.uuid, query.uuid);
                Ok(())
            }
            _ => Err(eyre::eyre!(
                "Invalid connection! types `a` and `b` must verify: a != b and (a, b) in {{Input, Output}} or {{Query, Queryable}}"
            )),
        }
    }
}
