use crate::project_slice::{
    island_print_order::{IslandPrintEntity, OrderedExtrusionLayer, PreparedPostIslandPrintOrder},
    perimeters::classic::traversal::PreparedPostClassicTraversal,
};

mod expression;
mod header;
mod lexer;
mod template;
mod value;
use crate::{GenerationMetadata, SliceError};

pub(super) fn emit(
    prepared: &PreparedPostIslandPrintOrder,
    traversal: &PreparedPostClassicTraversal,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    let mut output = Vec::new();
    header::append_header(&mut output, metadata, &prepared.objects, traversal);
    if let Some(config) = &traversal.config_block {
        output.extend_from_slice(config);
    }
    header::append_width_block(&mut output, traversal);
    output.extend_from_slice(b"; EXECUTABLE_BLOCK_START\n");
    append_machine_limits(&mut output, traversal);
    append_machine_start(&mut output, prepared, traversal)?;
    let mut state = EmitState::default();
    for object in &prepared.objects {
        for (layer_index, layer) in object.iter().enumerate() {
            if layer_index == 0 {
                append_print_preamble(&mut output);
            }
            output.extend_from_slice(b"; CHANGE_LAYER\n");
            let layer_height = traversal
                .objects
                .first()
                .and_then(|object| object.records.get(layer_index))
                .and_then(|record| record.as_ref())
                .map_or(0.0, |record| record.layer_height);
            let layer_z = object
                .iter()
                .take(layer_index + 1)
                .enumerate()
                .map(|(index, _)| {
                    traversal
                        .objects
                        .first()
                        .and_then(|object| object.records.get(index))
                        .and_then(|record| record.as_ref())
                        .map_or(0.0, |record| record.layer_height)
                })
                .sum::<f64>();
            output.extend_from_slice(
                format!("; Z_HEIGHT: {layer_z}\n; LAYER_HEIGHT: {layer_height}\n").as_bytes(),
            );
            if layer_index == 0 {
                output.extend_from_slice(b"G1 E-.4 F1800\n");
            }
            append_layer_change(&mut output, traversal, layer_index, layer_z)?;
            emit_layer(&mut output, layer, traversal.scale, &mut state)?;
        }
    }

    fn append_print_preamble(output: &mut Vec<u8>) {
        output.extend_from_slice(
            b"; filament start gcode\n;VT0\nG90\nG21\nM83 ; use relative distances for extrusion\n",
        );
        output.extend_from_slice(b"M981 S1 P20000 ;open spaghetti detector\nM106 S0\nM106 P2 S0\n");
    }

    fn append_layer_change(
        output: &mut Vec<u8>,
        traversal: &PreparedPostClassicTraversal,
        layer_index: usize,
        layer_z: f64,
    ) -> Result<(), SliceError> {
        let template = &traversal.resolved.views.runtime_gcode.layer_change_gcode.0;
        if template.is_empty() {
            return Ok(());
        }
        let mut config =
            value::Config::from_block(traversal.config_block.as_deref().unwrap_or_default());
        config.insert("current_extruder", value::Value::number(0.0));
        config.insert("layer_num", value::Value::number(layer_index as f64));
        config.insert("layer_z", value::Value::number(layer_z));
        config.insert("overall_chamber_temperature", value::Value::number(0.0));
        if let Some(value) = config
            .get("temperature_vitrification")
            .and_then(|value| value.index(0))
            .cloned()
        {
            config.insert("min_vitrification_temperature", value);
        }
        if let Some(value) = config
            .get("fan_max_speed")
            .and_then(|value| value.index(0))
            .cloned()
        {
            config.insert("max_additional_fan", value);
        }
        let rendered = template::render(template, &config).map_err(|error| {
            SliceError::InvalidInput(format!(
                "invalid project layer-change G-code template: {error}"
            ))
        })?;
        output.extend_from_slice(rendered.as_bytes());
        Ok(())
    }
    output.extend_from_slice(b"M2\n");
    Ok(output)
}

#[derive(Default)]
struct EmitState {
    x: f64,
    y: f64,
    e: f64,
}

