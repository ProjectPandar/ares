use super::*;
use crate::{
    OrcaInt,
    geometry::{Point, Polyline},
    project_slice::fill_entities::{FillExtrusionEntity, FillExtrusionPath},
};

#[test]
fn tool_buckets_preserve_source_collection_identity_and_region() {
    let entity = || {
        IslandPrintEntity::Fill(FillExtrusionEntity::Path(FillExtrusionPath {
            polyline: Polyline::new(vec![Point::new(0, 0), Point::new(1_000_000, 0)]),
            fitting: Vec::new(),
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
    let buckets = split(&mut layer, &[0, 1], |region| &regions[region]);
    assert_eq!(buckets[&0].islands[0].infill_regions, [0]);
    assert_eq!(buckets[&0].islands[0].infill_collections, [7]);
    assert_eq!(buckets[&1].islands[0].infill_regions, [1, 1, 1]);
    assert_eq!(buckets[&1].islands[0].infill_collections, [2, 2, 9]);
    assert_eq!(buckets[&1].islands[0].entities.len(), 3);
}
