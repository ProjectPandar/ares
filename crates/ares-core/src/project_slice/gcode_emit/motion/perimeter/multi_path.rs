//! Ordered non-loop extrusion and joined wipe (`GCode.cpp:6038–6085`).

use super::{EmitState, LayerGeometry};
use crate::project_slice::perimeters::classic::materialize::ExtrusionPath;

pub(super) fn emit(
    output: &mut Vec<u8>,
    paths: &[ExtrusionPath],
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), crate::SliceError> {
    // `estimate_malformations` collects each entity once — for an
    // `ExtrusionMultiPath` (`ExtrusionEntity.hpp:430-436`, role = first
    // path's role, `ExtrusionEntity.hpp:415`) that is the concatenated
    // sub-path polylines, contiguous so the curvature walk crosses the
    // junctions. Per-path collection would truncate the 8mm window at
    // every sub-path boundary.
    if state.options.enable_overhang_speed {
        state
            .curl
            .collect_entity(state.layer_index, paths, &geometry);
    }
    if state.options.adaptive_pa.enabled {
        // This typed producer carries ordinary extruding paths, not force-no-extrusion paths.
        state.adaptive_pa_flow =
            super::super::loop_paths::adaptive_pa::mean(paths, 0.0, geometry.scale);
        state.adaptive_pa_emitted = false;
    }
    for path in paths {
        super::super::materialized::emit(
            output,
            state,
            super::super::materialized::Emission {
                path,
                end_clip: 0.0,
                slope: None,
                geometry,
                inside_loop: true,
            },
        )?;
        state.adaptive_pa_emitted = true;
    }
    state.adaptive_pa_flow = None;
    state.adaptive_pa_emitted = false;
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
    Ok(())
}
