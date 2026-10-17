//! `create_polyholes` (`PrintObject.cpp:160-186`): regular
//! circumscribed polygon sized so a metric screw slides through the
//! inscribed circle — `radius / cos(PI / n)` with
//! `n = max(3, round(4 * unscaled(radius) * 0.4 / unscaled(nozzle)))`
//! (the 0.4 reference nozzle makes the Hydraraptor oversizing
//! nozzle-independent). `multiple` (`hole_to_polyhole_twisted`) builds
//! five variants rotated by `2*PI/(n*5)` each, interleaved so layer
//! `z % 5` rotates by `z` steps.

use crate::geometry::{CoordinateScale, Point, Polygon};

/// Port of `PrintObject.cpp:160-186`. `radius`/`nozzle_diameter` are
/// scaled `coord_t` (`scale_(...)`) as the upstream call passes them;
/// `unscaled` conversions go through `scale` like `unscaled(coord)`.
pub(super) fn create_polyholes(
    center: Point,
    radius: i64,
    nozzle_diameter: i64,
    multiple: bool,
    scale: CoordinateScale,
) -> Vec<Polygon> {
    // `nb_edges` (:162): `max(3, round(4 * radius_mm * 0.4 / nozzle_mm))`.
    let radius_mm = scale.unscale(radius);
    let nozzle_mm = scale.unscale(nozzle_diameter);
    let nb_edges = (4.0 * radius_mm * 0.4 / nozzle_mm).round().max(3.0) as usize;

    // (:165-170) — five pre-rotated variants when `twisted`.
    let (nb_polyhole, rotation) = if multiple {
        (5usize, 2.0 * std::f32::consts::PI / (nb_edges * 5) as f32)
    } else {
        (1usize, 0.0f32)
    };

    let mut list = vec![Polygon::new(Vec::new()); nb_polyhole];
    for i_poly in 0..nb_polyhole {
        // (:173-174) interleave: even i → first half, odd i → second
        // half — for nb=5 the fill order is list[0],list[3],list[1],
        // list[4],list[2].
        let slot = if i_poly % 2 == 0 {
            i_poly / 2
        } else {
            nb_polyhole.div_ceil(2) + i_poly / 2
        };
        // (:175) circumscribed radius — `radius` is coord_t, widened
        // through float exactly as upstream `radius / cos(...)`.
        let new_radius = radius as f32 / (std::f32::consts::PI / nb_edges as f32).cos();
        let mut points = Vec::with_capacity(nb_edges);
        for i_edge in 0..nb_edges {
            // (:177-179) float angle math; `Point(double,double)` rounds.
            let angle = rotation * i_poly as f32
                + (std::f32::consts::PI * 2.0 * i_edge as f32) / nb_edges as f32;
            // coord_t + float is float arithmetic upstream; the
            // `Point(double,double)` ctor then rounds to `coord_t`.
            points.push(Point::new(
                (center.x() as f32 + new_radius * angle.cos()).round() as i64,
                (center.y() as f32 + new_radius * angle.sin()).round() as i64,
            ));
        }
        let mut polygon = Polygon::new(points);
        // (:181) `make_clockwise` — reverse when counter-clockwise.
        if polygon.is_counter_clockwise() {
            polygon.reverse();
        }
        list[slot] = polygon;
    }
    list
}
