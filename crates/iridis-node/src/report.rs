use crate::prelude::*;

pub fn report_error_receiving(source: &NodeLayout, layout: impl Into<IOLayout>) -> eyre::Report {
    let layout: IOLayout = layout.into();

    eyre::Report::msg(format!(
        "Node '{}' (uuid: {}) failed to receive data from io '{}' (uuid: {})",
        source.label,
        source.uuid,
        layout.label(),
        layout.uuid()
    ))
}

pub fn report_error_sending(source: &NodeLayout, layout: impl Into<IOLayout>) -> eyre::Report {
    let layout: IOLayout = layout.into();

    eyre::Report::msg(format!(
        "Node '{}' (uuid: {}) failed to send data to io '{}' (uuid: {})",
        source.label,
        source.uuid,
        layout.label(),
        layout.uuid()
    ))
}

pub fn report_failed_conversion_from_arrow<T>(
    source: &NodeLayout,
    layout: impl Into<IOLayout>,
) -> eyre::Report {
    let layout: IOLayout = layout.into();

    eyre::Report::msg(format!(
        "Node '{}' (uuid: {}) failed to convert arrow data from input '{}' (uuid: {}) to message T: {}",
        source.label,
        source.uuid,
        layout.label(),
        layout.uuid(),
        std::any::type_name::<T>()
    ))
}

pub fn report_failed_conversion_to_arrow<T>(
    source: &NodeLayout,
    layout: impl Into<IOLayout>,
) -> eyre::Report {
    let layout: IOLayout = layout.into();

    eyre::Report::msg(format!(
        "Node '{}' (uuid: {}) failed to convert message T {} to arrow data for output '{}' (uuid: {})",
        source.label,
        source.uuid,
        std::any::type_name::<T>(),
        layout.label(),
        layout.uuid()
    ))
}

pub fn report_io_not_found(source: &NodeLayout, layout: impl Into<IOLayout>) -> eyre::Report {
    let layout: IOLayout = layout.into();

    eyre::Report::msg(format!(
        "Input '{}' (uuid: {}) not found for node '{}'. The IO you're trying to create does not match the dataflow layout created.",
        layout.label(),
        layout.uuid(),
        source.label
    ))
}
