//! An instance's arrange item: `get_arrange_poly` and `get_instance_arrange_poly`
//! (`ModelArrange.cpp:107-202` at v2.5.0-beta `345efec64`) over
//! `ModelInstance::get_arrange_polygon` (`Model.cpp:3920-3971`) and
//! `ModelObject::convex_hull_2d` (`:1819-1881`, `ModelVolume::calculate_convex_hull_2d`
//! `:2818-2844`).

use super::super::plate_list::InstanceKey;
use super::super::{selection, transformation};
use crate::arrangement::{ArrangePolygon, SCALING_FACTOR};
use crate::geometry::{ExPolygon, Point, Polygon, convex_hull};
use crate::options::{ObjectOptions, ProcessSupportType, ProjectBedType, ProjectSettings};
use crate::{Project, ProjectObject, ProjectVolumeType, Transform3d};

/// `MaterialType::all()` (`MaterialType.cpp:18-98`): each type's maximum nozzle temperature.
const MAX_TEMPERATURES: [(&str, i32); 75] = [
    ("ABS", 300),
    ("ABS-CF", 300),
    ("ABS-GF", 280),
    ("ASA", 300),
    ("ASA-CF", 300),
    ("ASA-GF", 300),
    ("ASA-AERO", 280),
    ("BVOH", 240),
    ("CoPE", 240),
    ("EVA", 220),
    ("FLEX", 230),
    ("HIPS", 270),
    ("PA", 280),
    ("PA-CF", 315),
    ("PA-GF", 290),
    ("PA6", 300),
    ("PA6-CF", 300),
    ("PA6-GF", 300),
    ("PA11", 295),
    ("PA11-CF", 295),
    ("PA11-GF", 295),
    ("PA12", 270),
    ("PA12-CF", 300),
    ("PA12-GF", 270),
    ("PAHT", 310),
    ("PAHT-CF", 310),
    ("PAHT-GF", 310),
    ("PC", 300),
    ("PC-ABS", 270),
    ("PC-CF", 295),
    ("PC-PBT", 300),
    ("PCL", 170),
    ("PCTG", 300),
    ("PE", 260),
    ("PE-CF", 260),
    ("PE-GF", 270),
    ("PEI-1010", 430),
    ("PEI-1010-CF", 430),
    ("PEI-1010-GF", 430),
    ("PEI-9085", 390),
    ("PEI-9085-CF", 390),
    ("PEI-9085-GF", 390),
    ("PEEK", 460),
    ("PEEK-CF", 410),
    ("PEEK-GF", 410),
    ("PEKK", 400),
    ("PEKK-CF", 400),
    ("PES", 390),
    ("PET", 290),
    ("PET-CF", 320),
    ("PET-GF", 320),
    ("PETG", 260),
    ("PETG-CF", 290),
    ("PETG-GF", 270),
    ("PHA", 250),
    ("PI", 410),
    ("PLA", 240),
    ("PLA-AERO", 270),
    ("PLA-CF", 250),
    ("POM", 250),
    ("PP", 240),
    ("PP-CF", 250),
    ("PP-GF", 260),
    ("PPA-CF", 300),
    ("PPA-GF", 290),
    ("PPS", 345),
    ("PPS-CF", 350),
    ("PPSU", 420),
    ("PSU", 380),
    ("PVA", 250),
    ("PVB", 250),
    ("PVDF", 265),
    ("SBS", 250),
    ("TPI", 445),
    ("TPU", 260),
];

/// `FilamentTempType` (`Print.hpp:1037-1042`).
const HIGH_TEMP: i32 = 0;
const LOW_TEMP: i32 = 1;
const HIGH_LOW_COMPATIBLE: i32 = 2;
const UNDEFINE: i32 = 3;

/// `ConfigOptionVector::get_at`: the value at `index`, else the first.
fn get_at<T: Copy>(values: &[T], index: usize) -> T {
    *values.get(index).unwrap_or(&values[0])
}

