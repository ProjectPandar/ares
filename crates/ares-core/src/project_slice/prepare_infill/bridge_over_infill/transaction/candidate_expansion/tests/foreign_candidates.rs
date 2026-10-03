use super::*;
use crate::{
    ProcessInfillPattern,
    geometry::{ExPolygon, Point, Polygon, Polyline},
    project_slice::{
        prepare_infill::{
            bridge_over_infill::types::{CandidateSource, CandidateSurface},
            surface_type_detection::types::PreparedSurfaceTypeRecord,
        },
        region_slices::RegionSurface,
    },
};

#[test]
fn later_region_bridge_extent_does_not_expand_front_region_generator_lines() {
    let baseline = generated_lines(false, CurrentSparseRegions::Front);
    assert!(!baseline.is_empty());
    assert_eq!(generated_lines(true, CurrentSparseRegions::Front), baseline);
}

#[test]
fn later_region_own_bridge_extent_changes_its_generator_lines() {
    let baseline = generated_lines(false, CurrentSparseRegions::Both);
    let expanded = generated_lines(true, CurrentSparseRegions::Both);
    assert!(!baseline.is_empty());
    assert!(!expanded.is_empty());
    assert_ne!(expanded, baseline);
}

#[test]
fn empty_front_internal_stops_later_region_bridge_expansion() {
    let baseline = generated_lines(false, CurrentSparseRegions::Later);
    assert!(!baseline.is_empty());
    assert_eq!(generated_lines(true, CurrentSparseRegions::Later), baseline);
}

enum CurrentSparseRegions {
    Front,
    Both,
    Later,
}

fn generated_lines(wide_candidate: bool, geometry: CurrentSparseRegions) -> Vec<Polyline> {
    let mut raw = prepare(KsrArchive::new());
    let horizontal = &mut raw.predecessor.predecessor;
    let prelude = &mut horizontal.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let layer_count = prelude.object.object.as_parts().0.plan.layers.len();
    let regions = &mut prelude.object.object.as_parts_mut().0.regions;
    regions[0].options.sparse_infill_pattern = ProcessInfillPattern::Lightning;
    let options = regions[0].options.clone();
    let later_id = regions[0].id + 1;
    regions.push(PostRegion {
        id: later_id,
        options,
        layers: (0..layer_count)
            .map(|_| RegionLayer {
                surfaces: Vec::new(),
            })
            .collect(),
    });
    prelude.object.records.resize_with(2 * layer_count, || None);
    let mut input = prelude.object.records[2].unwrap();
    input.current.region_index = 1;
    input.region_id = later_id;
    prelude.object.records[layer_count + 2] = Some(input);
    horizontal.objects[0]
        .records
        .resize_with(2 * layer_count, || None);
    horizontal.objects[0]
        .records
        .iter_mut()
        .flatten()
        .for_each(|record| record.fill_surfaces.clear());
    let square = rectangle(0, 20_000_000);
    let bridge = rectangle(5_500_000, 14_500_000);
    for layer in 0..2 {
        horizontal.objects[0].records[layer]
            .as_mut()
            .unwrap()
            .fill_surfaces = vec![RegionSurface::internal(ExPolygon::new(
            square.clone(),
            Vec::new(),
        ))];
    }
    let mut hole = bridge.clone();
    hole.reverse();
    horizontal.objects[0].records[2]
        .as_mut()
        .unwrap()
        .fill_surfaces = match geometry {
        CurrentSparseRegions::Front => vec![RegionSurface::internal(ExPolygon::new(
            square.clone(),
            vec![hole.clone()],
        ))],
        CurrentSparseRegions::Both => vec![RegionSurface::internal(ExPolygon::new(
            Polygon::new(
                square
                    .points()
                    .iter()
                    .map(|point| Point::new(point.x() + 30_000_000, point.y()))
                    .collect(),
            ),
            Vec::new(),
        ))],
        CurrentSparseRegions::Later => Vec::new(),
    };
    let solid = RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        ExPolygon::new(bridge.clone(), Vec::new()),
    );
    let mut later_surfaces = vec![solid.clone()];
    let boundary = match geometry {
        CurrentSparseRegions::Front => bridge.clone(),
        CurrentSparseRegions::Both | CurrentSparseRegions::Later => {
            later_surfaces.push(RegionSurface::internal(ExPolygon::new(
                square.clone(),
                vec![hole],
            )));
            square.clone()
        }
    };
    horizontal.objects[0].records[layer_count + 2] = Some(PreparedSurfaceTypeRecord {
        perimeters: Vec::new(),
        thin_fills: Vec::new(),
        slices: vec![solid.clone()],
        fill_surfaces: later_surfaces,
        fill_expolygons: vec![ExPolygon::new(boundary.clone(), Vec::new())],
        fill_no_overlap_expolygons: vec![ExPolygon::new(boundary, Vec::new())],
    });
    raw.objects[0].has_lightning_infill = true;
    raw.objects[0].surfaces_by_layer.clear();
    raw.objects[0].surfaces_by_layer.insert(
        2,
        vec![CandidateSurface {
            source: CandidateSource {
                layer_index: 2,
                region_index: 1,
                surface_index: 0,
            },
            new_polygons: vec![if wide_candidate {
                bridge
            } else {
                rectangle(9_500_000, 10_500_000)
            }],
            bridge_angle: 0.0,
        }],
    );

    super::super::prepare(&raw.predecessor, &mut raw.objects).unwrap();

    let lines = raw.objects[0]
        .lightning_generator
        .as_ref()
        .unwrap()
        .convert_to_lines(2, &[square], 0)
        .unwrap();
    bridge_over_infill::dispose(raw);
    lines
}

fn rectangle(low: i64, high: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(low, low),
        Point::new(high, low),
        Point::new(high, high),
        Point::new(low, high),
    ])
}
