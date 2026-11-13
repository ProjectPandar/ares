//! `Arrange.cpp`'s entry at `345efec64`, with `remove_items_outside_bin`
//! (`40ecc66f7:1110-1124`).

use super::arranger::{Arranger, ObjectiveParams};
use super::nest::geometry::NBox;
use super::nest::item::Item;
use super::nest::placer::PlacerConfig;
use super::nest::selection::pack_items;
use super::nest::{Alignment, Bin, Nest, ObjectFn};
use super::{ArrangeParams, ArrangePolygon, Bed};
use crate::geometry::{ExPolygon, Point};

/// `EPSILON` (`libslic3r.h:52`).
const EPSILON: f64 = 1e-4;

/// `process_arrangeable` (`Arrange.cpp:1221-1257`): the contour turned clockwise as an item, none
/// below three vertices; holes are dropped.
fn process_arrangeable(polygon: &ArrangePolygon) -> Option<Item> {
    let mut contour = polygon.poly.contour().clone();
    if contour.is_counter_clockwise() {
        contour.reverse();
    }
    if contour.points().len() < 3 {
        return None;
    }
    let mut item = Item::new(ExPolygon::new(contour, Vec::new()));
    item.set_rotation(polygon.rotation);
    item.set_translation(polygon.translation);
    item.bin_id = polygon.bed_idx;
    item.priority = polygon.priority;
    item.item_id = polygon.itemid;
    item.extrude_ids = polygon.extrude_ids.clone();
    item.height = polygon.height;
    item.z_min = polygon.z_min;
    item.z_max = polygon.z_max;
    item.has_z_range = polygon.has_z_range;
    item.is_bed_exclusion = polygon.is_bed_exclusion;
    item.bed_exclusion_extruder_ids = polygon.bed_exclusion_extruder_ids.clone();
    item.bed_exclusion_extruder_id = polygon.bed_exclusion_extruder_id;
    item.name = polygon.name.clone();
    item.is_virt_object = polygon.is_virt_object;
    item.is_wipe_tower = polygon.is_wipe_tower;
    item.bed_temp = f64::from(polygon.first_bed_temp);
    item.print_temp = f64::from(polygon.print_temp);
    item.vitrify_temp = f64::from(polygon.vitrify_temp);
    item.set_inflation(polygon.inflation);
    item.filament_temp_type = polygon.filament_temp_type;
    Some(item)
}

/// `to_nestbin` (`Arrange.cpp:1174-1182`).
fn to_nestbin(bed: &Bed) -> Bin {
    match bed {
        Bed::Box(bounds) => Bin::Box(NBox::new(bounds.min(), bounds.max())),
        Bed::Circle { center, radius } => Bin::Circle {
            center: *center,
            radius: *radius,
        },
        Bed::Polygon(polygon) => Bin::Polygon(ExPolygon::new(polygon.clone(), Vec::new())),
        Bed::Infinite { center } => {
            let reach = 1_i64 << 50;
            Bin::Box(NBox::new(
                Point::new(center.x() - reach, center.y() - reach),
                Point::new(center.x() + reach, center.y() + reach),
            ))
        }
    }
}

/// `scaled<coord_t>(double)`.
fn scaled(params: &ArrangeParams, value: f64) -> i64 {
    (value / params.scaling_factor) as i64
}