/// `scale_(value)` truncated to `coord_t`.
fn scaled(value: f64) -> i64 {
    (value / SCALING_FACTOR) as i64
}

/// `Print::get_filament_temp_type` (`Print.cpp:4554-4567`).
fn filament_temp_type(filament_type: &str) -> i32 {
    match MAX_TEMPERATURES
        .iter()
        .find(|(name, _)| *name == filament_type)
    {
        Some((_, max)) if *max <= 250 => LOW_TEMP,
        Some((_, max)) if *max < 280 => HIGH_LOW_COMPATIBLE,
        Some(_) => HIGH_TEMP,
        None => UNDEFINE,
    }
}

/// `Print::get_compatible_filament_type` (`Print.cpp:1814-1832`).
fn compatible_filament_type(types: &std::collections::BTreeSet<i32>) -> i32 {
    match (types.contains(&HIGH_TEMP), types.contains(&LOW_TEMP)) {
        (true, true) => HIGH_LOW_COMPATIBLE,
        (true, false) => HIGH_TEMP,
        (false, true) => LOW_TEMP,
        (false, false) => HIGH_LOW_COMPATIBLE,
    }
}

fn model_parts(object: &ProjectObject) -> impl Iterator<Item = &crate::ProjectVolume> {
    object
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
}

/// `ModelObject::convex_hull_2d(trafo_instance)`: each part's hull of its vertices through the
/// matrix without offset, moved by the offset, then the hull of all parts.
fn convex_hull_2d(object: &ProjectObject, instance: Transform3d) -> Polygon {
    let mut points = Vec::new();
    for volume in model_parts(object) {
        let matrix = instance.then(volume.transform());
        let linear = transformation::without_offset(matrix);
        let projected: Vec<Point> = volume
            .mesh()
            .vertices()
            .iter()
            .map(|vertex| {
                let p = linear.transform_point(*vertex);
                Point::new(scaled(p.x), scaled(p.y))
            })
            .collect();
        let offset = matrix.as_matrix();
        let shift = Point::new(scaled(offset[0][3]), scaled(offset[1][3]));
        points.extend(
            convex_hull(&projected)
                .into_iter()
                .map(|point| Point::new(point.x() + shift.x(), point.y() + shift.y())),
        );
    }
    Polygon::new(convex_hull(&points))
}

/// `ModelVolume::get_extruders` for a model part (`Model.cpp:2713-2742`): the painted
/// filaments, then the part's extruder (`extruder_id`, `:2690-2701`).
fn volume_extruders(object: &ProjectObject, volume: &crate::ProjectVolume) -> Vec<i32> {
    let painted = volume.mesh().mmu_segmentation_facets.used_states();
    let mut extruders: Vec<i32> = (1..painted.len())
        .filter(|&state| painted[state])
        .map(|state| state as i32)
        .collect();
    let extruder = volume
        .region_overrides()
        .extruder
        .filter(|value| value.0 != 0)
        .or(object.region_overrides().extruder)
        .map_or(1, |value| value.0);
    if extruder > 0 {
        extruders.push(extruder);
    }
    extruders
}

/// The object's options over the global ones (`ModelObject::get_config_value`).
fn object_options(settings: &ProjectSettings, object: &ProjectObject) -> ObjectOptions {
    ObjectOptions::overlay(&settings.process.object, object.object_overrides())
}

