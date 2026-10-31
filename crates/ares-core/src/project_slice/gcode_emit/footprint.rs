//! Per-instance object geometry for Klipper/Marlin object exclusion and
//! first-layer/timelapse bounds helpers.

use crate::geometry::convex_hull;
use crate::geometry::{CoordinateScale, Point, douglas_peucker};
use crate::project_slice::gcode_emit::{format_processor_float, tags::Tags};
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;
use crate::{Point3d, ProjectInstance, ProjectObject, ProjectVolumeType};

pub(super) type FirstLayerBounds = (f64, f64, f64, f64);

mod coverage;
pub(super) use coverage::covered_by_spacing;
mod islands;
pub(super) use islands::FirstLayerIslands;
pub(super) use islands::support_no_brim;
mod first_layer;
pub(super) use first_layer::{
    first_layer_bounds, first_layer_convex_hull, first_layer_object_center,
};

pub(super) fn model_bounds(
    traversal: &PreparedPostClassicTraversal,
) -> Option<(f64, f64, f64, f64)> {
    object_bounds(traversal.project.objects().first()?)
}

fn object_bounds(object: &ProjectObject) -> Option<(f64, f64, f64, f64)> {
    let instance_transform = object.instances().first()?.transform();
    let mut bounds = None::<(f64, f64, f64, f64)>;
    for volume in object
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
    {
        let transform = instance_transform.then(volume.transform());
        for &vertex in volume.mesh().vertices() {
            let point = transform.transform_point(vertex);
            bounds = Some(match bounds {
                Some((min_x, min_y, max_x, max_y)) => (
                    min_x.min(point.x),
                    min_y.min(point.y),
                    max_x.max(point.x),
                    max_y.max(point.y),
                ),
                None => (point.x, point.y, point.x, point.y),
            });
        }
    }
    bounds
}

/// `PrintApply.cpp:150`, `PrintObject.cpp:110,130`: scale translation and
/// raw model center separately, then add the integer instance shift.
pub(super) fn print_object_origin(
    traversal: &PreparedPostClassicTraversal,
    index: usize,
) -> (f64, f64) {
    let (resolved, group) = traversal.print_object_config(index);
    let source = &traversal.project.objects()[resolved.source_object_index];
    let center = crate::project_slice::centering::center(source, group, traversal.scale)
        .expect("sliced print object has validated raw bounds");
    let instance = &source.instances()[*group
        .instance_indices
        .first()
        .expect("print group has a member")];
    let shift = instance_shift(instance, center, traversal.scale);
    (
        traversal.scale.unscale(shift.x()),
        traversal.scale.unscale(shift.y()),
    )
}

pub(super) fn instance_shift(
    instance: &ProjectInstance,
    center: Point,
    scale: CoordinateScale,
) -> Point {
    let translation = instance
        .transform()
        .transform_point(Point3d::new(0.0, 0.0, 0.0));
    let x = scale
        .checked_scale(translation.x)
        .expect("resolved instance has a scaled X translation");
    let y = scale
        .checked_scale(translation.y)
        .expect("resolved instance has a scaled Y translation");
    Point::new(x + center.x(), y + center.y())
}

pub(super) const EXCLUDE_FLAVORS: [crate::GCodeFlavor; 4] = [
    crate::GCodeFlavor::Klipper,
    crate::GCodeFlavor::MarlinLegacy,
    crate::GCodeFlavor::MarlinFirmware,
    crate::GCodeFlavor::RepRapFirmware,
];

/// One `EXCLUDE_OBJECT_DEFINE` / `M486 S..` record per print instance
/// (`GCode.cpp:8084-8108`).
pub(super) struct InstanceDefinition {
    pub(super) name: String,
    pub(super) unique_id: usize,
    /// `CENTER=.. POLYGON=..` body for Klipper; `None` degenerates to a skip.
    pub(super) klipper_body: Option<String>,
}

pub(super) fn definitions(traversal: &PreparedPostClassicTraversal) -> Vec<InstanceDefinition> {
    let scale = traversal.scale;
    // Exclude-object definitions are plate-local like every emitted
    // coordinate (`set_gcode_offset`, Print.cpp:2636-2638).
    let plate_origin = traversal.project.plate_origin();
    let mut unique_id = 0;
    let mut output = Vec::new();
    for object_index in 0..traversal.objects.len() {
        let (resolved, group) = traversal.print_object_config(object_index);
        let object = &traversal.project.objects()[resolved.source_object_index];
        for (instance_index, &member) in group.instance_indices.iter().enumerate() {
            let instance = &object.instances()[member];
            output.push(InstanceDefinition {
                name: instance_name(object.name(), object_index as u32, instance_index as u32),
                unique_id,
                klipper_body: klipper_body(object, instance, scale, plate_origin),
            });
            unique_id += 1;
        }
    }
    output
}

impl InstanceDefinition {
    pub(super) fn append(&self, output: &mut Vec<u8>, klipper: bool, rrf: bool) {
        let text = if klipper {
            let Some(body) = &self.klipper_body else {
                return;
            };
            format!("EXCLUDE_OBJECT_DEFINE NAME={} {body}\n", self.name)
        } else if rrf {
            // RepRapFirmware takes the object name on the select line
            // (`GCode.cpp:8110-8113`).
            format!("M486 S{} A\"{}\"\nM486 S-1\n", self.unique_id, self.name)
        } else {
            format!("M486 S{}\nM486 A{}\nM486 S-1\n", self.unique_id, self.name)
        };
        output.extend_from_slice(text.as_bytes());
    }
}

