//! The arrange parameters and print config the job reads: `init_arrange_params`
//! (`ArrangeJob.cpp:989-1043` at v2.5.0-beta `345efec64`), `get_bed_shape(cfg, true)`
//! (`PrintConfig.cpp:13812-13864`) and `get_real_skirt_dist` (`:14405-14438`).

use super::ArrangeSettings;
use crate::SliceError;
use crate::arrangement::{ArrangeConfig, ArrangeParams, SCALING_FACTOR};
use crate::geometry::{Point, Polygon, intersection_polygons_paths};
use crate::options::{FloatOrPercent, ProcessDraftShield, ProcessPrintSequence, ProjectSettings};

/// `BED_SHRINK_SEQ_PRINT` (`Arrange.hpp:17`).
const BED_SHRINK_SEQ_PRINT: f32 = 5.0;

/// `coord_t(scale_(value))`.
fn scaled(value: f64) -> i64 {
    (value / SCALING_FACTOR) as i64
}

/// Whether the global print sequence is by object.
pub(super) fn is_seq_print(settings: &ProjectSettings) -> bool {
    settings.process.print.print_sequence == ProcessPrintSequence::ByObject
}

/// `init_arrange_params` with the object skirt offset of the current plate's print. For the
/// current plate (`PREPARE_STATE_MENU`, `ArrangeJob.cpp:1026-1034`) `plate_sequence` is the
/// plate's real print sequence: it decides the sequential print, and the distance turns
/// automatic when it differs from the global one.
pub(super) fn arrange_params(
    settings: &ProjectSettings,
    arrange: ArrangeSettings,
    object_skirt_offset: f32,
    plate_sequence: Option<ProcessPrintSequence>,
) -> ArrangeParams {
    let printer = &settings.printer.remaining;
    let mut params = ArrangeParams {
        clearance_height_to_rod: printer.extruder_clearance_height_to_rod.0 as f32,
        clearance_height_to_lid: printer.extruder_clearance_height_to_lid.0 as f32,
        clearance_radius: printer.extruder_clearance_radius.0 as f32 + object_skirt_offset * 2.0,
        object_skirt_offset,
        printable_height: printer.printable_height.0 as f32,
        allow_rotations: arrange.enable_rotation,
        nozzle_height: printer.nozzle_height.0 as f32,
        align_center: [printer.best_object_pos.x, printer.best_object_pos.y],
        allow_multi_materials_on_same_plate: arrange.allow_multi_materials_on_same_plate,
        avoid_extrusion_cali_region: arrange.avoid_extrusion_cali_region,
        is_seq_print: is_seq_print(settings),
        // `scaled(float)`: the division runs in float.
        min_obj_distance: (arrange.distance / SCALING_FACTOR as f32) as i64,
        align_to_y_axis: arrange.align_to_y_axis,
        ..ArrangeParams::default()
    };
    if let Some(sequence) = plate_sequence {
        params.is_seq_print = sequence == ProcessPrintSequence::ByObject;
        if sequence != settings.process.print.print_sequence {
            params.min_obj_distance = 0;
        }
    }
    if params.is_seq_print {
        params.bed_shrink_x = BED_SHRINK_SEQ_PRINT;
        params.bed_shrink_y = BED_SHRINK_SEQ_PRINT;
    }
    params
}

/// `to_points` (`PrintConfig.cpp:13812-13818`): each point scaled and truncated.
fn to_points(points: &[crate::options::Point2d]) -> Vec<Point> {
    points
        .iter()
        .map(|p| Point::new(scaled(p.x), scaled(p.y)))
        .collect()
}

/// `get_bed_shape(cfg, true)` (`PrintConfig.cpp:13837-13864`): the nozzle areas' shared polygon,
/// otherwise the printable area turned counter-clockwise.
fn bed_shape(settings: &ProjectSettings) -> Result<Vec<Point>, SliceError> {
    let printer = &settings.printer.remaining;
    match printer.extruder_printable_area.0.as_slice() {
        [] => {
            let mut area = printer.printable_area.0.clone();
            if !Polygon::new(to_points(&area)).is_counter_clockwise() {
                area.reverse();
            }
            Ok(to_points(&area))
        }
        [first, rest @ ..] => shared_poly(first, rest),
    }
}

/// `get_shared_poly` (`PrintConfig.cpp:13820-13835`): the first area, intersected with each next
/// one as `intersection(next, result)`, keeping each intersection's first polygon. Upstream indexes
/// the first polygon of an empty intersection, which is undefined behaviour; the port refuses it.
fn shared_poly(
    first: &[crate::options::Point2d],
    rest: &[Vec<crate::options::Point2d>],
) -> Result<Vec<Point>, SliceError> {
    let mut result = Polygon::new(to_points(first));
    for area in rest {
        result = intersection_polygons_paths(&[Polygon::new(to_points(area))], &[result])
            .expect("nozzle area intersection")
            .into_iter()
            .next()
            .ok_or_else(|| {
                SliceError::InvalidInput("the nozzle areas share no printable area".to_owned())
            })?;
    }
    Ok(result.points().to_vec())
}

/// `get_real_skirt_dist`: the skirt distance and loops' width when the print has a skirt or a
/// draft shield.
fn real_skirt_distance(settings: &ProjectSettings) -> f64 {
    let print = &settings.process.print;
    let draft_shield = print.draft_shield != ProcessDraftShield::Disabled;
    if !((print.skirt_height.0 > 0 && print.skirt_loops.0 > 0) || draft_shield) {
        return 0.0;
    }
    let distance = print.skirt_distance.0 as f32;
    let mut loops = print.skirt_loops.0;
    if draft_shield && loops == 0 {
        loops = 1;
    }
    let raw = |value: FloatOrPercent| match value {
        FloatOrPercent::Float(value) => value as f32,
        FloatOrPercent::Percent(value) => value.0 as f32,
    };
    let mut width = raw(print.initial_layer_line_width);
    if width <= 0.0 {
        width = raw(settings.process.object.line_width);
    }
    if width <= 0.0 {
        let nozzles = &settings.project.print.nozzle_diameter.0;
        width = if nozzles.is_empty() {
            0.4
        } else {
            nozzles.iter().map(|n| n.0).fold(f64::MIN, f64::max) as f32
        };
    }
    f64::from(distance + loops as f32 * width)
}

/// The config `update_arrange_params` and `get_shrink_bedpts` read.
pub(super) fn arrange_config(settings: &ProjectSettings) -> Result<ArrangeConfig, SliceError> {
    Ok(ArrangeConfig {
        bed_shape: bed_shape(settings)?,
        real_skirt_distance: real_skirt_distance(settings),
    })
}
