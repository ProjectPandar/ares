use crate::{
    ProcessInfillPattern, RegionOptions, SliceError, fill::cross_hatch::line_spacing,
    geometry::CoordinateScale,
};

use super::super::sparse_anchoring::{projected_anchor_lengths, projected_sparse_density};

pub(super) fn validate(
    options: &RegionOptions,
    spacing: f32,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    if !matches!(
        options.sparse_infill_pattern,
        ProcessInfillPattern::CrossHatch
            | ProcessInfillPattern::Grid
            | ProcessInfillPattern::Triangles
            | ProcessInfillPattern::Cubic
            | ProcessInfillPattern::Gyroid
            | ProcessInfillPattern::ThreeDHoneycomb
            | ProcessInfillPattern::Rectilinear
            | ProcessInfillPattern::ZigZag
            | ProcessInfillPattern::Monotonic
            | ProcessInfillPattern::MonotonicLine
            | ProcessInfillPattern::AlignedRectilinear
            | ProcessInfillPattern::Line
            | ProcessInfillPattern::CrossZag
            | ProcessInfillPattern::LockedZag
            | ProcessInfillPattern::Concentric
            | ProcessInfillPattern::Honeycomb
            | ProcessInfillPattern::HilbertCurve
            | ProcessInfillPattern::ArchimedeanChords
            | ProcessInfillPattern::OctagramSpiral
            // `AdaptiveCubic`/`SupportCubic` are accepted: the
            // FillAdaptive port provides `adaptive::anchoring_lines`
            // for the sparse anchoring path (`sparse_anchoring/arms.rs`).
            | ProcessInfillPattern::AdaptiveCubic
            | ProcessInfillPattern::SupportCubic
            | ProcessInfillPattern::TriHexagon
            | ProcessInfillPattern::QuarterCubic
            | ProcessInfillPattern::LateralLattice
            | ProcessInfillPattern::LateralHoneycomb
            | ProcessInfillPattern::TpmsD
            | ProcessInfillPattern::TpmsFk
            | ProcessInfillPattern::Lightning
    ) {
        return unsupported("sparse_infill_pattern");
    }
    // `gyroid_optimized` is supported — `FillGyroid.cpp` tpms/gyroid_optimized
    // field emission with the real layer height + post-offset re-clipping.
    if options.sparse_infill_density.0 <= 0.0 {
        return unsupported("sparse_infill_density");
    }
    // All rotate_template strings are admissible upstream: the meta
    // interpreter (`Fill.cpp:60-203`, ported in projection/rotation/meta.rs)
    // covers meta templates, and the non-meta path feeds every token to
    // `strtod`-style parsing (`Fill.cpp:206-210`) where unparseable tails
    // truncate and unparseable tokens become 0.0 — upstream never rejects.
    if !matches!(
        options.top_surface_pattern,
        ProcessInfillPattern::Monotonic
            | ProcessInfillPattern::MonotonicLine
            | ProcessInfillPattern::Rectilinear
            | ProcessInfillPattern::AlignedRectilinear
            | ProcessInfillPattern::Concentric
            | ProcessInfillPattern::HilbertCurve
            | ProcessInfillPattern::ArchimedeanChords
            | ProcessInfillPattern::OctagramSpiral
    ) {
        return unsupported("top_surface_pattern");
    }
    if !matches!(
        options.internal_solid_infill_pattern,
        ProcessInfillPattern::Monotonic
            | ProcessInfillPattern::MonotonicLine
            | ProcessInfillPattern::AlignedRectilinear
            | ProcessInfillPattern::Concentric
            | ProcessInfillPattern::Rectilinear
            | ProcessInfillPattern::HilbertCurve
            | ProcessInfillPattern::ArchimedeanChords
            | ProcessInfillPattern::OctagramSpiral
    ) {
        return unsupported("internal_solid_infill_pattern");
    }
    // `PrintObject::bridge_over_infill` (`PrintObject.cpp:2468+`) is
    // extruder-agnostic: rewritten surfaces stay in their LayerRegion and
    // emit on the region's role filament (`Fill.cpp:865`,
    // `Fill.cpp:920-924`), so divergent `sparse_infill_filament_id` /
    // `internal_solid_filament_id` / `top_surface_filament_id` are
    // admitted here; role→filament resolution happens downstream.
    let (anchor, anchor_max) = projected_anchor_lengths(options, f64::from(spacing));
    // `anchor_length_max < 0.05` is upstream's supported `dont_connect()`
    // selection (`FillBase.hpp:52`; `FillRectilinear.cpp:3366-3373`,
    // `FillBase.cpp:1820-1828`), not an unsupported input — the connect
    // asserts (`FillBase.cpp:1584-1585,2251-2252`) are unreachable there.
    if !anchor_max.is_finite() {
        return unsupported("infill_anchor_max");
    }
    // `infill_anchor < 0` is a schema-legal input upstream
    // (`PrintConfig.cpp:3137-3160` has no `min` bound; `Fill.cpp:984-988`
    // projects `min(anchor, anchor_max)` with no sign check). The
    // `anchor_length >= 0.` asserts (`FillBase.cpp:1584-1585,2251-2252`)
    // are debug-only — the release oracle defines the behavior.
    if !anchor.is_finite() {
        return unsupported("infill_anchor");
    }
    let density = projected_sparse_density(options);
    if line_spacing(f64::from(spacing), density, 1, scale) <= 0 {
        return Err(SliceError::InvalidInput(
            "invalid Orca option sparse_infill_line_width".to_owned(),
        ));
    }
    Ok(())
}

fn unsupported(key: &str) -> Result<(), SliceError> {
    Err(SliceError::UnsupportedProjectFeature(key.to_owned()))
}
