use crate::{
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::{
            bridge_over_infill::{
                self,
                types::{CandidateSource, CandidateSurface},
            },
            surface_type_detection::types::PreparedSurfaceTypeRecord,
        },
        region_slices::{PostRegion, RegionLayer, RegionSurface, RegionSurfaceKind},
        tests::{prepare_infill::bridge_over_infill::prepare, support::KsrArchive},
    },
};

#[test]
fn later_region_bridge_uses_its_flow_and_rewrites_all_physical_layer_regions() {
    let mut raw = prepare(KsrArchive::new());
    let horizontal = &mut raw.predecessor.predecessor;
    let prelude = &mut horizontal.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let layer_count = prelude.object.object.as_parts().0.plan.layers.len();
    let mut lower_input = *prelude.object.records[14].as_ref().unwrap();
    let mut upper_input = *prelude.object.records[15].as_ref().unwrap();
    let regions = &mut prelude.object.object.as_parts_mut().0.regions;
    regions.push(PostRegion {
        id: regions[0].id + 1,
        options: regions[0].options.clone(),
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
    lower_input.solid_infill_flow.spacing = 0.5;
    upper_input.solid_infill_flow.spacing = 0.2;
    prelude.object.records[14] = Some(lower_input);
    prelude.object.records[15] = Some(upper_input);
    lower_input.current.region_index = 1;
    upper_input.current.region_index = 1;
    upper_input.solid_infill_flow.spacing = 1.0;
    prelude.object.records[layer_count + 14] = Some(lower_input);
    prelude.object.records[layer_count + 15] = Some(upper_input);
    for region in 0..2 {
        horizontal.objects[0].records[region * layer_count + 14] =
            Some(record(RegionSurfaceKind::Internal));
        horizontal.objects[0].records[region * layer_count + 15] =
            Some(record(RegionSurfaceKind::InternalSolid));
    }
    raw.objects[0].surfaces_by_layer.clear();
    raw.objects[0].surfaces_by_layer.insert(
        15,
        vec![CandidateSurface {
            source: CandidateSource {
                layer_index: 15,
                region_index: 1,
                surface_index: 0,
            },
            new_polygons: vec![square().contour().clone()],
            bridge_angle: 0.75,
        }],
    );

    super::prepare(&mut raw.predecessor, &raw.objects).unwrap();

    let records = &raw.predecessor.predecessor.objects[0].records;
    for region in 0..2 {
        let solid_area: f64 = records[region * layer_count + 14]
            .as_ref()
            .unwrap()
            .fill_surfaces
            .iter()
            .filter(|surface| surface.as_parts().0 == RegionSurfaceKind::InternalSolid)
            .map(|surface| surface.as_parts().1.area())
            .sum();
        assert!(
            solid_area > 18e12 && solid_area < 20e12,
            "regional ensuring area {solid_area}"
        );
    }
    assert!(
        records[15]
            .as_ref()
            .unwrap()
            .fill_surfaces
            .iter()
            .all(|surface| surface.as_parts().0 != RegionSurfaceKind::InternalBridge)
    );
    assert!(records[15].as_ref().unwrap().fill_surfaces.is_empty());
    let bridges = records[layer_count + 15]
        .as_ref()
        .unwrap()
        .fill_surfaces
        .iter()
        .filter(|surface| surface.as_parts().0 == RegionSurfaceKind::InternalBridge)
        .collect::<Vec<_>>();
    assert_eq!(bridges.len(), 1);
    assert_eq!(bridges[0].as_parts().1.area(), 100e12);
    assert_eq!(bridges[0].as_parts().4, 0.75);
    bridge_over_infill::dispose(raw);
}

fn record(kind: RegionSurfaceKind) -> PreparedSurfaceTypeRecord {
    PreparedSurfaceTypeRecord {
        perimeters: Vec::new(),
        thin_fills: Vec::new(),
        slices: Vec::new(),
        fill_surfaces: vec![RegionSurface::new(kind, square())],
        fill_expolygons: Vec::new(),
        fill_no_overlap_expolygons: Vec::new(),
    }
}

fn square() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10_000_000, 0),
            Point::new(10_000_000, 10_000_000),
            Point::new(0, 10_000_000),
        ]),
        Vec::new(),
    )
}
