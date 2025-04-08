use std::{collections::HashMap, sync::Arc};

use arrow_buffer::ScalarBuffer;
use arrow_schema::{ArrowError, DataType, Field, UnionFields, UnionMode};

use arrow_array::{Array, ArrayRef, UnionArray};
use arrow_data::ArrayData;

use crate::prelude::*;

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

pub fn unpack_union(data: ArrayData) -> (HashMap<String, usize>, Vec<ArrayRef>) {
    let (fields, _, _, children) = UnionArray::from(data).into_parts();

    let map = fields
        .iter()
        .map(|(id, field)| (field.name().into(), id as usize))
        .collect::<HashMap<String, usize>>();

    (map, children)
}

pub fn extract_union_data<T: ArrowMessage>(
    field: &str,
    map: &HashMap<String, usize>,
    children: &[ArrayRef],
) -> ArrowResult<T> {
    T::try_from_arrow(
        children
            .get(
                *map.get(field)
                    .ok_or(ArrowError::InvalidArgumentError(format!(
                        "Field {} not found",
                        field
                    )))?,
            )
            .ok_or(ArrowError::InvalidArgumentError(format!(
                "Field {} not found",
                field
            )))?
            .into_data(),
    )
}

pub fn get_union_fields<T: ArrowMessage>() -> ArrowResult<UnionFields> {
    match T::field("").data_type() {
        DataType::Union(fields, _) => Ok(fields.clone()),
        _ => Err(ArrowError::InvalidArgumentError(
            "Expected Union data type".to_string(),
        )),
    }
}

pub fn make_union_array(
    union_fields: UnionFields,
    children: Vec<ArrayRef>,
) -> ArrowResult<ArrayRef> {
    UnionArray::try_new(
        union_fields,
        ScalarBuffer::from(vec![]),
        Some(ScalarBuffer::from(vec![])),
        children,
    )
    .map(|union| Arc::new(union) as ArrayRef)
}
