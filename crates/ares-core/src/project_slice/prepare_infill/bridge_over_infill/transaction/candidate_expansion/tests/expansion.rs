use super::*;
use crate::{
    ProcessInfillPattern,
    geometry::{ExPolygon, Point, Polygon, Polyline},
    project_slice::{
        prepare_infill::bridge_over_infill::types::{CandidateSource, CandidateSurface},
        region_slices::RegionSurface,
    },
};

#[test]
fn later_lower_lightning_geometry_preserves_expanded_generator_lines() {
    let baseline = generated_lines(false);
    assert!(!baseline.is_empty());
    assert_eq!(generated_lines(true), baseline);
}

fn generated_lines(move_lower: bool) -> Vec<Polyline> {
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
    horizontal.objects[0]
        .records
        .resize_with(2 * layer_count, || None);
    horizontal.objects[0]
        .records
        .iter_mut()
        .flatten()
        .for_each(|record| record.fill_surfaces.clear());
    let square = rectangle(0, 20_000_000);
    let bridge = rectangle(7_000_000, 13_000_000);
    for layer in 0..3 {
        horizontal.objects[0].records[layer]
            .as_mut()
            .unwrap()
            .fill_surfaces = vec![RegionSurface::internal(ExPolygon::new(
            square.clone(),
            Vec::new(),
        ))];
    }
    horizontal.objects[0].records[2]
        .as_mut()
        .unwrap()
        .fill_surfaces
        .insert(
            0,
            RegionSurface::new(
                RegionSurfaceKind::InternalSolid,
                ExPolygon::new(bridge.clone(), Vec::new()),
            ),
        );
    if move_lower {
        let mut input = prelude.object.records[1].take().unwrap();
        input.current.region_index = 1;
        input.region_id = later_id;
        prelude.object.records[layer_count + 1] = Some(input);
        horizontal.objects[0].records[layer_count + 1] = horizontal.objects[0].records[1].take();
    }
    raw.objects[0].has_lightning_infill = true;
    raw.objects[0].surfaces_by_layer.clear();
    raw.objects[0].surfaces_by_layer.insert(
        2,
        vec![CandidateSurface {
            source: CandidateSource {
                layer_index: 2,
                region_index: 0,
                surface_index: 0,
            },
            new_polygons: vec![bridge],
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
