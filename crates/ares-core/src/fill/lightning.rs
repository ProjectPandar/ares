//! `FillLightning` port (`src/libslic3r/Fill/Lightning/*`): the lightning
//! infill generator — a per-object tree grown top-down through the layer stack
//! then converted to lines per surface. `DistanceField` samples the overhang;
//! `Layer`/`TreeNode` build + propagate the support tree; `Generator`
//! orchestrates across all layers.

mod distance_field;
mod tree_geom;
mod tree_node;
mod tree_ops;

/// `Line::distance_to_squared` — squared distance from `point` to segment a-b.
pub(super) fn segment_distance_squared(
    point: crate::geometry::Point,
    a: crate::geometry::Point,
    b: crate::geometry::Point,
) -> f64 {
    let (px, py) = (point.x() as f64, point.y() as f64);
    let (ax, ay) = (a.x() as f64, a.y() as f64);
    let (bx, by) = (b.x() as f64, b.y() as f64);
    let (dx, dy) = (bx - ax, by - ay);
    let len2 = dx * dx + dy * dy;
    if len2 == 0.0 {
        let (ex, ey) = (px - ax, py - ay);
        return ex * ex + ey * ey;
    }
    let t = (((px - ax) * dx + (py - ay) * dy) / len2).clamp(0.0, 1.0);
    let (ex, ey) = (px - (ax + t * dx), py - (ay + t * dy));
    ex * ex + ey * ey
}
pub(crate) mod generator;
mod layer;
mod layer_geom;
