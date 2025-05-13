use iridis_node::prelude::thirdparty::Uuid;

use crate::prelude::*;

pub fn report_primitive_not_found(uuid: Uuid) -> eyre::Report {
    eyre::Report::msg(format!(
        "Primitive '{}' not found. This is a big error. Please report it",
        uuid
    ))
}
