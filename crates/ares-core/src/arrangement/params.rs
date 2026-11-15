//! The parameter, inflation and bed-exclusion updates of `Arrange.cpp` at `40ecc66f7`.

use super::{
    ArrangeConfig, ArrangeParams, ArrangePolygon, BoundingBox, Coord, MAX_OUTER_NOZZLE_DIAMETER,
    Point, Polygon,
};

use crate::geometry::{JoinType, offset_expolygon};

/// `EPSILON` (`libslic3r.h:52`).
const EPSILON: f64 = 1e-4;

/// `scaled<coord_t>(double)`: divide, then truncate.
fn scaled(params: &ArrangeParams, value: f64) -> Coord {
    (value / params.scaling_factor) as Coord
}

/// `scaled<coord_t>(float)`: the division runs in `float`.
fn scaled_f32(params: &ArrangeParams, value: f32) -> Coord {
    (value / params.scaling_factor as f32) as Coord
}

fn bed_bounds(config: &ArrangeConfig) -> BoundingBox {
    BoundingBox::from_polygon(&Polygon::new(config.bed_shape.clone())).expect("a bed shape")
}

/// `update_arrange_params` (`Arrange.cpp:150-158`): the bed keeps the skirt's room.
pub fn update_arrange_params(
    params: &mut ArrangeParams,
    config: &ArrangeConfig,
    _selected: &[ArrangePolygon],
) {
    params.brim_skirt_distance = config.real_skirt_distance as f32;
    params.bed_shrink_x += params.brim_skirt_distance;
    params.bed_shrink_y += params.brim_skirt_distance;
}

/// `update_selected_items_inflation` (`Arrange.cpp:160-226`).
pub fn update_selected_items_inflation(
    selected: &mut [ArrangePolygon],
    config: &ArrangeConfig,
    params: &mut ArrangeParams,
) {
    // A brim reaching the printable area's edge is fine, so measure against the full bed.
    let bed = bed_bounds(config);
    let bed_size = [bed.max().x() - bed.min().x(), bed.max().y() - bed.min().y()];
    if params.is_seq_print {
        let all_short = selected
            .iter()
            .all(|item| item.height < f64::from(params.nozzle_height));
        let floor = if all_short {
            f64::from((MAX_OUTER_NOZZLE_DIAMETER as f32).max(params.object_skirt_offset * 2.0))
                + 0.001
        } else {
            // +0.001 mm against a clearance check failing on rounding.
            f64::from(params.clearance_radius) + 0.001
        };
        params.min_obj_distance = params.min_obj_distance.max(scaled(params, floor));
    }
    let brim_max = selected
        .iter()
        .map(|item| item.brim_width)
        .fold(0.0, f64::max);
    let tree_support = selected.iter().any(|item| item.has_tree_support);
    for item in selected.iter_mut() {
        // An explicit distance first; with tree support the widest branch for all; else the
        // item's own brim.
        item.inflation = if params.min_obj_distance != 0 {
            params.min_obj_distance / 2
        } else if tree_support {
            scaled(params, brim_max / 2.0)
        } else {
            scaled(params, item.brim_width)
        };
        if params.min_obj_distance != 0 {
            continue;
        }
        // An item cannot be larger than the printable area, inflation included.
        let size = item_size(item);
        item.inflation = item
            .inflation
            .min((bed_size[0] - size[0]) / 2)
            .min((bed_size[1] - size[1]) / 2)
            .max(0);
    }
    // Give the bed margin back where the largest item needs it; the skirt keeps its room.
    if !selected.is_empty() {
        let [widest, deepest] = selected
            .iter()
            .map(|item| margin_size(item, params))
            .fold([0_i64; 2], |largest, size| {
                [largest[0].max(size[0]), largest[1].max(size[1])]
            });
        let margin = |bed: Coord, item: Coord| {
            (((bed - item) as f64 * params.scaling_factor) / 2.0).max(0.0)
        };
        let brim_skirt = f64::from(params.brim_skirt_distance);
        params.bed_shrink_x =
            f64::from(params.bed_shrink_x).min(brim_skirt.max(margin(bed_size[0], widest))) as f32;
        params.bed_shrink_y =
            f64::from(params.bed_shrink_y).min(brim_skirt.max(margin(bed_size[1], deepest))) as f32;
    }
    if params.is_seq_print {
        let growth = params.min_obj_distance as f32 * params.scaling_factor as f32 / 2.0;
        params.bed_shrink_x -= growth;
        params.bed_shrink_y -= growth;
    }
}

