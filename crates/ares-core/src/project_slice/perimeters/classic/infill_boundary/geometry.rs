use crate::{
    SliceError,
    geometry::{
        ExPolygon, FillRule, JoinType, difference_ex_polygons, intersection_ex, offset_expolygons,
        offset2_ex, simplify_expolygon_polygons, union_ex, union_expolygons,
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
    extra: Option<&super::super::extra_perimeters::Context<'_>>,
    counterbore_fill: &[ExPolygon],
) -> Result<StagedRecord, SliceError> {
    assert_eq!(source.len(), top.len());
    assert_eq!(source.len(), validated.surfaces.len());
    // process_no_bridge appends stInternal before the per-surface wall loop.
    let mut fill_surfaces = counterbore_fill
        .iter()
        .cloned()
        .map(RegionSurface::internal)
        .collect::<Vec<_>>();
    let mut fill_no_overlap = Vec::new();
    let mut overlap = Vec::with_capacity(source.len());
    let mut extra_perimeters = Vec::with_capacity(extra.map_or(0, |_| source.len()));
    for ((source, top), validated) in source.iter().zip(top).zip(&validated.surfaces) {
        assert_eq!(source.source_index, top.source_index);
        assert_eq!(source.source_index, validated.overlap.source_index);
        let staged = stage_surface(source, top, *validated)?;
        let generated = extra
            .map(|context| {
                let infill = staged
                    .fill_surfaces
                    .iter()
                    .map(|surface| surface.as_parts().1.clone())
                    .collect::<Vec<_>>();
                context.generate(&infill)
            })
            .transpose()?;
        fill_surfaces.extend(staged.fill_surfaces);
        let mut paths = Vec::new();
        if let Some(generated) = generated
            && generated.has_regions
        {
            let mut remaining = Vec::new();
            for surface in &fill_surfaces {
                remaining.extend(
                    difference_ex_polygons(
                        std::slice::from_ref(surface.as_parts().1),
                        &generated.filled,
                    )
                    .map_err(geometry_error)?
                    .into_iter()
                    .map(RegionSurface::internal),
                );
            }
            fill_surfaces = remaining;
            paths = extra.unwrap().materialize(generated.paths);
        }
        if extra.is_some() {
            extra_perimeters.push(paths);
        }
        fill_no_overlap.extend(staged.fill_no_overlap);
        overlap.push(validated.overlap);
    }
    Ok(StagedRecord {
        surface_count: source.len(),
        fill_surfaces,
        fill_no_overlap,
        overlap,
        extra_perimeters,
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
    observe_extra_perimeters_guard();
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

fn observe(_step: GeometryStep) -> Result<(), SliceError> {
    Ok(())
}

fn observe_extra_perimeters_guard() {}

pub(super) struct StagedSurfaceOutput {
    pub(super) fill_surfaces: Vec<RegionSurface>,
    pub(super) fill_no_overlap: Vec<ExPolygon>,
}
