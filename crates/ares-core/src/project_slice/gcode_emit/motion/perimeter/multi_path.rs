//! Ordered non-loop extrusion and joined wipe (`GCode.cpp:6038–6085`).

use super::{EmitState, LayerGeometry};
use crate::project_slice::perimeters::classic::materialize::ExtrusionPath;

pub(super) fn emit(
    output: &mut Vec<u8>,
    paths: &[ExtrusionPath],
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    for path in paths {
        super::super::materialized::emit_flat(output, path, 0.0, geometry, state);
    }
    if state.options.wipe {
        let mut wipe = Vec::new();
        for path in paths {
            let points = &path.polyline.points;
            let first = (points[0].x, points[0].y);
            let skip = usize::from(wipe.last().copied() == Some(first));
            wipe.extend(points[skip..].iter().map(|point| (point.x, point.y)));
        }
        state.wipe_path = wipe
            .into_iter()
            .rev()
            .map(|(x, y)| super::super::arc::Point {
                x: geometry.scale.unscale(x),
                y: geometry.scale.unscale(y),
            })
            .collect();
    }
}
