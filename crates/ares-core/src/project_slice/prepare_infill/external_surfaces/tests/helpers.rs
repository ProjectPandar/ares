use super::super::ExpansionZone;
use crate::{
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polygon, RegionExpansion,
        RegionExpansionParameters, difference_ex, merge_expansions_into_expolygons, offset2_ex,
        propagate_waves_from_sources,
    },
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

pub(super) type PathSnapshot = Vec<(i64, i64)>;
pub(super) type ExPolygonSnapshot = (PathSnapshot, Vec<PathSnapshot>);

pub(super) fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

pub(super) fn expolygon(contour: &[(i64, i64)], holes: Vec<Polygon>) -> ExPolygon {
    ExPolygon::new(polygon(contour), holes)
}

pub(super) fn square(min: i64, max: i64) -> ExPolygon {
    expolygon(
        &[(min, min), (max, min), (max, max), (min, max)],
        Vec::new(),
    )
}

pub(super) fn params() -> RegionExpansionParameters {
    RegionExpansionParameters {
        tiny_expansion: 1.0,
        initial_step: 2.0,
        other_step: 2.0,
        num_other_steps: 0,
        max_inflation: 4.0,
        arc_tolerance: 0.25,
        shortest_edge_length: 0.0,
    }
}

pub(super) fn zone(expolygons: Vec<ExPolygon>) -> ExpansionZone {
    ExpansionZone::new(expolygons, params())
}

pub(super) fn surface(
    kind: RegionSurfaceKind,
    expolygon: ExPolygon,
    metadata: (f64, u16, f64, u16),
) -> RegionSurface {
    let (thickness, layers, bridge_angle, extra_perimeters) = metadata;
    let mut surface = RegionSurface::internal_with_metadata(
        expolygon,
        thickness,
        layers,
        bridge_angle,
        extra_perimeters,
    );
    surface.retag(kind);
    surface
}

pub(super) fn snapshots(expolygons: &[ExPolygon]) -> Vec<ExPolygonSnapshot> {
    expolygons
        .iter()
        .map(|expolygon| {
            (
                path_snapshot(expolygon.contour()),
                expolygon.holes().iter().map(path_snapshot).collect(),
            )
        })
        .collect()
}

pub(super) fn surface_snapshots(surfaces: &[RegionSurface]) -> Vec<ExPolygonSnapshot> {
    surfaces
        .iter()
        .map(|surface| snapshots(std::slice::from_ref(surface.as_parts().1)).remove(0))
        .collect()
}

fn path_snapshot(polygon: &Polygon) -> PathSnapshot {
    polygon
        .points()
        .iter()
        .map(|point| (point.x(), point.y()))
        .collect()
}

pub(super) fn explicit_pipeline(
    src: Vec<ExPolygon>,
    zones: &mut [ExpansionZone],
    closing_radius: f32,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut processed_expolygons_count = 0_u32;
    let mut expansions = Vec::<RegionExpansion>::new();
    for zone in &mut *zones {
        let mut zone_expansions =
            propagate_waves_from_sources(&src, &zone.expolygons, &zone.parameters, scale)?;
        zone.expanded_into = !zone_expansions.is_empty();
        for expansion in &mut zone_expansions {
            expansion.boundary_id = expansion
                .boundary_id
                .wrapping_add(processed_expolygons_count);
        }
        processed_expolygons_count =
            processed_expolygons_count.wrapping_add(zone.expolygons.len() as u32);
        expansions.append(&mut zone_expansions);
    }

    let expanded = merge_expansions_into_expolygons(src, expansions, scale)?;
    let expanded = offset2_ex(
        &expanded,
        closing_radius,
        -closing_radius,
        JoinType::Miter,
        3.0,
    )?;
    for zone in &mut *zones {
        if zone.expanded_into {
            zone.expolygons = difference_ex(&zone.expolygons, &expanded)?;
        }
    }
    Ok(expanded)
}
