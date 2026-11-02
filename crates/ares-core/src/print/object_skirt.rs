//! `Print::object_skirt_offset` (`Print.cpp:5796-5817` at v2.5.0-beta `345efec64`) with
//! `Print::skirt_flow` (`:3084-3104`), `nozzle_diameter_for_filament` (`PrintConfig.cpp:148-155`)
//! and `Print::is_all_objects_are_short` (`Print.hpp:1379-1381`): how far a per-object skirt widens
//! a by-object print's clearance. Upstream's locals are `float` and its config reads `double`; the
//! port keeps each promotion and narrowing.

use crate::arrangement::SCALING_FACTOR;
use crate::project_slice::skirt_flow_width_spacing;
use crate::{FloatOrPercent, ProcessDraftShield, ProcessSkirtType, ProjectSettings, SliceError};

/// The first print object's own settings `skirt_flow` reads.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FirstObject {
    pub(crate) line_width: FloatOrPercent,
    pub(crate) support_filament: i32,
}

/// `PrintObject::height()` (`Print.hpp:406`) of an object whose snug Z extent is `size_z` mm:
/// `(bbox.size() * (1. / SCALING_FACTOR)).cast<coord_t>()` (`PrintObject.cpp:161`).
pub(crate) fn print_object_height(size_z: f64) -> i64 {
    (size_z * (1. / SCALING_FACTOR)) as i64
}

/// `is_all_objects_are_short`: every object's scaled height below `scale_(nozzle_height)`.
pub(crate) fn is_all_objects_are_short(
    heights: impl IntoIterator<Item = i64>,
    nozzle_height: f64,
) -> bool {
    let limit = nozzle_height / SCALING_FACTOR;
    heights.into_iter().all(|height| (height as f64) < limit)
}

/// `ConfigOptionFloatOrPercent::get_abs_value(ratio_over)`.
fn get_abs_value(value: FloatOrPercent, ratio_over: f64) -> f64 {
    match value {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(percent) => ratio_over * percent.0 / 100.,
    }
}

/// `nozzle_diameter_for_filament`: on a Bambu Lab printer with several nozzles, the filament goes
/// through `filament_map`; `get_at` falls back to the first nozzle past the list.
fn nozzle_diameter_for_filament(settings: &ProjectSettings, filament_id: i32, is_bbl: bool) -> f64 {
    let nozzles = &settings.project.print.nozzle_diameter.0;
    let filament_map = &settings.project.gcode.filament_map.0;
    let mut extruder = filament_id;
    if is_bbl
        && nozzles.len() > 1
        && filament_id >= 1
        && ((filament_id - 1) as usize) < filament_map.len()
    {
        extruder = filament_map[(filament_id - 1) as usize].0;
    }
    usize::try_from(extruder - 1)
        .ok()
        .and_then(|index| nozzles.get(index))
        .unwrap_or(&nozzles[0])
        .0
}

/// `Print::skirt_flow`: the first-layer line width, else the first object's `line_width`, on the
/// nozzle of the first object's support filament; `width()` and `spacing()`.
fn skirt_flow(
    settings: &ProjectSettings,
    first: FirstObject,
    is_bbl: bool,
) -> Result<(f32, f32), SliceError> {
    let initial = settings.process.print.initial_layer_line_width;
    let width = if initial.is_non_positive() {
        first.line_width
    } else {
        initial
    };
    let nozzle = nozzle_diameter_for_filament(settings, first.support_filament, is_bbl) as f32;
    // `skirt_first_layer_height` (`Print.cpp:3058-3061`).
    let height = settings.process.print.initial_layer_print_height.0 as f32;
    skirt_flow_width_spacing(width, height, nozzle)
}

/// `object_skirt_offset()`, with `margin_height` at its default 0 as both callers pass it: the
/// offset and the skirt's width. `first` is `None` for a print without objects.
pub(crate) fn object_skirt_offset(
    settings: &ProjectSettings,
    first: Option<FirstObject>,
    all_short: bool,
    is_bbl: bool,
) -> Result<(f32, f32), SliceError> {
    let print = &settings.process.print;
    let Some(first) = first else {
        return Ok((0., 0.));
    };
    if print.skirt_loops.0 == 0 || print.skirt_type != ProcessSkirtType::PerObject {
        return Ok((0., 0.));
    }
    let max = |values: &mut dyn Iterator<Item = f64>| values.fold(f64::MIN, f64::max);
    let max_nozzle_diameter =
        max(&mut settings.project.print.nozzle_diameter.0.iter().map(|n| n.0)) as f32;
    let max_layer_height = max(&mut settings
        .project
        .print
        .max_layer_height
        .0
        .iter()
        .map(|h| h.0)) as f32;
    let line_width = get_abs_value(
        print.initial_layer_line_width,
        f64::from(max_nozzle_diameter),
    ) as f32;
    let (width, spacing) = skirt_flow(settings, first, is_bbl)?;
    let object_skirt_width = width + (print.skirt_loops.0 - 1) as f32 * spacing;
    let skirt_distance = print.skirt_distance.0;
    let half_radius = settings.printer.remaining.extruder_clearance_radius.0 / 2.;
    let skirt_layers_height = print.skirt_height.0 as f32 * max_layer_height;
    let offset = if all_short {
        (skirt_distance + f64::from(object_skirt_width)) as f32
    } else if print.draft_shield == ProcessDraftShield::Enabled
        || f64::from(skirt_layers_height) > settings.printer.remaining.nozzle_height.0 - 0.
    {
        (skirt_distance + f64::from(line_width)) as f32
    } else if skirt_distance + f64::from(object_skirt_width) > half_radius {
        (skirt_distance + f64::from(object_skirt_width) - half_radius) as f32
    } else {
        return Ok((0., 0.));
    };
    Ok((offset, object_skirt_width))
}
