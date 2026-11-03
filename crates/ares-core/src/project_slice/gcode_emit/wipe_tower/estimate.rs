//! The prime tower size estimate the GUI and CLI place a tower by (`GCode/WipeTowerEstimate.cpp`
//! at v2.5.0-beta `345efec64`), with the mixed-filament expansion it calls
//! (`FilamentMixer.cpp:172-187, 486-538`).

pub(crate) mod config;
mod statics;

use std::f64::consts::PI;

use self::config::{DynamicPrintConfig, get_at};
pub(crate) use self::statics::{align_ceil, auto_brim_by_height, limit_depth_by_height};
use crate::geometry::{
    ExPolygon, FillRule, Point, Polygon, convex_hull, difference_ex_polygons, point_in_polygon,
    union_ex,
};

/// `EPSILON` (`libslic3r.h`).
pub(crate) const EPSILON: f64 = 1e-4;
/// `SCALING_FACTOR` (`libslic3r.h`).
const SCALING_FACTOR: f64 = 1e-6;

/// `WipeTowerType`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WipeTowerType {
    Type1,
    Type2,
}

/// `WipeTowerFootprint`: the tower body, its height and the resolved brim width, in mm.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Footprint {
    pub(crate) width: f64,
    pub(crate) depth: f64,
    pub(crate) height: f64,
    pub(crate) brim_width: f64,
}

/// `Point::new_scale`: truncated.
pub(crate) fn new_scale(x: f64, y: f64) -> Point {
    Point::new((x / SCALING_FACTOR) as i64, (y / SCALING_FACTOR) as i64)
}

/// `parse_mixed_components`: the comma-separated non-negative ids that parse.
fn parse_mixed_components(text: &str) -> Vec<u32> {
    if text.is_empty() {
        return Vec::new();
    }
    text.split(',')
        .filter_map(|token| cxx_stoi(token).filter(|value| *value >= 0))
        .map(|value| value as u32)
        .collect()
}

/// `std::stoi`: optional leading whitespace and sign, then at least one digit.
fn cxx_stoi(token: &str) -> Option<i64> {
    let token = token.trim_start();
    let (sign, digits) = match token.as_bytes().first() {
        Some(b'-') => (-1, &token[1..]),
        Some(b'+') => (1, &token[1..]),
        _ => (1, token),
    };
    let end = digits.bytes().take_while(u8::is_ascii_digit).count();
    digits[..end].parse::<i64>().ok().map(|value| sign * value)
}

/// `has_any_mixed_filament`.
pub(crate) fn has_any_mixed_filament(is_mixed: &[bool]) -> bool {
    is_mixed.iter().any(|&mixed| mixed)
}

/// `expand_mixed_filaments`: each mixed slot replaced by its physical components, sorted and
/// deduplicated.
pub(crate) fn expand_mixed_filaments(
    extruders: &[u32],
    is_mixed: &[bool],
    components: &[String],
) -> Vec<u32> {
    let mut result = Vec::new();
    for &ext in extruders {
        let slot = ext as usize;
        if slot < is_mixed.len() && is_mixed[slot] && slot < components.len() {
            result.extend(
                parse_mixed_components(&components[slot])
                    .into_iter()
                    .filter(|&c| c >= 1)
                    .map(|c| c - 1),
            );
        } else {
            result.push(ext);
        }
    }
    result.sort_unstable();
    result.dedup();
    result
}

/// `resolve_wipe_tower_type` (`:62-73`).
pub(crate) fn resolve_wipe_tower_type(config: &DynamicPrintConfig) -> WipeTowerType {
    if config
        .option("printer_model")
        .text()
        .starts_with("Bambu Lab")
    {
        return WipeTowerType::Type1;
    }
    match config.option("wipe_tower_type").key() {
        "type1" => WipeTowerType::Type1,
        _ => WipeTowerType::Type2,
    }
}

/// `estimate_wipe_tower_first_layer_outline` (`:75-84`) with `WipeTower2::cone_base_polygon`
/// (`WipeTower2.cpp:2167-2185`).
pub(crate) fn first_layer_outline(
    config: &DynamicPrintConfig,
    tower_type: WipeTowerType,
    (width, depth, height): (f64, f64, f64),
) -> Polygon {
    let cone =
        tower_type == WipeTowerType::Type2 && config.option("wipe_tower_wall_type").key() == "cone";
    let angle = if cone {
        config.option("wipe_tower_cone_angle").float()
    } else {
        0.0
    };
    let tower = Polygon::new(vec![
        new_scale(0.0, 0.0),
        new_scale(width, 0.0),
        new_scale(width, depth),
        new_scale(0.0, depth),
    ]);
    if angle <= EPSILON || height <= EPSILON || width <= EPSILON || depth <= EPSILON {
        return tower;
    }
    let (radius, x_scale) = super::geometry::cone_base(width, height, depth, angle);
    if radius <= EPSILON {
        return tower;
    }
    let center = (width / 2.0, depth / 2.0);
    let mut ellipse = Vec::new();
    let mut alpha = 0.0_f64;
    while alpha < 2.0 * PI {
        ellipse.push(new_scale(
            center.0 + radius * (alpha.cos() / x_scale),
            center.1 + radius * alpha.sin(),
        ));
        alpha += PI / 20.0;
    }
    // `union_`, whose first path is the outer contour.
    union_ex(&[tower.clone(), Polygon::new(ellipse)], FillRule::NonZero)
        .ok()
        .and_then(|union| union.into_iter().next())
        .map_or(tower, |shape| shape.contour().clone())
}