// Arrange.cpp:200-218: measure the square-join outline as it is packed, then remove spacing.
fn margin_size(item: &ArrangePolygon, params: &ArrangeParams) -> [Coord; 2] {
    if !params.is_seq_print {
        let [width, height] = item_size(item);
        return [width + 2 * item.inflation, height + 2 * item.inflation];
    }
    let inflated = offset_expolygon(&item.poly, item.inflation as f32, JoinType::Square, 3.0)
        .expect("offset of an arrangement shape");
    let mut outline = inflated
        .first()
        .map_or(item.poly.contour(), |shape| shape.contour())
        .clone();
    let (sin, cos) = item.rotation.sin_cos();
    outline.rotate(cos, sin);
    let bounds = BoundingBox::from_polygon(&outline).expect("an item outline");
    let inflation = scaled(params, EPSILON)
        - if inflated.is_empty() {
            0
        } else {
            item.inflation
        };
    [
        bounds.max().x() - bounds.min().x() + 2 * inflation,
        bounds.max().y() - bounds.min().y() + 2 * inflation,
    ]
}

fn item_size(item: &ArrangePolygon) -> [Coord; 2] {
    let bounds = BoundingBox::from_polygon(item.poly.contour()).expect("an item outline");
    [
        bounds.max().x() - bounds.min().x(),
        bounds.max().y() - bounds.min().y(),
    ]
}

/// `update_unselected_items_inflation` (`Arrange.cpp:228-246`): fixed parts keep their brim or
/// half the object distance; virtual regions take the exclusion gap, except the extrusion
/// calibration region, which a brim may overlap.
pub fn update_unselected_items_inflation(
    unselected: &mut [ArrangePolygon],
    _config: &ArrangeConfig,
    params: &ArrangeParams,
) {
    let mut exclusion_gap = 1.0_f32;
    if params.is_seq_print {
        let growth = params.min_obj_distance as f32 * params.scaling_factor as f32 / 2.0;
        exclusion_gap = exclusion_gap.max(growth + params.bed_shrink_x + 1.0);
        let shift = Point::new(
            scaled_f32(params, params.bed_shrink_x.min(0.0)),
            scaled_f32(params, params.bed_shrink_y.min(0.0)),
        );
        for region in unselected.iter_mut().filter(|region| region.is_virt_object) {
            region.poly = translated(&region.poly, shift);
        }
    }
    for item in unselected.iter_mut() {
        let inflation = if !item.is_virt_object {
            if params.min_obj_distance == 0 {
                (item.brim_width / params.scaling_factor).trunc()
            } else {
                (params.min_obj_distance / 2) as f64
            }
        } else if item.is_extrusion_cali_object {
            0.0
        } else {
            f64::from(exclusion_gap) / params.scaling_factor
        };
        item.inflation = inflation as Coord;
    }
}

fn translated(poly: &super::ExPolygon, by: Point) -> super::ExPolygon {
    let shift = |polygon: &Polygon| {
        Polygon::new(
            polygon
                .points()
                .iter()
                .map(|point| Point::new(point.x() + by.x(), point.y() + by.y()))
                .collect(),
        )
    };
    super::ExPolygon::new(
        shift(poly.contour()),
        poly.holes().iter().map(shift).collect(),
    )
}

/// `update_selected_items_axis_align` (`Arrange.cpp:234-337`): turns each item so the dominant
/// axis of its second area moments runs along Y.
pub fn update_selected_items_axis_align(
    selected: &mut [ArrangePolygon],
    _config: &ArrangeConfig,
    params: &ArrangeParams,
) {
    if !params.align_to_y_axis {
        return;
    }
    for item in selected.iter_mut() {
        if let Some(angle) = y_axis_angle(item.transformed_poly().contour().points()) {
            item.rotation += angle;
        }
    }
}

