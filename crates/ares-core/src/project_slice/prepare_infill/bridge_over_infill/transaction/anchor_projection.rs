use crate::{
    RegionOptions, SliceError, fill::cross_hatch::line_spacing, geometry::CoordinateScale,
};

use super::super::sparse_anchoring::projected_sparse_density;

pub(super) fn validate(
    options: &RegionOptions,
    spacing: f32,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    // All rotate_template strings are admissible upstream: the meta
    // interpreter (`Fill.cpp:60-203`, ported in projection/rotation/meta.rs)
    // covers meta templates, and the non-meta path feeds every token to
    // `strtod`-style parsing (`Fill.cpp:206-210`) where unparseable tails
    // truncate and unparseable tokens become 0.0 — upstream never rejects.
    // `PrintObject::bridge_over_infill` (`PrintObject.cpp:2468+`) is
    // extruder-agnostic: rewritten surfaces stay in their LayerRegion and
    // emit on the region's role filament (`Fill.cpp:865`,
    // `Fill.cpp:920-924`), so divergent `sparse_infill_filament_id` /
    // `internal_solid_filament_id` / `top_surface_filament_id` are
    // admitted here; role→filament resolution happens downstream.
    // `anchor_length_max < 0.05` is upstream's supported `dont_connect()`
    // selection (`FillBase.hpp:52`; `FillRectilinear.cpp:3366-3373`,
    // `FillBase.cpp:1820-1828`), not an unsupported input — the connect
    // asserts (`FillBase.cpp:1584-1585,2251-2252`) are unreachable there.
    // `infill_anchor < 0` is a schema-legal input upstream
    // (`PrintConfig.cpp:3137-3160` has no `min` bound; `Fill.cpp:984-988`
    // projects `min(anchor, anchor_max)` with no sign check). The
    // `anchor_length >= 0.` asserts (`FillBase.cpp:1584-1585,2251-2252`)
    // are debug-only — the release oracle defines the behavior.
    let density = projected_sparse_density(options);
    if line_spacing(f64::from(spacing), density, 1, scale) <= 0 {
        return Err(SliceError::InvalidInput(
            "invalid Orca option sparse_infill_line_width".to_owned(),
        ));
    }
    Ok(())
}
