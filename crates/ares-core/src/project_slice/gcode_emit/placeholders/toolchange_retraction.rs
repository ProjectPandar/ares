use crate::options::{GCodeOptions, OrcaFloats};

use super::value::{Config, Value};

/// `Print.cpp:4747-4748` replaces the retract keys in the placeholder parser with the
/// overlaid vectors, while the configuration block keeps the machine values.
pub(super) fn insert(config: &mut Config, runtime: &GCodeOptions) {
    for (name, values) in [
        (
            "retract_length_toolchange",
            &runtime.retract_length_toolchange,
        ),
        (
            "retract_restart_extra_toolchange",
            &runtime.retract_restart_extra_toolchange,
        ),
    ] {
        config.insert(name, floats(values));
    }
}

fn floats(values: &OrcaFloats) -> Value {
    Value::List(
        values
            .0
            .iter()
            .map(|value| Value::number(value.0))
            .collect(),
    )
}