/// `get_bed_temp_key` / `get_bed_temp_1st_layer_key` (`PrintConfig.hpp:732-776`); the default
/// plate has none.
fn bed_temperatures(settings: &ProjectSettings) -> Option<(&[crate::OrcaInt], &[crate::OrcaInt])> {
    let print = &settings.filament.print;
    let (other, first) = match settings.project.print.curr_bed_type {
        ProjectBedType::SupertackPlate => (
            &print.supertack_plate_temp,
            &print.supertack_plate_temp_initial_layer,
        ),
        ProjectBedType::CoolPlate => (&print.cool_plate_temp, &print.cool_plate_temp_initial_layer),
        ProjectBedType::TexturedCoolPlate => (
            &print.textured_cool_plate_temp,
            &print.textured_cool_plate_temp_initial_layer,
        ),
        ProjectBedType::EngineeringPlate => {
            (&print.eng_plate_temp, &print.eng_plate_temp_initial_layer)
        }
        ProjectBedType::HighTempPlate => {
            (&print.hot_plate_temp, &print.hot_plate_temp_initial_layer)
        }
        ProjectBedType::TexturedPeiPlate => (
            &print.textured_plate_temp,
            &print.textured_plate_temp_initial_layer,
        ),
        ProjectBedType::DefaultPlate => return None,
    };
    Some((&other.0, &first.0))
}

/// `get_instance_arrange_poly` of the instance `key`; `bed_idx` 0 keeps an unarranged item's
/// transformation.
pub(super) fn instance_arrange_polygon(project: &Project, key: InstanceKey) -> ArrangePolygon {
    let settings = project.settings();
    let object = &project.objects()[key.0];
    let transform = object.instances()[key.1].transform();
    let mut rotation = transformation::rotation(transform);
    rotation[2] = 0.0;
    let offset = transform.as_matrix();
    let hull_frame = transformation::with_rotation(
        transformation::with_offset(transform, [0.0, 0.0, offset[2][3]]),
        rotation,
    );
    let mut ap = ArrangePolygon {
        poly: ExPolygon::new(convex_hull_2d(object, hull_frame), Vec::new()),
        translation: Point::new(scaled(offset[0][3]), scaled(offset[1][3])),
        rotation: transformation::rotation(transform)[2],
        bed_idx: 0,
        ..ArrangePolygon::default()
    };
    for volume in model_parts(object) {
        ap.extrude_ids.extend(volume_extruders(object, volume));
    }
    let options = object_options(settings, object);
    if options.enable_support.0 {
        for filament in [
            options.support_filament.0,
            options.support_interface_filament.0,
        ] {
            if filament > 0 {
                ap.extrude_ids.push(filament);
            }
        }
    }
    ap.extrude_ids.dedup();
    if ap.extrude_ids.is_empty() {
        ap.extrude_ids.push(1);
    }
    let first = (ap.extrude_ids[0] - 1) as usize;
    if let Some((other, initial)) = bed_temperatures(settings) {
        ap.bed_temp = get_at(other, first).0;
        ap.first_bed_temp = get_at(initial, first).0;
    }
    let filament = &settings.filament;
    ap.print_temp = get_at(&filament.print.nozzle_temperature.0, first).0;
    ap.first_print_temp = get_at(&filament.print.nozzle_temperature_initial_layer.0, first).0;
    ap.vitrify_temp = get_at(&filament.gcode.temperature_vitrification.0, first).0;
    let types = &filament.gcode.filament_type.0;
    let temp_types = ap
        .extrude_ids
        .iter()
        .map(|&id| filament_temp_type(get_at_str(types, (id - 1) as usize)))
        .collect();
    ap.filament_temp_type = compatible_filament_type(&temp_types);
    ap.brim_width = 1.0;
    if options.enable_support.0 {
        if matches!(
            options.support_type,
            ProcessSupportType::NormalAuto | ProcessSupportType::NormalManual
        ) {
            ap.brim_width = 6.0;
        } else {
            ap.brim_width = 24.0;
            ap.has_tree_support = true;
        }
    }
    let (min, max) = selection::instance_bounding_box(project, key);
    ap.height = max[2] - min[2];
    ap.z_min = min[2];
    ap.z_max = max[2];
    ap.has_z_range = model_parts(object).next().is_some();
    ap.name = object.name().to_owned();
    ap
}

fn get_at_str(values: &[String], index: usize) -> &str {
    values.get(index).unwrap_or(&values[0])
}
