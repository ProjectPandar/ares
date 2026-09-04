mod apply;
mod collision;
mod contour;
mod graph;
mod scale;
mod touching;
mod types;

use crate::geometry::{BoundingBox, ClipperError, CoordinateScale, ExPolygon, Polygon, Polyline};
use apply::apply_connections;
use graph::build_working_graph;
use scale::{scaled_epsilon, scaled_f32, scaled_f64};

pub(crate) struct FillBoundary<'a> {
    pub(crate) polygons: &'a [Polygon],
    pub(crate) bbox: BoundingBox,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FillConnectionParams {
    pub(crate) anchor_length: f32,
    pub(crate) anchor_length_max: f32,
    pub(crate) multiline: i32,
    pub(crate) dont_sort: bool,
}

pub(crate) fn connect_infill(
    infill_ordered: Vec<Polyline>,
    boundary: &ExPolygon,
    spacing: f64,
    params: FillConnectionParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    debug_assert!(!infill_ordered.is_empty());
    debug_assert!(infill_ordered.iter().all(Polyline::is_valid));
    debug_assert!(!boundary.contour().points().is_empty());
    debug_assert!(spacing.is_finite() && spacing > 0.0);
    debug_assert!(params.anchor_length >= 0.0);
    debug_assert!(params.anchor_length_max >= 0.01);
    debug_assert!(params.anchor_length_max >= params.anchor_length);
    debug_assert!(params.multiline >= 1);

    let mut boundaries = Vec::with_capacity(boundary.holes().len() + 1);
    boundaries.push(boundary.contour().clone());
    boundaries.extend(boundary.holes().iter().cloned());
    let bbox =
        BoundingBox::from_expolygon(boundary).expect("the fill boundary contour must be nonempty");
    connect_infill_polygons(
        infill_ordered,
        FillBoundary {
            polygons: &boundaries,
            bbox,
        },
        spacing,
        params,
        scale,
    )
}

pub(crate) fn connect_infill_polygons(
    infill_ordered: Vec<Polyline>,
    boundary: FillBoundary<'_>,
    spacing: f64,
    params: FillConnectionParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let mut dump = debug_dump();
    if let Some(dump) = dump.as_mut() {
        dump_polygons_input(dump, &infill_ordered, &boundary, params, scale);
    }
    let anchor_length = scaled_f32(params.anchor_length, scale);
    let anchor_length_max = scaled_f32(params.anchor_length_max, scale);
    let scaled_spacing = scaled_f64(spacing, scale);
    let epsilon = scaled_epsilon(scale);
    let graph = build_working_graph(
        infill_ordered,
        boundary.polygons,
        boundary.bbox,
        spacing,
        scale,
    )?;
    let out = apply_connections(
        graph,
        anchor_length,
        anchor_length_max,
        scaled_spacing,
        params.multiline,
        params.dont_sort,
        epsilon,
    );
    if let Some(dump) = dump.as_mut() {
        use std::io::Write;
        for polyline in &out {
            let _ = write!(dump, "O {}:", polyline.points().len());
            for point in polyline.points() {
                let _ = write!(
                    dump,
                    " ({:.6},{:.6})",
                    scale.unscale(point.x()),
                    scale.unscale(point.y())
                );
            }
            let _ = writeln!(dump);
        }
    }
    Ok(out)
}

fn debug_dump() -> Option<std::fs::File> {
    std::env::var("ARES_DUMP_FCONNECT")
        .ok()
        .map(|path| {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .expect("ARES_DUMP_FCONNECT path is writable");
            let _ = writeln!(file, "SECTION");
            Some(file)
        })
        .flatten()
}

fn dump_polygons_input(
    file: &mut std::fs::File,
    infill_ordered: &[Polyline],
    boundary: &FillBoundary<'_>,
    params: FillConnectionParams,
    scale: CoordinateScale,
) {
    use std::io::Write;
    let _ = writeln!(
        file,
        "IN n={} dont_connect={} dont_sort={} multiline={}",
        infill_ordered.len(),
        params.anchor_length_max < 0.05,
        params.dont_sort,
        params.multiline
    );
    for polyline in infill_ordered {
        let _ = writeln!(
            file,
            "I {:.6} {:.6} -> {:.6} {:.6}",
            scale.unscale(polyline.front().expect("valid").x()),
            scale.unscale(polyline.front().expect("valid").y()),
            scale.unscale(polyline.back().expect("valid").x()),
            scale.unscale(polyline.back().expect("valid").y())
        );
    }
    let _ = writeln!(file, "BOUND n={}", boundary.polygons.len());
    for polygon in boundary.polygons {
        for point in polygon.points() {
            let _ = writeln!(
                file,
                "B {:.6} {:.6}",
                scale.unscale(point.x()),
                scale.unscale(point.y())
            );
        }
    }
}

#[cfg(test)]
mod tests;
