use super::{EmitState, LayerGeometry};
use crate::project_slice::perimeters::classic::materialize::ExtrusionPath;

pub(super) fn emit(
    output: &mut Vec<u8>,
    paths: &[ExtrusionPath],
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    let mut remaining_clip = state.options.seam_gap;
    let mut path_count = paths.len();
    while path_count > 0 {
        let length = path_length(&paths[path_count - 1], geometry);
        if length > remaining_clip {
            break;
        }
        path_count -= 1;
        remaining_clip -= length;
    }
    let mut emitted_loop_path = Vec::new();
    for (index, path) in paths[..path_count].iter().enumerate() {
        let end_clip = if index + 1 == path_count {
            remaining_clip
        } else {
            0.0
        };
        super::emit_materialized_path(output, path, end_clip, geometry, state);
        if emitted_loop_path.is_empty() {
            emitted_loop_path.extend_from_slice(&state.wipe_path);
        } else {
            emitted_loop_path.extend(state.wipe_path.iter().copied().skip(1));
        }
    }
    state.wipe_path = emitted_loop_path;
}

fn path_length(path: &ExtrusionPath, geometry: LayerGeometry<'_>) -> f64 {
    path.polyline
        .points
        .windows(2)
        .map(|segment| {
            geometry
                .scale
                .unscale(segment[1].x - segment[0].x)
                .hypot(geometry.scale.unscale(segment[1].y - segment[0].y))
        })
        .sum()
}
