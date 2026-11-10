//! `self_start_config` — the template placeholder `Config` assembled
//! for the machine-start render
//! (`GCode.cpp:3002-3040`, `PrintConfig` machine-limit keys).

use super::super::{placeholders, value};
use super::{footprint, temperature};
use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

pub(super) fn self_start_config(
    traversal: &PreparedPostClassicTraversal,
    state: &super::super::motion::EmitState,
    metadata: GenerationMetadata,
    first_layer_bounds: Option<footprint::FirstLayerBounds>,
) -> Result<value::Config, SliceError> {
    let mut config = placeholders::base_config(
        traversal,
        metadata,
        first_layer_bounds,
        Some(&state.template_rng),
        state.total_layer_count,
    )?;
    config.insert("next_extruder", value::Value::Integer(0));
    config.insert("next_hotend", value::Value::Integer(-1));
    config.insert(
        "overall_chamber_temperature",
        value::Value::Integer(i64::from(temperature::overall_chamber_temperature(
            traversal,
            state.extruders.ids(),
        ))),
    );
    let max_z = traversal
        .objects
        .first()
        .into_iter()
        .flat_map(|object| object.records.iter())
        .filter_map(|record| record.as_ref())
        .map(|record| record.layer_height)
        .sum::<f64>();
    config.insert("max_layer_z", value::Value::number(max_z));
    super::super::motion::refresh_writer_state(&mut config, state);
    state.initial_filaments.insert_machine(&mut config);
    Ok(config)
}
