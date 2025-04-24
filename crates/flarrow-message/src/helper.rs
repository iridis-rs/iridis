use std::{collections::HashMap, sync::Arc};

use crate::prelude::*;

use arrow_array::*;
use arrow_buffer::*;
use arrow_data::*;
use arrow_schema::*;

/// Helper function to create a union field.
pub fn make_union_fields(name: impl Into<String>, fields: Vec<Field>) -> Field {
    Field::new(
        name,
        DataType::Union(
            UnionFields::new(0..fields.len() as i8, fields),
            UnionMode::Dense,
        ),
        false,
    )
}

/// Helper function to unpack a union array.
pub fn unpack_union(data: ArrayData) -> (HashMap<String, usize>, Vec<ArrayRef>) {
    let (fields, _, _, children) = UnionArray::from(data).into_parts();

    let map = fields
        .iter()
        .map(|(id, field)| (field.name().into(), id as usize))
        .collect::<HashMap<String, usize>>();

    (map, children)
}

/// Helper function to extract data from a union array.
pub fn extract_union_data<T: ArrowMessage>(
    field: &str,
    map: &HashMap<String, usize>,
    children: &[ArrayRef],
) -> Result<T> {
    T::try_from_arrow(
        children
            .get(
                *map.get(field)
                    .ok_or(eyre::eyre!("Field {} not found", field))?,
            )
            .ok_or(eyre::eyre!("Field {} not found", field))?
            .into_data(),
    )
}

/// Helper function to get the union fields of a message.
pub fn get_union_fields<T: ArrowMessage>() -> Result<UnionFields> {
    match T::field("").data_type() {
        DataType::Union(fields, _) => Ok(fields.clone()),
        _ => Err(eyre::eyre!("Expected Union data type")),
    }
}

/// Helper function to create a union array.
pub fn make_union_array(union_fields: UnionFields, children: Vec<ArrayRef>) -> Result<ArrayRef> {
    UnionArray::try_new(
        union_fields,
        ScalarBuffer::from(vec![]),
        Some(ScalarBuffer::from(vec![])),
        children,
    )
    .map(|union| Arc::new(union) as ArrayRef)
    .map_err(eyre::Report::msg)
}
