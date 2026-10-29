use super::*;
use crate::project_slice::gcode_emit::tool_ordering::wiping::CollectionId;
use crate::project_slice::perimeters::classic::materialize::Polyline3;
use crate::{
    OrcaInt,
    geometry::{Point, Polyline},
    project_slice::fill_entities::{FillExtrusionEntity, FillExtrusionPath},
};

#[test]
fn tool_buckets_preserve_source_collection_identity_and_region() {
    let entity = || {
        IslandPrintEntity::Fill(FillExtrusionEntity::Path(FillExtrusionPath {
            polyline: Polyline3::from_xy(
                Polyline::new(vec![Point::new(0, 0), Point::new(1_000_000, 0)]),
                Vec::new(),
            ),
            z_contoured: false,
            role: ExtrusionRole::InternalInfill,
            mm3_per_mm: 0.04,
            width: 0.4,
            height: 0.2,
        }))
    };
    let regions = [
        RegionOptions {
            sparse_infill_filament_id: OrcaInt(1),
            ..RegionOptions::from_base(&Default::default())
        },
        RegionOptions {
            sparse_infill_filament_id: OrcaInt(2),
            ..RegionOptions::from_base(&Default::default())
        },
    ];
    let mut layer = OrderedExtrusionLayer {
        tool_regions: Vec::new(),
        islands: vec![OrderedExtrusionIsland {
            entities: (0..4).map(|_| entity()).collect(),
            perimeter_regions: Vec::new(),
            infill_regions: vec![1, 1, 0, 1],
            infill_collections: vec![2, 2, 7, 9],
        }],
    };
    let mut other_copy = layer.clone();
    let wiped = split(
        &mut other_copy,
        &[0, 1],
        |region| &regions[region],
        |id| matches!(id, CollectionId::Infill(7 | 2)).then_some(0),
    );
    assert_eq!(wiped[&(0, true)].islands[0].infill_collections, [2, 2, 7]);
    assert_eq!(wiped[&(0, true)].islands[0].infill_regions, [1, 1, 0]);
    assert_eq!(wiped[&(1, false)].islands[0].infill_collections, [9]);
    let buckets = split(&mut layer, &[0, 1], |region| &regions[region], |_| None);
    assert_eq!(buckets[&(0, false)].islands[0].infill_regions, [0]);
    assert_eq!(buckets[&(0, false)].islands[0].infill_collections, [7]);
    assert_eq!(buckets[&(1, false)].islands[0].infill_regions, [1, 1, 1]);
    assert_eq!(
        buckets[&(1, false)].islands[0].infill_collections,
        [2, 2, 9]
    );
    assert_eq!(buckets[&(1, false)].islands[0].entities.len(), 3);
}