fn append_machine_limits(output: &mut Vec<u8>, traversal: &PreparedPostClassicTraversal) {
    let machine = &traversal.resolved.views.full.printer.machine;
    let travel_acceleration = if traversal.resolved.views.full.printer.gcode.gcode_flavor
        == crate::GCodeFlavor::MarlinLegacy
    {
        machine.machine_max_acceleration_extruding.clone()
    } else {
        machine.machine_max_acceleration_travel.clone()
    };
    output.extend_from_slice(b"M73 P0 R0\n");
    output.extend_from_slice(
        format!(
            "M201 X{} Y{} Z{} E{}\n",
            first(&machine.machine_max_acceleration_x),
            first(&machine.machine_max_acceleration_y),
            first(&machine.machine_max_acceleration_z),
            first(&machine.machine_max_acceleration_e),
        )
        .as_bytes(),
    );
    output.extend_from_slice(
        format!(
            "M203 X{} Y{} Z{} E{}\n",
            first(&machine.machine_max_speed_x),
            first(&machine.machine_max_speed_y),
            first(&machine.machine_max_speed_z),
            first(&machine.machine_max_speed_e),
        )
        .as_bytes(),
    );
    output.extend_from_slice(
        format!(
            "M204 P{} R{} T{}\n",
            first(&machine.machine_max_acceleration_extruding),
            first(&machine.machine_max_acceleration_retracting),
            first(&travel_acceleration),
        )
        .as_bytes(),
    );
    output.extend_from_slice(
        format!(
            "M205 X{:.2} Y{:.2} Z{:.2} E{:.2} ; sets the jerk limits, mm/sec\n",
            first(&machine.machine_max_jerk_x),
            first(&machine.machine_max_jerk_y),
            first(&machine.machine_max_jerk_z),
            first(&machine.machine_max_jerk_e),
        )
        .as_bytes(),
    );
    output.extend_from_slice(b"M106 S0\nM106 P2 S0\n");
}

fn first(values: &crate::OrcaFloats) -> f64 {
    values.0.first().map_or(0.0, |value| value.0)
}