/// Start/end label strings for the in-print object markers
/// (`GCode.cpp:5360-5372, 5478-5494`), or `None` when exclusion is disabled.
pub(super) fn in_print_labels(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    copy_index: usize,
) -> (Option<String>, Option<String>) {
    let settings = &traversal.resolved.views.full;
    if !EXCLUDE_FLAVORS.contains(&settings.printer.gcode.gcode_flavor)
        || !settings.process.print.exclude_object.0
        || Tags::of(traversal).is_bbl()
    {
        return (None, None);
    }
    let (resolved, _) = traversal.print_object_config(object_index);
    let object = &traversal.project.objects()[resolved.source_object_index];
    let preceding_instances = (0..object_index)
        .map(|index| {
            traversal
                .print_object_config(index)
                .1
                .instance_indices
                .len()
        })
        .sum::<usize>()
        + copy_index;
    let name = instance_name(object.name(), object_index as u32, copy_index as u32);
    let klipper = settings.printer.gcode.gcode_flavor == crate::GCodeFlavor::Klipper;
    let start = if klipper {
        format!("EXCLUDE_OBJECT_START NAME={name}\n")
    } else {
        format!("M486 S{preceding_instances}\n")
    };
    let end = if klipper {
        format!("EXCLUDE_OBJECT_END NAME={name}\n")
    } else {
        "M486 S-1\n".to_owned()
    };
    (Some(start), Some(end))
}

fn klipper_body(
    object: &ProjectObject,
    instance: &ProjectInstance,
    scale: CoordinateScale,
    plate_origin: (f64, f64),
) -> Option<String> {
    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    let mut points = Vec::new();
    for volume in object
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
    {
        let transform = instance.transform().then(volume.transform());
        // `ModelVolume::calculate_convex_hull_2d` (Model.cpp:2625-2644)
        // applies the instance transform WITHOUT its translation
        // (`get_matrix_no_offset`), scales with the C-cast truncating
        // `scaled()` — the rounding variant is commented out
        // (Point.hpp:665-670) — and translates the polygon by the
        // truncated scaled offset afterwards. Truncation toward zero
        // shifts negative rotated coordinates by +1 scaled unit, which
        // breaks exact collinearity for rotation-residue transforms and
        // the hull then keeps edge midpoints the uniform-translation
        // ordering would collapse.
        let matrix = transform.as_matrix();
        let offset = ((matrix[0][3] / 1e-6) as i64, (matrix[1][3] / 1e-6) as i64);
        for vertex in volume.mesh().vertices() {
            let rotated_x =
                matrix[0][0] * vertex.x + matrix[0][1] * vertex.y + matrix[0][2] * vertex.z;
            let rotated_y =
                matrix[1][0] * vertex.x + matrix[1][1] * vertex.y + matrix[1][2] * vertex.z;
            let x = (rotated_x / 1e-6) as i64 + offset.0;
            let y = (rotated_y / 1e-6) as i64 + offset.1;
            let transformed = crate::Point3d::new(scale.unscale(x), scale.unscale(y), vertex.z);
            if !transformed.x.is_finite() || !transformed.y.is_finite() {
                return None;
            }
            min[0] = min[0].min(transformed.x);
            min[1] = min[1].min(transformed.y);
            max[0] = max[0].max(transformed.x);
            max[1] = max[1].max(transformed.y);
            let x = scale.checked_scale(transformed.x)?;
            let y = scale.checked_scale(transformed.y)?;
            points.push(Point::new(x, y));
        }
    }
    if points.is_empty() {
        return None;
    }
    let hull = douglas_peucker(&convex_hull(&points), 0.1);
    if hull.is_empty() {
        return None;
    }
    let unscaled = |point: &Point| [scale.unscale(point.x()), scale.unscale(point.y())];
    // `polygon_to_string` renders through `translate_to_print_space`
    // (`unscale(p) - m_origin`, GCode.cpp:8059-8072, Print.cpp:3055-3062):
    // the plate origin subtracts from the rendered hull coordinates and
    // center, not from the hull input.
    let coordinate = |point: &Point| {
        let [x, y] = unscaled(point);
        format!(
            "[{},{}]",
            format_processor_float(x - plate_origin.0),
            format_processor_float(y - plate_origin.1)
        )
    };
    let polygon = hull
        .iter()
        .chain(hull.first())
        .map(coordinate)
        .collect::<Vec<_>>()
        .join(",");
    Some(format!(
        "CENTER={},{} POLYGON=[{polygon}]",
        format_processor_float((min[0] + max[0]) * 0.5 - plate_origin.0),
        format_processor_float((min[1] + max[1]) * 0.5 - plate_origin.1),
    ))
}

/// `get_instance_name` (`GCode.cpp:4378-4386`): sanitize the object name,
/// compose `<name>_id_<object>_copy_<instance>`, then sanitize again.
fn instance_name(object_name: &str, object_id: u32, instance_id: u32) -> String {
    let composed = format!(
        "{}_id_{object_id}_copy_{instance_id}",
        sanitize_instance_name(object_name)
    );
    sanitize_instance_name(&composed)
}

/// `sanitize_instance_name` (`GCode.cpp:4364-4375`): collapse runs of
/// printable-special characters into `_`, then strip edge underscores.
fn sanitize_instance_name(name: &str) -> String {
    const SPECIAL: [char; 22] = [
        ' ', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '=', '+', '[', ']', '{', '}', ';',
        ':', '"', ',', '\'',
    ];
    let mut output = String::with_capacity(name.len());
    let mut separator = false;
    for character in name.chars() {
        if SPECIAL.contains(&character) {
            if !separator {
                output.push('_');
            }
            separator = true;
        } else {
            separator = false;
            output.push(character);
        }
    }
    // Upstream erases at most one leading and one trailing underscore
    // (`GCode.cpp:4368-4375`).
    let stripped = output.strip_prefix('_').unwrap_or(&output);
    stripped.strip_suffix('_').unwrap_or(stripped).to_owned()
}
