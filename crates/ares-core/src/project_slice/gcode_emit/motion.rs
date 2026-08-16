use super::super::island_print_order::{IslandPrintEntity, OrderedExtrusionLayer};

use crate::SliceError;

#[derive(Default)]
pub(super) struct EmitState {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) e: f64,
    pub(super) offset: (f64, f64),
}

#[expect(
    clippy::excessive_nesting,
    reason = "keeps the source ordered extrusion-entity traversal together"
)]
pub(super) fn emit_layer(
    output: &mut Vec<u8>,
    layer: &OrderedExtrusionLayer,
    scale: crate::geometry::CoordinateScale,
    state: &mut EmitState,
) -> Result<(), SliceError> {
    for island in &layer.islands {
        for entity in &island.entities {
            match entity {
                IslandPrintEntity::Perimeter(collection) => {
                    for loop_ in &collection.entities {
                        for path in &loop_.extrusion_loop.paths {
                            emit_path(output, path, scale, state);
                        }
                    }
                }
                IslandPrintEntity::Fill(collection) => {
                    for path in &collection.paths {
                        emit_polyline(output, &path.polyline, path.mm3_per_mm, scale, state);
                    }
                }
                IslandPrintEntity::Thin(entity) => match entity {
                    crate::project_slice::perimeters::classic::gap_extrusion::GapFillEntity::Path(path) => {
                        emit_path(output, path, scale, state);
                    }
                    crate::project_slice::perimeters::classic::gap_extrusion::GapFillEntity::Loop(paths) => {
                        for path in paths {
                            emit_path(output, path, scale, state);
                        }
                    }
                },
            }
        }
    }
    Ok(())
}

fn emit_path(
    output: &mut Vec<u8>,
    path: &crate::project_slice::perimeters::classic::materialize::ExtrusionPath,
    scale: crate::geometry::CoordinateScale,
    state: &mut EmitState,
) {
    emit_polyline3(output, &path.polyline.points, path.mm3_per_mm, scale, state);
}

fn emit_polyline(
    output: &mut Vec<u8>,
    polyline: &crate::geometry::Polyline,
    mm3_per_mm: f64,
    scale: crate::geometry::CoordinateScale,
    state: &mut EmitState,
) {
    emit_points(
        output,
        polyline.points().iter().map(|point| (point.x(), point.y())),
        mm3_per_mm,
        scale,
        state,
    );
}

fn emit_polyline3(
    output: &mut Vec<u8>,
    points: &[crate::project_slice::perimeters::classic::materialize::Point3],
    mm3_per_mm: f64,
    scale: crate::geometry::CoordinateScale,
    state: &mut EmitState,
) {
    emit_points(
        output,
        points.iter().map(|point| (point.x, point.y)),
        mm3_per_mm,
        scale,
        state,
    );
}

fn emit_points(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    mm3_per_mm: f64,
    scale: crate::geometry::CoordinateScale,
    state: &mut EmitState,
) {
    for (x, y) in points {
        let x = scale.unscale(x) + state.offset.0;
        let y = scale.unscale(y) + state.offset.1;
        let distance = (x - state.x).hypot(y - state.y);
        state.e += distance * mm3_per_mm;
        output.extend_from_slice(format!("G1 X{x:.5} Y{y:.5} E{:.5}\n", state.e).as_bytes());
        state.x = x;
        state.y = y;
    }
}
