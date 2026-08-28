use crate::{
    SliceError,
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, Point, Polygon, Polyline,
        difference_ex, intersection_ex, intersection_ex_with_safety_offset, medial_axis,
        offset_open_paths, offset2_ex, opening_ex, union_ex, union_expolygons,
    },
    project_slice::perimeters::classic::{
        gap_extrusion::{GapFillCollection, variable_width},
        materialize::ExtrusionRole,
    },
};

use super::{FillExtrusionEntity, geometry_error};

// `FillRectilinear::fill_surface_by_lines` runs a residual medial-axis gap
// pass after laying the solid infill lines: the area not covered by the
// extruded lines (intersected with the no-overlap domain) is opened, offset,
// and re-filled with variable-width erGapFill lines (FillRectilinear.cpp
// 3730-3782).
pub(super) struct ResidualInput<'a> {
    pub(super) output_entities: &'a mut Vec<FillExtrusionEntity>,
    pub(super) no_overlap_expolygons: &'a [ExPolygon],
    pub(super) params: crate::project_slice::group_fills::SurfaceFillParams,
    pub(super) expolygon: &'a ExPolygon,
    pub(super) filled: &'a [Polyline],
    pub(super) spacing: f32,
    pub(super) scale: CoordinateScale,
}

pub(super) fn append_residual(input: ResidualInput<'_>) -> Result<(), SliceError> {
    let ResidualInput {
        output_entities,
        no_overlap_expolygons,
        params,
        expolygon,
        filled,
        spacing,
        scale,
    } = input;
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
    let covered = union_ex(
        &covered_polygons(filled, spacing, scale).map_err(geometry_error)?,
        FillRule::NonZero,
    )
    .map_err(geometry_error)?;
    let unextruded = difference_ex(&domain, &covered).map_err(geometry_error)?;
    let gapfill_areas = intersection_ex(
        &union_expolygons(&unextruded, &[]).map_err(geometry_error)?,
        &domain,
    )
    .map_err(geometry_error)?;
    if gapfill_areas.is_empty() {
        return Ok(());
    }
    let scaled_spacing = scaled(scale, f64::from(spacing))?;
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
    let mut polylines = Vec::new();
    for mut gap in gaps {
        gap.douglas_peucker(SCALED_RESOLUTION * 0.1);
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

const INSET_OVERLAP_TOLERANCE: f64 = 0.15;
const MITER_LIMIT: f32 = 3.0;
const CLIPPER_SAFETY_OFFSET: f64 = 10.0;
const SCALED_RESOLUTION: f64 = 0.0125;
const VORONOI_ERROR: &str = "medial-axis gap fill Voronoi diagram failed";
const FLOW_ERROR: &str = "gap residual flow is invalid";