fn y_axis_angle(points: &[Point]) -> Option<f64> {
    let [mut a00, mut a10, mut a01, mut a20, mut a11, mut a02] = [0.0_f64; 6];
    let last = points.last()?;
    let (mut xi_1, mut yi_1) = (last.x() as f64, last.y() as f64);
    for point in points {
        let (xi, yi) = (point.x() as f64, point.y() as f64);
        let dxy = xi_1 * yi - xi * yi_1;
        let (xii_1, yii_1) = (xi_1 + xi, yi_1 + yi);
        a00 += dxy;
        a10 += dxy * xii_1;
        a01 += dxy * yii_1;
        a20 += dxy * (xi_1 * xii_1 + xi * xi);
        a11 += dxy * (xi_1 * (yii_1 + yi_1) + xi * (yii_1 + yi));
        a02 += dxy * (yi_1 * yii_1 + yi * yi);
        (xi_1, yi_1) = (xi, yi);
    }
    if a00.abs() <= EPSILON {
        return None;
    }
    let sign = if a00 > 0.0 { 1.0 } else { -1.0 };
    let m00 = a00 * sign * 0.5;
    // The upstream literals `0.1666…67`, `0.0833…33`, `0.04166…67` are these doubles.
    let (m10, m01) = (a10 * sign * (1.0 / 6.0), a01 * sign * (1.0 / 6.0));
    let (m20, m11, m02) = (
        a20 * sign * (1.0 / 12.0),
        a11 * sign * (1.0 / 24.0),
        a02 * sign * (1.0 / 12.0),
    );
    let (cx, cy) = (m10 / m00, m01 / m00);
    let a = m20 / m00 - cx * cx;
    let b = m11 / m00 - cx * cy;
    let c = m02 / m00 - cy * cy;
    // Close second moments have no dominant axis; the ratio is at most 1.
    let ratio = if a.abs() > c.abs() {
        (c / a).abs()
    } else if c.abs() > 0.0 {
        (a / c).abs()
    } else {
        0.0
    };
    if ratio > 0.66 {
        return None;
    }
    let angle = std::f64::consts::FRAC_PI_2 - (2.0 * b).atan2(a - c) / 2.0;
    // Close to ±PI the item already stands along Y.
    Some(if (angle.abs() - std::f64::consts::PI).abs() < 0.01 {
        0.0
    } else {
        angle
    })
}

/// `get_shrink_bedpts` (`Arrange.cpp:339-353`): the bed points moved towards the bed centre by
/// the bed shrink.
pub fn get_shrink_bedpts(config: &ArrangeConfig, params: &ArrangeParams) -> Vec<Point> {
    let center = bed_bounds(config).center();
    let distance = [
        scaled_f32(params, params.bed_shrink_x),
        scaled_f32(params, params.bed_shrink_y),
    ];
    let toward = |from: Coord, to: Coord| if to - from >= 0 { 1 } else { -1 };
    config
        .bed_shape
        .iter()
        .map(|point| {
            Point::new(
                point.x() + distance[0] * toward(point.x(), center.x()),
                point.y() + distance[1] * toward(point.y(), center.y()),
            )
        })
        .collect()
}

/// Whether two Z ranges overlap; touching counts (`z_ranges_overlap`, `Arrange.cpp:108-118`).
fn z_ranges_overlap(first: [f64; 2], second: [f64; 2]) -> bool {
    let [first_min, first_max] = if first[0] > first[1] {
        [first[1], first[0]]
    } else {
        first
    };
    let [second_min, second_max] = if second[0] > second[1] {
        [second[1], second[0]]
    } else {
        second
    };
    first_max >= second_min - EPSILON && second_max >= first_min - EPSILON
}

/// `has_bed_exclusion_regions` (`Arrange.cpp:136-141`).
pub fn has_bed_exclusion_regions(polygons: &[ArrangePolygon]) -> bool {
    polygons.iter().any(|polygon| polygon.is_bed_exclusion)
}

/// `bed_exclusion_applies` (`Arrange.cpp:120-146`): a fixed exclusion applies unless both Z
/// ranges are known and apart; a shared exclusion or an item with unresolved nozzles is always
/// affected, otherwise only the owning nozzle's items are.
pub fn bed_exclusion_applies(item: &ArrangePolygon, fixed_item: &ArrangePolygon) -> bool {
    if !fixed_item.is_bed_exclusion {
        return false;
    }
    if item.has_z_range
        && fixed_item.has_z_range
        && !z_ranges_overlap(
            [item.z_min, item.z_max],
            [fixed_item.z_min, fixed_item.z_max],
        )
    {
        return false;
    }
    fixed_item.bed_exclusion_extruder_id < 0
        || item.bed_exclusion_extruder_ids.is_empty()
        || item
            .bed_exclusion_extruder_ids
            .contains(&fixed_item.bed_exclusion_extruder_id)
}
