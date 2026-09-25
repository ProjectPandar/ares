//! `ARES_DUMP_STAGES=<file>` diagnostic — appended at prepare-stage
//! boundaries to trace which stage creates a given fill surface.

use crate::project_slice::region_slices::RegionSurface;

pub(in crate::project_slice) fn dump_stage_surfaces(
    stage: &str,
    layer_index: usize,
    surfaces: &[RegionSurface],
) {
    let Ok(path) = std::env::var("ARES_DUMP_STAGES") else {
        return;
    };
    use std::io::Write;
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    else {
        return;
    };
    for surface in surfaces {
        let (kind, expolygon, ..) = surface.as_parts();
        let bounds = expolygon.contour().points().iter().fold(
            [i64::MAX, i64::MAX, i64::MIN, i64::MIN],
            |mut bounds, p| {
                bounds[0] = bounds[0].min(p.x());
                bounds[1] = bounds[1].min(p.y());
                bounds[2] = bounds[2].max(p.x());
                bounds[3] = bounds[3].max(p.y());
                bounds
            },
        );
        let _ = writeln!(file, "{stage} l={layer_index} kind={kind:?} bb={bounds:?}");
    }
}
