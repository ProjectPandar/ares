use crate::{
    ProcessInternalBridgeFilter,
    geometry::{
        ClipperError, Coord, CoordinateScale, ExPolygon, JoinType, Polygon,
        difference_polygons_paths, intersection_polygons_paths, offset_paths,
    },
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::types::{BridgeCandidateObject, CandidateSource, CandidateSurface};

const MITER_LIMIT: f64 = 3.0;
const UNSCALED_EPSILON: f64 = 1.0e-4;

pub(super) struct CandidateLayer<'a> {
    pub(super) lower_layer_index: Option<usize>,
    pub(super) region_index: usize,
    pub(super) fill_expolygons: &'a [ExPolygon],
    pub(super) fill_surfaces: &'a [RegionSurface],
    pub(super) sparse_infill_density_percent: f64,
    pub(super) solid_infill_spacing: Coord,
}

pub(super) fn gather_candidates(
    layers: &[Option<CandidateLayer<'_>>],
    has_lightning_infill: bool,
    filter: ProcessInternalBridgeFilter,
    scale: CoordinateScale,
) -> Result<BridgeCandidateObject, ClipperError> {
    let mut surfaces_by_layer = std::collections::BTreeMap::new();
    for (layer_index, layer) in layers.iter().enumerate() {
        let Some(layer) = layer else { continue };
        let Some(lower_layer_index) = layer.lower_layer_index else {
            continue;
        };
        let lower = layers[lower_layer_index].as_ref();
        let candidates = gather_layer(layer_index, layer, lower, filter, scale)?;
        if !candidates.is_empty() {
            surfaces_by_layer.insert(layer_index, candidates);
        }
    }
    Ok(BridgeCandidateObject {
        has_lightning_infill,
        surfaces_by_layer,
    })
}

fn gather_layer(
    layer_index: usize,
    current: &CandidateLayer<'_>,
    lower: Option<&CandidateLayer<'_>>,
    filter: ProcessInternalBridgeFilter,
    scale: CoordinateScale,
) -> Result<Vec<CandidateSurface>, ClipperError> {
    let spacing = current.solid_infill_spacing as f64;
    let spacing_f32 = spacing as f32;
    let scaled_epsilon = scale
        .checked_scale(UNSCALED_EPSILON)
        .expect("the fixed slicer epsilon fits every coordinate scale")
        as f32;
    let multiplier = match filter {
        ProcessInternalBridgeFilter::Disabled => 3.0,
        ProcessInternalBridgeFilter::Limited | ProcessInternalBridgeFilter::NoFilter => 1.0,
    };

    let unsupported =
        lower.map_or_else(Vec::new, |lower| flatten_expolygons(lower.fill_expolygons));
    let unsupported = close_paths(&unsupported, scaled_epsilon)?;
    let mut lower_solids = lower
        .into_iter()
        .flat_map(|lower| {
            lower.fill_surfaces.iter().filter(move |surface| {
                surface.as_parts().0 != RegionSurfaceKind::Internal
                    || lower.sparse_infill_density_percent == 100.0
            })
        })
        .flat_map(surface_paths)
        .collect::<Vec<_>>();
    lower_solids = offset_paths(&lower_solids, -spacing_f32, JoinType::Miter, MITER_LIMIT)?;
    lower_solids = offset_paths(
        &lower_solids,
        ((1.0 + multiplier) * spacing) as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )?;
    let unsupported = offset_paths(
        &unsupported,
        (-multiplier * spacing) as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )?;
    let unsupported = difference_polygons_paths(&unsupported, &lower_solids)?;

    let mut candidates = Vec::new();
    for (surface_index, surface) in current.fill_surfaces.iter().enumerate() {
        let (kind, expolygon, ..) = surface.as_parts();
        if kind != RegionSurfaceKind::InternalSolid {
            continue;
        }
        let source = expolygon_paths(expolygon);
        let surface_unsupported = intersection_polygons_paths(&source, &unsupported)?;
        let new_polygons = match filter {
            ProcessInternalBridgeFilter::NoFilter => offset_paths(
                &surface_unsupported,
                (4.0 * spacing) as f32,
                JoinType::Miter,
                MITER_LIMIT,
            )?,
            ProcessInternalBridgeFilter::Disabled | ProcessInternalBridgeFilter::Limited => {
                let unsupported_area = polygons_area(&surface_unsupported);
                let partially_supported =
                    unsupported_area < polygons_area(&source) - UNSCALED_EPSILON;
                if surface_unsupported.is_empty()
                    || (partially_supported && unsupported_area <= 9.0 * spacing * spacing)
                {
                    continue;
                }
                filtered_candidate(&source, &surface_unsupported, spacing, scale)?
            }
        };
        candidates.push(CandidateSurface {
            source: CandidateSource {
                layer_index,
                region_index: current.region_index,
                surface_index,
            },
            new_polygons,
            bridge_angle: 0.0,
        });
    }
    Ok(candidates)
}

fn filtered_candidate(
    source: &[Polygon],
    unsupported: &[Polygon],
    spacing: f64,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError> {
    let spacing_f32 = spacing as f32;
    let scaled_epsilon = scale
        .checked_scale(UNSCALED_EPSILON)
        .expect("the fixed slicer epsilon fits every coordinate scale")
        as f32;
    let expanded = offset_paths(
        unsupported,
        (4.0 * spacing) as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )?;
    let mut worth_bridging = intersection_polygons_paths(source, &expanded)?;
    let worth_expanded = offset_paths(&worth_bridging, spacing_f32, JoinType::Miter, MITER_LIMIT)?;
    let leftovers = difference_polygons_paths(source, &worth_expanded)?;
    let maximum_leftover_area = spacing
        * scale
            .checked_scale(12.0)
            .expect("the fixed 12 mm threshold fits every coordinate scale") as f64;
    let minimum_leftover_area = spacing * spacing;
    worth_bridging.extend(leftovers.into_iter().filter(|polygon| {
        let area = polygon.area();
        area < maximum_leftover_area && area > minimum_leftover_area
    }));
    let closed = close_paths(&worth_bridging, scaled_epsilon)?;
    intersection_polygons_paths(&closed, source)
}

fn close_paths(paths: &[Polygon], delta: f32) -> Result<Vec<Polygon>, ClipperError> {
    let expanded = offset_paths(paths, delta, JoinType::Miter, MITER_LIMIT)?;
    offset_paths(&expanded, -delta, JoinType::Miter, MITER_LIMIT)
}

fn flatten_expolygons(expolygons: &[ExPolygon]) -> Vec<Polygon> {
    expolygons.iter().flat_map(expolygon_paths).collect()
}

fn surface_paths(surface: &RegionSurface) -> impl Iterator<Item = Polygon> + '_ {
    let expolygon = surface.as_parts().1;
    std::iter::once(expolygon.contour().clone()).chain(expolygon.holes().iter().cloned())
}

fn expolygon_paths(expolygon: &ExPolygon) -> Vec<Polygon> {
    std::iter::once(expolygon.contour().clone())
        .chain(expolygon.holes().iter().cloned())
        .collect()
}

fn polygons_area(polygons: &[Polygon]) -> f64 {
    polygons.iter().map(Polygon::area).sum()
}
