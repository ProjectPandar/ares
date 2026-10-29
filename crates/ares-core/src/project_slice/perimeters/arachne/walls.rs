// Per-surface arachne wall generation from OrcaSlicer v2.4.2
// `PerimeterGenerator::process_arachne`
// (`PerimeterGenerator.cpp:2111-2176`): the per-island loop-number and
// outer-wall offset preparation feeding `Arachne::WallToolPaths`
// (`WallToolPaths.cpp:472-702`), plus the infill boundary of
// `PerimeterGenerator.cpp:2475-2531`.

use crate::{
    SliceError,
    arachne::{ExtrusionLine, wall_toolpaths},
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, Polygon, offset_expolygons,
        offset_paths_tree, offset2_ex, simplify_expolygon_polygons, union_ex, union_expolygons,
    },
    project_slice::region_slices::RegionSurface,
};

use super::config::ArachneRecordConfig;

const INSET_OVERLAP_TOLERANCE: f64 = 0.4;
pub(in crate::project_slice) const MITER_LIMIT: f64 = 3.0;

/// One island's generated walls and inner contour
/// (`PerimeterGenerator.cpp:2148-2176`).
pub(in crate::project_slice) struct GeneratedSurfaceWalls {
    pub(in crate::project_slice) toolpaths: Vec<Vec<ExtrusionLine>>,
    pub(in crate::project_slice) infill_contour: Vec<ExPolygon>,
    pub(in crate::project_slice) top_expolygons: Vec<ExPolygon>,
}

/// Scaled flow widths `process_arachne` derives per record
/// (`PerimeterGenerator.cpp:2097-2109`).
#[derive(Clone, Copy)]
pub(in crate::project_slice) struct ArachneSpacings {
    pub(in crate::project_slice) perimeter_spacing: i64,
    /// `perimeter_flow.scaled_width()` — used by the top-surface
    /// regeneration (`PerimeterGenerator.cpp:2179`).
    pub(in crate::project_slice) perimeter_width: i64,
    pub(in crate::project_slice) ext_perimeter_width: i64,
    pub(in crate::project_slice) ext_perimeter_spacing: i64,
    pub(in crate::project_slice) ext_perimeter_spacing2: i64,
    pub(in crate::project_slice) solid_infill_spacing: i64,
}

impl ArachneSpacings {
    pub(in crate::project_slice) fn new(
        scale: CoordinateScale,
        perimeter_flow: &crate::project_slice::perimeters::types::Flow,
        external_perimeter_flow: &crate::project_slice::perimeters::types::Flow,
        solid_infill_flow: &crate::project_slice::perimeters::types::Flow,
    ) -> Result<Self, SliceError> {
        let perimeter_spacing_mm = perimeter_flow.spacing;
        let perimeter_width_mm = perimeter_flow.width;
        let ext_width_mm = external_perimeter_flow.width;
        let ext_spacing_mm = external_perimeter_flow.spacing;
        let solid_infill_spacing_mm = solid_infill_flow.spacing;
        let scaled = |value: f32| -> Result<i64, SliceError> {
            scale.checked_scale(f64::from(value)).ok_or_else(|| {
                SliceError::InvalidInput(
                    "Arachne perimeter spacing exceeds the coordinate range".to_owned(),
                )
            })
        };
        // `ext_perimeter_spacing2`
        // (`PerimeterGenerator.cpp:2103`): the plain external/internal
        // spacing average.
        let spacing2 = scale
            .checked_scale(0.5 * f64::from(ext_spacing_mm + perimeter_spacing_mm))
            .ok_or_else(|| {
                SliceError::InvalidInput(
                    "Arachne perimeter spacing exceeds the coordinate range".to_owned(),
                )
            })?;
        Ok(Self {
            perimeter_spacing: scaled(perimeter_spacing_mm)?,
            perimeter_width: scaled(perimeter_width_mm)?,
            ext_perimeter_width: scaled(ext_width_mm)?,
            ext_perimeter_spacing: scaled(ext_spacing_mm)?,
            ext_perimeter_spacing2: spacing2,
            solid_infill_spacing: scaled(solid_infill_spacing_mm)?,
        })
    }
}

