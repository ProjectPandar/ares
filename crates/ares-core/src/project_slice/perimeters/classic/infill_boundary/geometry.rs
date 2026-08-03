use crate::{
    SliceError,
    geometry::{
        ExPolygon, FillRule, JoinType, intersection_ex, offset_expolygons, offset2_ex,
        simplify_expolygon_polygons, union_ex, union_expolygons,
    },
    project_slice::{
        perimeters::classic::{
            gap_extrusion::PreparedGapExtrusionSurface, top_split::PreparedTopSplitSurface,
        },
        region_slices::RegionSurface,
    },
};

use super::types::{NoOverlapOffset, StagedRecord, ValidatedRecord};

const MITER_LIMIT: f64 = 3.0;
const GEOMETRY_ERROR: &str =
    "Classic infill-boundary geometry is outside the supported Clipper range";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum GeometryStep {
    Simplify,
    AggregateUnion,
    OrdinaryOffset,
    TopOffset,
    TopIntersection,
    TopOverlapOffset,
    TopUnion,
    SurfaceAppend,
    ExtraPerimeterGuard,
    NoOverlapTwo,
    NoOverlapOne,
    FinalTopUnion,
}

pub(super) fn stage_record(
    source: &[PreparedGapExtrusionSurface],
    top: &[PreparedTopSplitSurface],
    validated: &ValidatedRecord,
) -> Result<StagedRecord, SliceError> {
    assert_eq!(source.len(), top.len());
    assert_eq!(source.len(), validated.surfaces.len());
    let mut fill_surfaces = Vec::new();
    let mut fill_no_overlap = Vec::new();
    let mut overlap = Vec::with_capacity(source.len());
    for ((source, top), validated) in source.iter().zip(top).zip(&validated.surfaces) {
        assert_eq!(source.source_index, top.source_index);
        assert_eq!(source.source_index, validated.overlap.source_index);
        let staged = stage_surface(source, top, *validated)?;
        fill_surfaces.extend(staged.fill_surfaces);
        fill_no_overlap.extend(staged.fill_no_overlap);
        overlap.push(validated.overlap);
    }
    Ok(StagedRecord {
        surface_count: source.len(),
        fill_surfaces,
        fill_no_overlap,
        overlap,
    })
}

pub(super) fn stage_surface(
    source: &PreparedGapExtrusionSurface,
    top: &PreparedTopSplitSurface,
    validated: super::types::ValidatedSurface,
) -> Result<StagedSurfaceOutput, SliceError> {
    let mut polygons = Vec::new();
    for expolygon in &source.remaining {
        observe(GeometryStep::Simplify)?;
        polygons.extend(
            simplify_expolygon_polygons(expolygon, validated.overlap.scaled_resolution)
                .map_err(geometry_error)?,
        );
    }
    observe(GeometryStep::AggregateUnion)?;
    let not_filled = union_ex(&polygons, FillRule::NonZero).map_err(geometry_error)?;
    observe(GeometryStep::OrdinaryOffset)?;
    let ordinary = offset2_ex(
        &not_filled,
        validated.ordinary_first,
        validated.ordinary_second,
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(geometry_error)?;

    observe(GeometryStep::TopOffset)?;
    let top_offset = offset_expolygons(
        &top.top_fills,
        validated.top_offset,
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(geometry_error)?;
    observe(GeometryStep::TopIntersection)?;
    let top_infill = intersection_ex(&top.fill_clip, &top_offset).map_err(geometry_error)?;
    let infill = if top.top_fills.is_empty() {
        ordinary
    } else {
        observe(GeometryStep::TopOverlapOffset)?;
        let expanded_top = offset_expolygons(
            &top_infill,
            validated.top_overlap,
            JoinType::Miter,
            MITER_LIMIT,
        )
        .map_err(geometry_error)?;
        observe(GeometryStep::TopUnion)?;
        union_expolygons(&ordinary, &expanded_top).map_err(geometry_error)?
    };
    let fill_surfaces = infill.into_iter().map(RegionSurface::internal).collect();
    observe(GeometryStep::SurfaceAppend)?;

    observe(GeometryStep::ExtraPerimeterGuard)?;
    observe_inactive_extra_perimeters_guard();
    let ordinary_no_overlap = match validated.no_overlap {
        NoOverlapOffset::Two { first, second } => {
            observe(GeometryStep::NoOverlapTwo)?;
            offset2_ex(&not_filled, first, second, JoinType::Miter, MITER_LIMIT)
                .map_err(geometry_error)?
        }
        NoOverlapOffset::One { delta } => {
            observe(GeometryStep::NoOverlapOne)?;
            offset_expolygons(&not_filled, delta, JoinType::Miter, MITER_LIMIT)
                .map_err(geometry_error)?
        }
    };
    let fill_no_overlap = if top.top_fills.is_empty() {
        ordinary_no_overlap
    } else {
        observe(GeometryStep::FinalTopUnion)?;
        union_expolygons(&ordinary_no_overlap, &top_infill).map_err(geometry_error)?
    };
    Ok(StagedSurfaceOutput {
        fill_surfaces,
        fill_no_overlap,
    })
}

fn geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    geometry_error_value()
}

fn geometry_error_value() -> SliceError {
    SliceError::InvalidInput(GEOMETRY_ERROR.to_owned())
}

fn observe(step: GeometryStep) -> Result<(), SliceError> {
    #[cfg(test)]
    if super::tests::observe_step(step) {
        return Err(geometry_error_value());
    }
    #[cfg(not(test))]
    let _ = step;
    Ok(())
}

fn observe_inactive_extra_perimeters_guard() {
    #[cfg(test)]
    super::tests::observe_guard();
}

pub(super) struct StagedSurfaceOutput {
    pub(super) fill_surfaces: Vec<RegionSurface>,
    pub(super) fill_no_overlap: Vec<ExPolygon>,
}