/// `arrangement::arrange` (`Arrange.cpp:1290-1316`) with `_arrange` (`:1124-1172`). Items below
/// three vertices are skipped and the results are written back by position, as upstream does.
pub(super) fn arrange(
    items: &mut [ArrangePolygon],
    excludes: &[ArrangePolygon],
    bed: &Bed,
    params: &ArrangeParams,
) {
    let shapes: Vec<Item> = items.iter().filter_map(process_arrangeable).collect();
    let mut fixed: Vec<Item> = excludes.iter().filter_map(process_arrangeable).collect();
    for item in &mut fixed {
        item.inflate(scaled(params, -2.0 * EPSILON));
    }
    let bin = to_nestbin(bed);
    // Arrange.cpp:1110-1124: retain fixed items that reach into or touch the bin bounds.
    let bin_bounds = bin.bounding_box();
    fixed.retain(|item| {
        let bounds = item.bounding_box();
        item.is_virt_object
            || (bounds.max.x() >= bin_bounds.min.x()
                && bounds.min.x() <= bin_bounds.max.x()
                && bounds.max.y() >= bin_bounds.min.y()
                && bounds.min.y() <= bin_bounds.max.y())
    });
    let (config, regions) = fill_config(&bin, params);
    let objective = ObjectiveParams {
        is_seq_print: params.is_seq_print,
        clearance_height_to_lid: f64::from(params.clearance_height_to_lid),
        clearance_height_to_rod: f64::from(params.clearance_height_to_rod),
        printable_height: f64::from(params.printable_height),
        allow_multi_materials_on_same_plate: params.allow_multi_materials_on_same_plate,
    };
    let mut arranger = Arranger::new(bin.clone(), config.starting_point, objective);
    let shape_count = shapes.len();
    let fixed_count = fixed.len();
    for item in &mut fixed {
        let bin_id = item.bin_id;
        item.mark_as_fixed_in_bin(bin_id);
    }
    arranger.item_count += fixed_count;
    let mut nest = Nest {
        items: shapes.into_iter().chain(fixed).collect(),
        arranger,
    };
    let region_base = nest.items.len();
    nest.items.extend(regions);
    let mut config = config;
    config.excluded_items = (region_base..nest.items.len()).collect();
    if params.allow_rotations {
        for index in 0..shape_count {
            super::rotcalipers::start_rotation(&mut nest.items[index], &bin);
        }
    }
    let inp: Vec<usize> = (0..shape_count + fixed_count).collect();
    nest.arranger.item_count += inp.len();
    pack_items(&mut nest, &inp, &bin, &config);
    for (item, placed) in items.iter_mut().zip(&nest.items[..shape_count]) {
        item.translation = placed.translation();
        item.rotation = placed.rotation();
        item.bed_idx = placed.bin_id;
        item.itemid = placed.item_id;
    }
}

/// `fill_config` and the `AutoArranger` constructor's configuration (`Arrange.cpp:356-414,
/// 857-870, 921-924`): the starting corner, alignment, rotations, the custom alignment point
/// and the excluded and non-preferred regions as items.
fn fill_config(bin: &Bin, params: &ArrangeParams) -> (PlacerConfig, Vec<Item>) {
    let starting_point = if params.is_seq_print {
        Alignment::BottomLeft
    } else {
        Alignment::TopRight
    };
    let mut alignment = if params.do_final_align {
        Alignment::Center
    } else {
        Alignment::DontAlign
    };
    let rotations = if params.allow_rotations {
        vec![
            0.0,
            std::f64::consts::FRAC_PI_4,
            std::f64::consts::FRAC_PI_2,
            3.0 * std::f64::consts::FRAC_PI_4,
        ]
    } else {
        vec![0.0]
    };
    let binbb = bin.bounding_box();
    let mut best_object_pos = Point::new(0, 0);
    let [ax, ay] = params.align_center;
    if (ax - 0.5).abs() > 0.001 || (ay - 0.5).abs() > 0.001 {
        best_object_pos = Point::new(
            binbb.min.x() + (binbb.width() as f64 * ax).round() as i64,
            binbb.min.y() + (binbb.height() as f64 * ay).round() as i64,
        );
        alignment = Alignment::UserDefined;
    }
    let origin_pack = match bin {
        Bin::Box(bounds) => match starting_point {
            Alignment::Center => bounds.center(),
            Alignment::TopRight => bounds.max,
            _ => bounds.min,
        },
        _ => {
            if starting_point == Alignment::Center {
                binbb.center()
            } else {
                binbb.min
            }
        }
    };
    let mut regions = Vec::new();
    for region in params
        .excluded_regions
        .iter()
        .filter_map(process_arrangeable)
    {
        let mut region = region;
        region.mark_as_fixed_in_bin(0);
        region.inflate(scaled(params, -2.0 * EPSILON));
        regions.push(region);
    }
    let config = PlacerConfig {
        rotations,
        alignment,
        starting_point,
        best_object_pos,
        object_fn: ObjectFn::Bin { origin_pack },
        accuracy: params.accuracy,
        excluded_items: Vec::new(),
    };
    (config, regions)
}