/// Inputs from `PerimeterGenerator.cpp:2111-2248`.
pub(in crate::project_slice) struct SurfaceWallParameters {
    pub(in crate::project_slice) wall_loops: i32,
    pub(in crate::project_slice) layer_height: f64,
    pub(in crate::project_slice) scale: CoordinateScale,
}

/// Generate one island's walls (`PerimeterGenerator.cpp:2111-2248`).
pub(in crate::project_slice) fn generate_surface_walls(
    surface: &RegionSurface,
    config: &ArachneRecordConfig,
    spacings: &ArachneSpacings,
    SurfaceWallParameters {
        wall_loops,
        layer_height,
        scale,
    }: SurfaceWallParameters,
    top_surface: Option<&super::top_surface::TopSurfaceInputs<'_>>,
) -> Result<GeneratedSurfaceWalls, SliceError> {
    let (_, expolygon, _, _, _, extra_perimeters) = surface.as_parts();
    // 0-indexed loops for this island (`PerimeterGenerator.cpp:2113-2114`).
    let mut loop_number = wall_loops + i32::from(extra_perimeters) - 1;
    // Set the bottommost layer to be one wall (`:2121-2122`).
    if config.is_bottom_layer && config.only_one_wall_first_layer {
        loop_number = 0;
    }
    // Orca: set the topmost layer to be one wall (`:2125-2127`).
    if config.is_topmost_layer && loop_number > 0 && config.only_one_wall_top {
        loop_number = 0;
    }
    // `:2163`: the top-surface regeneration keeps the inner loops for
    // non-top areas (the second pass below).
    let inner_loop_number = if config.only_one_wall_top && top_surface.is_some() {
        loop_number - 1
    } else {
        -1
    };
    // `:2165-2166`: the main pass generates a single wall when the feature
    // is active on a non-topmost layer.
    if config.only_one_wall_top && loop_number > 0 {
        loop_number = 0;
    }
    let outer_offset = if config.precise_outer_wall {
        -f64::from((spacings.ext_perimeter_width - spacings.ext_perimeter_spacing) as f32)
    } else {
        -(spacings.ext_perimeter_width as f64 / 2.0 - spacings.ext_perimeter_spacing as f64 / 2.0)
    };
    let simplified = simplify_expolygon_polygons(expolygon, config.surface_simplify_resolution)
        .map_err(geometry_error)?;
    let last = offset_paths_tree(
        &simplified,
        outer_offset as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(geometry_error)?
    .into_expolygons();
    let wall_0_inset = if config.precise_outer_wall {
        -(spacings.ext_perimeter_width / 2 - spacings.ext_perimeter_spacing / 2)
    } else {
        0
    };
    let params = config.params;
    let generated = wall_toolpaths::generate(
        &to_polygons(&last),
        wall_toolpaths::RawWallToolPathConfig {
            outer_spacing: spacings.ext_perimeter_spacing,
            inner_spacing: spacings.perimeter_spacing,
            inset_count: usize::try_from(loop_number + 1)
                .map_err(|_| invalid("wall_loops underflow"))?,
            outer_wall_inset: wall_0_inset,
            layer_height,
            min_bead_width: params.min_bead_width,
            min_feature_size: params.min_feature_size,
            transition_length: params.wall_transition_length,
            // WallToolPaths.cpp:490 / Geometry.hpp:301-305: T is float.
            transitioning_angle: f64::from(
                std::f32::consts::PI * params.wall_transition_angle as f32 / 180.0_f32,
            ),
            transition_filter_deviation: params.wall_transition_filter_deviation,
            wall_distribution_count: params.wall_distribution_count,
            min_length_factor: params.min_length_factor,
            wall_maximum_resolution: params.wall_maximum_resolution,
            wall_maximum_deviation: params.wall_maximum_deviation,
            is_top_or_bottom_layer: config.is_bottom_layer || config.is_topmost_layer,
            coordinate_scale: scale,
        },
    )
    .map_err(|error| {
        SliceError::InvalidInput(format!("Arachne wall generation failed: {error:?}"))
    })?;
    let infill_contour =
        union_ex(&generated.inner_contour, FillRule::NonZero).map_err(geometry_error)?;
    if inner_loop_number < 0 {
        return Ok(GeneratedSurfaceWalls {
            toolpaths: generated.toolpaths,
            infill_contour,
            top_expolygons: Vec::new(),
        });
    }
    // Top-surface regeneration (`PerimeterGenerator.cpp:2177-2247`): the
    // main pass above emitted a single wall; the not-top areas regrow
    // their inner walls in the dedicated module.
    super::top_surface::regenerate(
        generated.toolpaths,
        infill_contour,
        top_surface.expect("inner_loop_number >= 0 implies top surface"),
        config,
        super::top_surface::Regeneration {
            spacings,
            last: &last,
            wall_0_inset,
            layer_height,
            params,
            scale,
            inner_loop_number,
        },
    )
}

/// The infill boundary of one island (`PerimeterGenerator.cpp:2475-2531`).
#[derive(Default)]
pub(in crate::project_slice) struct SurfaceInfillBoundary {
    pub(in crate::project_slice) fill_surfaces: Vec<ExPolygon>,
    pub(in crate::project_slice) fill_no_overlap: Vec<ExPolygon>,
}

#[derive(Clone, Copy)]
pub(in crate::project_slice) struct InfillOverlapPercents {
    pub(in crate::project_slice) infill_wall_overlap: f64,
    pub(in crate::project_slice) top_bottom_infill_wall_overlap: f64,
}

/// Inputs from `PerimeterGenerator.cpp:2475-2478 emitted-wall filter`.
pub(in crate::project_slice) struct WallCounts {
    pub(in crate::project_slice) wall_count: usize,
    pub(in crate::project_slice) loop_number: i32,
}

/// Inputs from `PerimeterGenerator.cpp:2475-2519 overlap selection`.
pub(in crate::project_slice) struct BoundaryOverlap<'a> {
    pub(in crate::project_slice) overlaps: InfillOverlapPercents,
    pub(in crate::project_slice) top_or_bottom: bool,
    pub(in crate::project_slice) scale: CoordinateScale,
    pub(in crate::project_slice) top_expolygons: &'a [ExPolygon],
}

