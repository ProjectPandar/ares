use crate::{
    ProcessGapFillTarget, SliceError,
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, Point, Polygon, Polyline,
        chain_points, difference_ex, intersection_ex, intersection_ex_with_safety_offset,
        medial_axis, offset_open_paths, offset2_ex, opening_ex, union_ex, union_expolygons,
    },
    project_slice::{
        perimeters::classic::{
            gap_extrusion::{GapFillCollection, variable_width},
            materialize::ExtrusionRole,
        },
        region_slices::RegionSurfaceKind,
    },
};

use super::{FillExtrusionEntity, geometry_error};

// The residual medial-axis gap pass runs after a full-density non-bridge fill
// lays its extrusions: the area not covered by those extrusions (intersected
// with the no-overlap domain) is opened, offset, and re-filled with
// variable-width erGapFill lines. This mirrors the active upstream
// `Fill::_create_gap_fill` (FillBase.cpp:195-245), which is attached after
// every full-density Fill is materialized (FillBase.cpp:133-189).
pub(super) struct ResidualInput<'a> {
    pub(super) output_entities: &'a mut Vec<FillExtrusionEntity>,
    pub(super) no_overlap_expolygons: &'a [ExPolygon],
    pub(super) params: crate::project_slice::group_fills::SurfaceFillParams,
    pub(super) kind: RegionSurfaceKind,
    pub(super) expolygon: &'a ExPolygon,
    pub(super) filled: &'a [Polyline],
    pub(super) spacing: f32,
    // Some(fillers) lets a non-line pattern supply its own coverage (e.g. the
    // concentric solid, which fully covers its narrow domain); None derives
    // coverage by inflating `filled` centrelines by `spacing` (straight-line
    // patterns).
    pub(super) covered_override: Option<&'a [ExPolygon]>,
    pub(super) scale: CoordinateScale,
}

pub(super) fn append_residual(input: ResidualInput<'_>) -> Result<(), SliceError> {
    let ResidualInput {
        output_entities,
        no_overlap_expolygons,
        params,
        kind,
        expolygon,
        filled,
        spacing,
        covered_override,
        scale,
    } = input;
    // FillBase.cpp:201-203: gap_fill_target gates the whole pass — `nowhere`
    // disables it entirely, and internal-solid surfaces require `everywhere`.
    // FillBase.cpp:236: bridge surfaces never receive gap fill.
    if params.gap_fill_target == ProcessGapFillTarget::Nowhere
        || (kind == RegionSurfaceKind::InternalSolid
            && params.gap_fill_target != ProcessGapFillTarget::Everywhere)
        || params.extrusion_role.is_bridge()
    {
        return Ok(());
    }
    if no_overlap_expolygons.is_empty() || params.density < 100.0 {
        return Ok(());
    }
    // Fill.cpp:1328 intersects the no-overlap domain with the surface using a
    // safety offset.
    let domain =
        intersection_ex_with_safety_offset(no_overlap_expolygons, std::slice::from_ref(expolygon))
            .map_err(geometry_error)?;
    if domain.is_empty() {
        return Ok(());
    }
    let covered = match covered_override {
        Some(covered) => covered.to_vec(),
        None => union_ex(
            &covered_polygons(filled, spacing, scale).map_err(geometry_error)?,
            FillRule::NonZero,
        )
        .map_err(geometry_error)?,
    };
    let unextruded = difference_ex(&domain, &covered).map_err(geometry_error)?;
    let gapfill_areas = intersection_ex(
        &union_expolygons(&unextruded, &[]).map_err(geometry_error)?,
        &domain,
    )
    .map_err(geometry_error)?;
    if gapfill_areas.is_empty() {
        return Ok(());
    }
    // FillBase.cpp:205,212-214: medial min/max derive from the nominal
    // configured flow spacing (`new_flow = params.flow`), not the adjusted
    // generator spacing used for covered-area reconstruction.
    let scaled_spacing = scaled(scale, f64::from(params.flow.spacing))?;
    let minimum = 0.2 * scaled_spacing * (1.0 - INSET_OVERLAP_TOLERANCE);
    let maximum = 2.0 * scaled_spacing;
    let gaps = difference_ex(
        &opening_ex(
            &gapfill_areas,
            (minimum / 2.0) as f32,
            JoinType::Miter,
            f64::from(MITER_LIMIT),
        )
        .map_err(geometry_error)?,
        &offset2_ex(
            &gapfill_areas,
            -((maximum / 2.0) as f32),
            (maximum / 2.0 + CLIPPER_SAFETY_OFFSET) as f32,
            JoinType::Miter,
            f64::from(MITER_LIMIT),
        )
        .map_err(geometry_error)?,
    )
    .map_err(geometry_error)?;
    if gaps.is_empty() {
        return Ok(());
    }
    let minimum_length = scaled(scale, params.filter_out_gap_fill)?;
    // FillBase.cpp:218-227: sort gap regions by chained first point so the
    // medial pass orders travel the same way upstream does.
    let ordering = gaps
        .iter()
        .map(|gap| gap.contour().points()[0])
        .collect::<Vec<_>>();
    let order = chain_points(&ordering);
    let mut polylines = Vec::new();
    for index in order {
        let mut gap = gaps[index].clone();
        // FillBase.cpp:230: DP-simplify in scaled units (SCALED_RESOLUTION is a
        // scaled quantity upstream, libslic3r.h:76-79).
        gap.douglas_peucker(scaled(scale, RESOLUTION * 0.1)?);
        polylines.extend(
            medial_axis(&gap, minimum, maximum, scale)
                .map_err(|_| SliceError::InvalidInput(VORONOI_ERROR.to_owned()))?,
        );
    }
    polylines.retain(|polyline| polyline.length() >= minimum_length);
    if polylines.is_empty() {
        return Ok(());
    }
    let GapFillCollection { entities } =
        variable_width::convert_with_role(&polylines, params.flow, scale, ExtrusionRole::GapFill)
            .map_err(|_| SliceError::InvalidInput(FLOW_ERROR.to_owned()))?;
    output_entities.extend(entities.into_iter().map(FillExtrusionEntity::VariableWidth));
    Ok(())
}

fn covered_polygons(
    filled: &[Polyline],
    spacing: f32,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError> {
    let delta = (f64::from(spacing / 2.0) / scale.factor()) as f32 + CLIPPER_SAFETY_OFFSET as f32;
    let mut output = Vec::new();
    for polyline in filled {
        let points = polyline
            .points()
            .iter()
            .map(|point| Point::new(point.x(), point.y()))
            .collect();
        output.append(&mut offset_open_paths(
            &[Polygon::new(points)],
            delta,
            JoinType::Square,
            0.0,
        )?);
    }
    Ok(output)
}

fn scaled(scale: CoordinateScale, value: f64) -> Result<f64, SliceError> {
    scale
        .checked_scale(value)
        .map(|coordinate| coordinate as f64)
        .ok_or_else(|| SliceError::InvalidInput("gap residual spacing is out of range".to_owned()))
}

const INSET_OVERLAP_TOLERANCE: f64 = 0.4;
const MITER_LIMIT: f32 = 3.0;
const CLIPPER_SAFETY_OFFSET: f64 = 10.0;
const RESOLUTION: f64 = 0.0125;
const VORONOI_ERROR: &str = "medial-axis gap fill Voronoi diagram failed";
const FLOW_ERROR: &str = "gap residual flow is invalid";
