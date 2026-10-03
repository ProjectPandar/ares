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

pub(in crate::project_slice::gcode_emit) fn insert_aliases(config: &mut Config, filament: usize) {
    for (target, source) in [
        (
            "retraction_distance_when_cut",
            "retraction_distances_when_cut",
        ),
        ("long_retraction_when_cut", "long_retractions_when_cut"),
        (
            "retraction_distance_when_ec",
            "retraction_distances_when_ec",
        ),
        ("long_retraction_when_ec", "long_retractions_when_ec"),
    ] {
        let Some(value) = config.get(source).and_then(|value| value.index(filament)) else {
            continue;
        };
        let alias = if target.starts_with("long_") {
            value
                .as_number()
                .map_or_else(|| value.clone(), |value| Value::option_bool(value != 0.0))
        } else {
            value.clone()
        };
        config.insert(target, alias);
    }
}
