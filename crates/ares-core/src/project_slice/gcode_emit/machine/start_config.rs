//! `self_start_config` — the template placeholder `Config` assembled
//! for the machine-start render
//! (`GCode.cpp:3002-3040`, `PrintConfig` machine-limit keys).

use super::super::placeholders;
use super::{first_filaments, footprint, temperature, value};
use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

pub(super) fn self_start_config(
    traversal: &PreparedPostClassicTraversal,
    metadata: GenerationMetadata,
    first_layer_bounds: Option<footprint::FirstLayerBounds>,
) -> Result<super::value::Config, SliceError> {
    let mut config = placeholders::base_config(traversal, metadata, first_layer_bounds)?;
    config.insert("next_extruder", value::Value::number(0.0));
    config.insert("next_hotend", value::Value::number(-1.0));
    config.insert("initial_no_support_extruder", value::Value::number(0.0));
    config.insert("initial_no_support_hotend", value::Value::number(-1.0));
    config.insert(
        "overall_chamber_temperature",
        value::Value::number(f64::from(temperature::overall_chamber_temperature(
            traversal,
        ))),
    );
    config.insert(
        "hold_chamber_temp_for_flat_print",
        value::Value::option_bool(false),
    );
    let max_z = traversal
        .objects
        .first()
        .into_iter()
        .flat_map(|object| object.records.iter())
        .filter_map(|record| record.as_ref())
        .map(|record| record.layer_height)
        .sum::<f64>();
    config.insert("max_print_z", value::Value::number(max_z));
    config.insert("max_layer_z", value::Value::number(max_z));
    config.insert(
        "total_layer_count",
        value::Value::number(
            traversal
                .objects
                .first()
                .map_or(0, |object| object.records.len()) as f64,
        ),
    );
    config.insert("layer_num", value::Value::number(0.0));
    if let Some((min_x, min_y, size_x, size_y)) = first_layer_bounds {
        config.insert(
            "first_layer_print_min",
            value::Value::List(vec![
                value::Value::number(min_x),
                value::Value::number(min_y),
            ]),
        );
        config.insert(
            "first_layer_print_size",
            value::Value::List(vec![
                value::Value::number(size_x),
                value::Value::number(size_y),
            ]),
        );
    }
    // Upstream seeds the writer output vectors (`e_retracted`,
    // `e_restart_extra`, `e_position`) with `MAXIMUM_EXTRUDER_NUMBER`
    // (64) zeros BEFORE rendering (`GCode.cpp:1632-1642`), so
    // `{e_retracted[0]=1}` assigns in place rather than resizing; the
    // `e_position` output exists only for absolute extrusion.
    for key in ["e_retracted", "e_restart_extra", "e_position"] {
        if key == "e_position"
            && traversal
                .resolved
                .views
                .runtime_gcode
                .use_relative_e_distances
                .0
        {
            continue;
        }
        config.insert(key, value::Value::List(vec![value::Value::number(0.0); 64]));
    }
    let (first_filaments, first_non_support_filaments) = first_filaments::resolve(traversal);
    config.insert("first_tools", first_filaments.clone());
    config.insert("first_filaments", first_filaments);
    config.insert(
        "first_non_support_tools",
        first_non_support_filaments.clone(),
    );
    config.insert("first_non_support_filaments", first_non_support_filaments);
    Ok(config)
}