pub(in crate::project_slice) fn surface_infill_boundary(
    infill_contour: Vec<ExPolygon>,
    WallCounts {
        wall_count,
        loop_number,
    }: WallCounts,
    spacings: &ArachneSpacings,
    scaled_resolution: f64,
    BoundaryOverlap {
        overlaps,
        top_or_bottom,
        scale,
        top_expolygons,
    }: BoundaryOverlap<'_>,
) -> Result<SurfaceInfillBoundary, SliceError> {
    // `PerimeterGenerator.cpp:2475-2478`: the too-small infill filter uses
    // the single-wall external spacing average.
    let spacing = if wall_count == 1 {
        spacings.ext_perimeter_spacing2
    } else {
        spacings.perimeter_spacing
    };
    let mut infill_contour = infill_contour;
    if offset_expolygons(
        &infill_contour,
        -((spacing as f64 / 2.0) as f32),
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(geometry_error)?
    .is_empty()
    {
        infill_contour.clear();
    }
    // Inset base by loop count (`PerimeterGenerator.cpp:2480-2493`), then the
    // wall-overlap percentage replaces it.
    let base_inset = if loop_number < 0 {
        0
    } else if loop_number == 0 {
        spacings.ext_perimeter_spacing
    } else {
        spacings.perimeter_spacing
    };
    // `PerimeterGenerator.cpp:2490-2497`: retain the top overlap before
    // replacing the ordinary inset with its selected wall overlap.
    let top_inset = scale
        .scaled_delta(scale.unscale(base_inset) * overlaps.top_bottom_infill_wall_overlap / 100.0)
        as i64;
    let overlap_factor = if top_or_bottom {
        overlaps.top_bottom_infill_wall_overlap / 100.0
    } else {
        overlaps.infill_wall_overlap / 100.0
    };
    // `coord_t(scale_(...get_abs_value(...)))` (`PerimeterGenerator.cpp:2492-2501`):
    // the overlap percentage product truncates to the integer domain before it
    // feeds the offset delta, keeping `inset + min/2` on upstream's rounding
    // grid.
    let inset = (overlap_factor * base_inset as f64) as i64;
    // Simplify infill contours and collapse narrow areas
    // (`PerimeterGenerator.cpp:2495-2502`).
    let mut polygons = Vec::new();
    for expolygon in &infill_contour {
        polygons.extend(
            simplify_expolygon_polygons(expolygon, scaled_resolution).map_err(geometry_error)?,
        );
    }
    let not_filled = union_ex(&polygons, FillRule::NonZero).map_err(geometry_error)?;
    // `coord_t(solid_infill_spacing * (1. - INSET_OVERLAP_TOLERANCE))`
    // (`PerimeterGenerator.cpp:2506`): the truncation to the integer domain
    // before the halving keeps the downstream offset deltas on upstream's
    // half-integer rounding boundary (e.g. 125561/2 = 62780.5).
    let min_perimeter_infill_spacing =
        (spacings.solid_infill_spacing as f64 * (1.0 - INSET_OVERLAP_TOLERANCE)) as i64;
    let mut fill_surfaces = offset2_ex(
        &not_filled,
        (-(min_perimeter_infill_spacing as f64) / 2.0) as f32,
        ((inset as f64) + (min_perimeter_infill_spacing as f64) / 2.0) as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(geometry_error)?;
    // `PerimeterGenerator.cpp:2516-2518`: single-wall top areas use the
    // separate top overlap instead of only the ordinary infill boundary.
    if !top_expolygons.is_empty() {
        let top_fill = offset_expolygons(
            top_expolygons,
            top_inset as f32,
            JoinType::Miter,
            MITER_LIMIT,
        )
        .map_err(geometry_error)?;
        fill_surfaces = union_expolygons(&fill_surfaces, &top_fill).map_err(geometry_error)?;
    }
    // BBS no-overlap infill expolygons (`PerimeterGenerator.cpp:2522-2531`).
    let mut fill_no_overlap = offset2_ex(
        &not_filled,
        (-(min_perimeter_infill_spacing as f64) / 2.0) as f32,
        ((min_perimeter_infill_spacing as f64) / 2.0) as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(geometry_error)?;
    // `PerimeterGenerator.cpp:2529-2531`: retain the unoffset top region.
    if !top_expolygons.is_empty() {
        fill_no_overlap =
            union_expolygons(&fill_no_overlap, top_expolygons).map_err(geometry_error)?;
    }
    Ok(SurfaceInfillBoundary {
        fill_surfaces,
        fill_no_overlap,
    })
}

/// `to_polygons` (`ExPolygons.hpp`): contours and holes as one polygon set.
fn to_polygons(expolygons: &[ExPolygon]) -> Vec<Polygon> {
    let mut polygons = Vec::new();
    for expolygon in expolygons {
        polygons.push(expolygon.contour().clone());
        polygons.extend(expolygon.holes().iter().cloned());
    }
    polygons
}

fn geometry_error(_: ClipperError) -> SliceError {
    SliceError::InvalidInput(
        "Arachne perimeter geometry is outside the supported Clipper range".to_owned(),
    )
}

fn invalid(message: &str) -> SliceError {
    SliceError::InvalidInput(message.to_owned())
}
