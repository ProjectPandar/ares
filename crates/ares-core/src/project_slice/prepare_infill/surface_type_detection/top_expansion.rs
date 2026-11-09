//! `PrintObject::detect_surfaces_type` top surface expansion
//! (`PrintObject.cpp:2112-2172`, OrcaSlicer v2.5.0-beta `345efec64`).

use crate::{
    ProcessTopSurfaceExpansionDirection as Direction, RegionOptions, SliceError,
    geometry::{
        BoundingBox, CoordinateScale, ExPolygon, JoinType,
        clip_clipper_expolygons_with_subject_bbox, difference_ex, intersection_ex,
        intersection_polygons_ex, offset_expolygon, offset_expolygons, opening_ex,
        union_expolygons,
    },
    project_slice::{
        perimeters::types::Flow,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

use super::geometry::{expolygons, fresh, geometry_error};

/// `ClipperUtils.hpp` `DefaultMiterLimit`.
const DEFAULT_MITER_LIMIT: f64 = 3.0;

/// The scaled inputs of `:2123-2135`, present only when the option gate (`:2121-2122`) passes.
#[derive(Clone, Copy)]
pub(super) struct TopExpansion {
    distance: f32,
    wall_band: f64,
    margin: f64,
    min_top: f32,
    direction: Direction,
    bbox_epsilon: i64,
}

impl TopExpansion {
    /// `(external perimeter, perimeter)` flows come from the layer region; `top_flow` is
    /// `layerm->flow(frTopSolidInfill)`, resolved only behind the gate.
    pub(super) fn new(
        region: &RegionOptions,
        (external, perimeter): (Flow, Flow),
        top_flow: impl FnOnce() -> Result<Flow, SliceError>,
        scale: CoordinateScale,
    ) -> Result<Option<Self>, SliceError> {
        if !(region.top_surface_expansion.0 > 0.0
            && region.top_shell_layers.0 > 0
            && region.top_surface_density.0 > 0.0)
        {
            return Ok(None);
        }
        let wall_loops = if region.only_one_wall_top.0 {
            region.wall_loops.0.min(1)
        } else {
            region.wall_loops.0
        };
        let wall_band = if wall_loops <= 0 {
            0.0
        } else {
            external.scaled_width(scale) as f64
                + perimeter.scaled_spacing(scale) as f64 * f64::from(wall_loops - 1)
        };
        Ok(Some(Self {
            distance: scale.scaled_delta(region.top_surface_expansion.0) as f32,
            wall_band,
            margin: scale.scaled_delta(region.top_surface_expansion_margin.0),
            min_top: top_flow()?.scaled_width(scale) as f32,
            direction: region.top_surface_expansion_direction,
            bbox_epsilon: scale.scaled_delta(1e-4).round() as i64,
        }))
    }
}

/// Grows `top` island by island of `slices` (`layerm_slices_surfaces`), then rebuilds it as
/// the union with the original top minus `bottom`.
pub(super) fn apply(
    expansion: TopExpansion,
    top: &mut Vec<RegionSurface>,
    bottom: &[RegionSurface],
    slices: &[ExPolygon],
) -> Result<(), SliceError> {
    if top.is_empty() {
        return Ok(());
    }
    let tops = union_expolygons(&expolygons(top), &[]).map_err(geometry_error)?;
    let mut grown = Vec::new();
    for island in union_expolygons(slices, &[]).map_err(geometry_error)? {
        let infill_region = if expansion.wall_band > 0.0 {
            offset_expolygon(
                &island,
                -(expansion.wall_band as f32),
                JoinType::Miter,
                DEFAULT_MITER_LIMIT,
            )
            .map_err(geometry_error)?
        } else {
            vec![island.clone()]
        };
        let bounds = BoundingBox::from_expolygon(&island)
            .expect("a union island has a contour")
            .inflated(expansion.bbox_epsilon);
        let island_top = intersection_polygons_ex(
            &clip_clipper_expolygons_with_subject_bbox(&tops, bounds),
            &infill_region,
        )
        .map_err(geometry_error)?;
        if opening_ex(
            &island_top,
            expansion.min_top,
            JoinType::Miter,
            DEFAULT_MITER_LIMIT,
        )
        .map_err(geometry_error)?
        .is_empty()
        {
            continue;
        }
        let expanded = offset_expolygons(&island_top, expansion.distance, JoinType::Miter, 2.0)
            .map_err(geometry_error)?;
        let expanded = match expansion.direction {
            Direction::InwardAndOutward => expanded,
            Direction::Inward => {
                intersection_ex(&expanded, &outline(&island_top)?).map_err(geometry_error)?
            }
            Direction::Outward => {
                let holes =
                    difference_ex(&outline(&island_top)?, &island_top).map_err(geometry_error)?;
                difference_ex(&expanded, &holes).map_err(geometry_error)?
            }
        };
        let allowed = if expansion.margin > 0.0 {
            offset_expolygons(
                &infill_region,
                -(expansion.margin as f32),
                JoinType::Miter,
                DEFAULT_MITER_LIMIT,
            )
            .map_err(geometry_error)?
        } else {
            infill_region
        };
        grown.extend(intersection_ex(&expanded, &allowed).map_err(geometry_error)?);
    }
    let new_top = difference_ex(
        &union_expolygons(&tops, &grown).map_err(geometry_error)?,
        &expolygons(bottom),
    )
    .map_err(geometry_error)?;
    *top = fresh(RegionSurfaceKind::Top, new_top);
    Ok(())
}

/// The top's filled outline: each part's contour with its holes closed (`:2153-2157`).
fn outline(island_top: &[ExPolygon]) -> Result<Vec<ExPolygon>, SliceError> {
    let contours = island_top
        .iter()
        .map(|part| ExPolygon::new(part.contour().clone(), Vec::new()))
        .collect::<Vec<_>>();
    union_expolygons(&contours, &[]).map_err(geometry_error)
}