mod footprint;
pub(crate) use footprint::estimate_footprint;

/// `line_alg::distance_to_squared` with the nearest point (`Line.hpp:46-71`).
fn distance_to_squared((a, b): (Point, Point), point: Point) -> (f64, Point) {
    let v = ((b.x() - a.x()) as f64, (b.y() - a.y()) as f64);
    let va = ((point.x() - a.x()) as f64, (point.y() - a.y()) as f64);
    let l2 = v.0 * v.0 + v.1 * v.1;
    let squared = |x: f64, y: f64| x * x + y * y;
    if l2 == 0.0 {
        return (squared(va.0, va.1), a);
    }
    let t = (va.0 * v.0 + va.1 * v.1) / l2;
    if t <= 0.0 {
        return (squared(va.0, va.1), a);
    }
    if t >= 1.0 {
        return (
            squared((point.x() - b.x()) as f64, (point.y() - b.y()) as f64),
            b,
        );
    }
    let nearest = Point::new(
        (a.x() as f64 + t * v.0) as i64,
        (a.y() as f64 + t * v.1) as i64,
    );
    (squared(t * v.0 - va.0, t * v.1 - va.1), nearest)
}

/// `to_lines(ExPolygon)`: the contour's closed edges, then each hole's.
fn to_lines(shape: &ExPolygon) -> Vec<(Point, Point)> {
    std::iter::once(shape.contour())
        .chain(shape.holes())
        .flat_map(|ring| {
            let points = ring.points();
            (0..points.len()).map(move |index| (points[index], points[(index + 1) % points.len()]))
        })
        .collect()
}

/// `ExPolygon::contains(point)`: inside or on the contour, not strictly inside a hole.
fn expolygon_contains(shape: &ExPolygon, point: Point) -> bool {
    point_in_polygon(point, shape.contour().points()) != 0
        && shape
            .holes()
            .iter()
            .all(|hole| point_in_polygon(point, hole.points()) != 1)
}

/// `nearest_clear_wipe_tower_anchor` (`:247-285`): the anchor in `anchor_bounds` nearest
/// `target` that puts `footprint` on no obstacle.
pub(crate) fn nearest_clear_anchor(
    anchor_bounds: (Point, Point),
    footprint: &Polygon,
    obstacles: &[Polygon],
    target: Point,
) -> Option<Point> {
    let tower = convex_hull(footprint.points());
    let blocked: Vec<Polygon> = obstacles
        .iter()
        .map(|obstacle| {
            let differences: Vec<Point> = obstacle
                .points()
                .iter()
                .flat_map(|o| {
                    tower
                        .iter()
                        .map(move |f| Point::new(o.x() - f.x(), o.y() - f.y()))
                })
                .collect();
            Polygon::new(convex_hull(&differences))
        })
        .collect();
    // `bounds.offset(SCALED_EPSILON)`.
    let grow = (EPSILON / SCALING_FACTOR) as i64;
    let (min, max) = (
        Point::new(anchor_bounds.0.x() - grow, anchor_bounds.0.y() - grow),
        Point::new(anchor_bounds.1.x() + grow, anchor_bounds.1.y() + grow),
    );
    let bounds = Polygon::new(vec![
        min,
        Point::new(max.x(), min.y()),
        max,
        Point::new(min.x(), max.y()),
    ]);
    let clear = difference_ex_polygons(&[ExPolygon::new(bounds, Vec::new())], &blocked)
        .expect("clear anchors");
    let mut nearest = None;
    let mut nearest_distance = f64::MAX;
    for region in &clear {
        if expolygon_contains(region, target) {
            return Some(target);
        }
        for edge in to_lines(region) {
            let (distance, closest) = distance_to_squared(edge, target);
            if distance < nearest_distance {
                nearest_distance = distance;
                nearest = Some(closest);
            }
        }
    }
    nearest
}

/// Values read off the config by `option_of`.
fn opt_float(config: &DynamicPrintConfig, key: &str) -> f64 {
    config.option(key).float()
}

/// `float_at`: the element for `id` of a non-empty vector, else `fallback`.
fn float_at(config: &DynamicPrintConfig, key: &str, id: u32, fallback: f64) -> f64 {
    let option = config.option(key);
    let values = option.floats();
    if values.is_empty() {
        fallback
    } else {
        get_at(values, id as usize)
    }
}

/// `int_at`.
fn int_at(config: &DynamicPrintConfig, key: &str, id: u32, fallback: i64) -> i64 {
    let option = config.option(key);
    let values = option.ints();
    if values.is_empty() {
        fallback
    } else {
        get_at(values, id as usize)
    }
}

/// `max_of`.
fn max_of(config: &DynamicPrintConfig, key: &str, fallback: f64) -> f64 {
    let option = config.option(key);
    option
        .floats()
        .iter()
        .copied()
        .reduce(f64::max)
        .unwrap_or(fallback)
}