#[expect(
    clippy::excessive_nesting,
    reason = "keeps extrusion entity bounds traversal ordered"
)]
fn first_layer_bounds(
    prepared: &PreparedPostIslandPrintOrder,
    scale: crate::geometry::CoordinateScale,
) -> Option<(f64, f64, f64, f64)> {
    let layer = prepared.objects.first()?.first()?;
    let mut bounds = None::<(f64, f64, f64, f64)>;
    let mut include = |x: f64, y: f64| {
        bounds = Some(match bounds {
            Some((min_x, min_y, max_x, max_y)) => {
                (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
            }
            None => (x, y, x, y),
        });
    };
    for island in &layer.islands {
        for entity in &island.entities {
            match entity {
                IslandPrintEntity::Perimeter(collection) => {
                    for loop_ in &collection.entities {
                        for path in &loop_.extrusion_loop.paths {
                            for point in &path.polyline.points {
                                include(scale.unscale(point.x), scale.unscale(point.y));
                            }
                        }
                    }
                }
                IslandPrintEntity::Fill(collection) => {
                    for path in &collection.paths {
                        for point in path.polyline.points() {
                            include(scale.unscale(point.x()), scale.unscale(point.y()));
                        }
                    }
                }
                IslandPrintEntity::Thin(entity) => match entity {
                    crate::project_slice::perimeters::classic::gap_extrusion::GapFillEntity::Path(path) => {
                        for point in &path.polyline.points {
                            include(scale.unscale(point.x), scale.unscale(point.y));
                        }
                    }
                    crate::project_slice::perimeters::classic::gap_extrusion::GapFillEntity::Loop(paths) => {
                        for path in paths {
                            for point in &path.polyline.points {
                                include(scale.unscale(point.x), scale.unscale(point.y));
                            }
                        }
                    }
                },
            }
        }
    }
    bounds.map(|(min_x, min_y, max_x, max_y)| (min_x, min_y, max_x - min_x, max_y - min_y))
}

fn append_machine_start(
    output: &mut Vec<u8>,
    prepared: &PreparedPostIslandPrintOrder,
    traversal: &PreparedPostClassicTraversal,
) -> Result<(), SliceError> {
    let template = &traversal.resolved.views.runtime_gcode.machine_start_gcode.0;
    if template.is_empty() {
        return Ok(());
    }
    let mut config =
        value::Config::from_block(traversal.config_block.as_deref().unwrap_or_default());
    config.insert("current_extruder", value::Value::number(0.0));
    config.insert("current_hotend", value::Value::number(-1.0));
    config.insert("next_extruder", value::Value::number(0.0));
    config.insert("next_hotend", value::Value::number(-1.0));
    config.insert("initial_no_support_extruder", value::Value::number(0.0));
    config.insert("initial_no_support_hotend", value::Value::number(-1.0));
    config.insert("overall_chamber_temperature", value::Value::number(0.0));
    config.insert(
        "hold_chamber_temp_for_flat_print",
        value::Value::Bool(false),
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
    if let Some((min_x, min_y, size_x, size_y)) = first_layer_bounds(prepared, traversal.scale) {
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
    let filament_count = traversal.resolved.logical_filament_count;
    config.insert(
        "first_non_support_filaments",
        value::Value::List(
            (0..filament_count)
                .map(|_| value::Value::number(-1.0))
                .collect(),
        ),
    );
    config.insert(
        "first_filaments",
        value::Value::List(
            (0..filament_count)
                .map(|index| value::Value::number(index as f64 - 1.0))
                .collect(),
        ),
    );
    for (target, source) in [
        ("flush_temperatures", "nozzle_temperature_range_high"),
        ("flush_volumetric_speeds", "filament_max_volumetric_speed"),
        (
            "first_layer_temperature",
            "nozzle_temperature_initial_layer",
        ),
    ] {
        if let Some(value) = config.get(source).cloned() {
            config.insert(target, value);
        }
    }
    if let Some(value) = config
        .get("hot_plate_temp_initial_layer")
        .and_then(|value| value.index(0))
        .cloned()
    {
        config.insert("bed_temperature_initial_layer_single", value);
    }
    output.extend_from_slice(b"; FEATURE: Custom\n");
    let rendered = template::render(template, &config).map_err(|error| {
        SliceError::InvalidInput(format!("invalid project G-code template: {error}"))
    })?;
    output.extend_from_slice(rendered.as_bytes());
    if !rendered.ends_with('\n') {
        output.push(b'\n');
    }
    Ok(())
}

#[expect(
    clippy::excessive_nesting,
    reason = "keeps the source ordered extrusion-entity traversal together"
)]
fn emit_layer(
    output: &mut Vec<u8>,
    layer: &OrderedExtrusionLayer,
    scale: crate::geometry::CoordinateScale,
    state: &mut EmitState,
) -> Result<(), SliceError> {
    for island in &layer.islands {
        for entity in &island.entities {
            match entity {
                IslandPrintEntity::Perimeter(collection) => {
                    for loop_ in &collection.entities {
                        for path in &loop_.extrusion_loop.paths {
                            emit_path(output, path, scale, state);
                        }
                    }
                }
                IslandPrintEntity::Fill(collection) => {
                    for path in &collection.paths {
                        emit_polyline(output, &path.polyline, path.mm3_per_mm, scale, state);
                    }
                }
                IslandPrintEntity::Thin(entity) => match entity {
                    crate::project_slice::perimeters::classic::gap_extrusion::GapFillEntity::Path(path) => {
                        emit_path(output, path, scale, state);
                    }
                    crate::project_slice::perimeters::classic::gap_extrusion::GapFillEntity::Loop(paths) => {
                        for path in paths {
                            emit_path(output, path, scale, state);
                        }
                    }
                },
            }
        }
    }
    Ok(())
}

fn emit_path(
    output: &mut Vec<u8>,
    path: &crate::project_slice::perimeters::classic::materialize::ExtrusionPath,
    scale: crate::geometry::CoordinateScale,
    state: &mut EmitState,
) {
    emit_polyline3(output, &path.polyline.points, path.mm3_per_mm, scale, state);
}

fn emit_polyline(
    output: &mut Vec<u8>,
    polyline: &crate::geometry::Polyline,
    mm3_per_mm: f64,
    scale: crate::geometry::CoordinateScale,
    state: &mut EmitState,
) {
    emit_points(
        output,
        polyline.points().iter().map(|point| (point.x(), point.y())),
        mm3_per_mm,
        scale,
        state,
    );
}

fn emit_polyline3(
    output: &mut Vec<u8>,
    points: &[crate::project_slice::perimeters::classic::materialize::Point3],
    mm3_per_mm: f64,
    scale: crate::geometry::CoordinateScale,
    state: &mut EmitState,
) {
    emit_points(
        output,
        points.iter().map(|point| (point.x, point.y)),
        mm3_per_mm,
        scale,
        state,
    );
}

fn emit_points(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    mm3_per_mm: f64,
    scale: crate::geometry::CoordinateScale,
    state: &mut EmitState,
) {
    for (x, y) in points {
        let x = scale.unscale(x);
        let y = scale.unscale(y);
        let distance = (x - state.x).hypot(y - state.y);
        state.e += distance * mm3_per_mm;
        output.extend_from_slice(format!("G1 X{x:.5} Y{y:.5} E{:.5}\n", state.e).as_bytes());
        state.x = x;
        state.y = y;
    }
}
