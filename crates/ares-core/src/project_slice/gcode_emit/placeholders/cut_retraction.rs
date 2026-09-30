use crate::options::GCodeOptions;

use super::value::{Config, Value};

pub(super) fn insert(config: &mut Config, runtime: &GCodeOptions) {
    // GCode::_do_export exposes m_config after filament overrides, not the footer.
    config.insert(
        "long_retractions_when_cut",
        Value::List(
            runtime
                .long_retractions_when_cut
                .0
                .iter()
                .map(|value| Value::option_bool(value.0))
                .collect(),
        ),
    );
    config.insert(
        "retraction_distances_when_cut",
        Value::List(
            runtime
                .retraction_distances_when_cut
                .0
                .iter()
                .map(|value| Value::number(value.0))
                .collect(),
        ),
    );
}
