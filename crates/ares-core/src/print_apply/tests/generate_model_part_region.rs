use super::super::generate_model_part_region_state::{
    StagedGenerateModelPartLayer, StagedGenerateModelPartVolume,
    staged_generate_model_part_volume_regions,
};
use super::super::generate_regions_state::{
    StagedGeneratePrintObjectRegions, StagedGenerateRegionConfigKey, StagedGenerateRegionSet,
};
use super::super::model_volume_state::StagedModelVolumeType;
use super::super::volume_cache_state::{StagedExtentBox, StagedVolumeExtents};

fn bbox(marker: f32) -> StagedExtentBox {
    StagedExtentBox::new([marker, 0.0, 0.0], [marker, 1.0, 1.0])
}

fn extent(volume_id: u64, marker: f32) -> StagedVolumeExtents {
    StagedVolumeExtents::new(volume_id, bbox(marker))
}

fn volume(
    id: u64,
    volume_type: StagedModelVolumeType,
    config: u64,
) -> StagedGenerateModelPartVolume {
    StagedGenerateModelPartVolume::new(
        id,
        volume_type,
        StagedGenerateRegionConfigKey::new(config, config),
    )
}

fn layer(extents: Vec<StagedVolumeExtents>) -> StagedGenerateModelPartLayer {
    StagedGenerateModelPartLayer::new(extents)
}

#[test]
fn generate_model_part_volume_regions_model_part_creates_region() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut layers = vec![layer(vec![extent(10, 1.0)])];
    let mut region_set = StagedGenerateRegionSet::new();

    staged_generate_model_part_volume_regions(
        &mut shell,
        &mut layers,
        &[volume(10, StagedModelVolumeType::ModelPart, 100)],
        &mut region_set,
    );

    let regions = layers[0].volume_regions();
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].volume_id(), 10);
    assert_eq!(regions[0].parent(), -1);
    assert_eq!(regions[0].region_id(), 0);
    assert_eq!(regions[0].bbox(), bbox(1.0));
    assert_eq!(shell.all_regions().len(), 1);
}

#[test]
fn generate_model_part_volume_regions_defers_negative_and_modifier() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut layers = vec![layer(vec![extent(20, 2.0), extent(30, 3.0)])];
    let mut region_set = StagedGenerateRegionSet::new();

    staged_generate_model_part_volume_regions(
        &mut shell,
        &mut layers,
        &[
            volume(20, StagedModelVolumeType::NegativeVolume, 200),
            volume(30, StagedModelVolumeType::ParameterModifier, 300),
        ],
        &mut region_set,
    );

    assert!(layers[0].volume_regions().is_empty());
    assert!(shell.all_regions().is_empty());
}

#[test]
fn generate_model_part_volume_regions_uses_per_layer_extent_gate() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut layers = vec![layer(vec![extent(10, 1.0)]), layer(vec![extent(20, 2.0)])];
    let mut region_set = StagedGenerateRegionSet::new();

    staged_generate_model_part_volume_regions(
        &mut shell,
        &mut layers,
        &[
            volume(10, StagedModelVolumeType::ModelPart, 100),
            volume(20, StagedModelVolumeType::ModelPart, 200),
        ],
        &mut region_set,
    );

    assert_eq!(layers[0].volume_regions().len(), 1);
    assert_eq!(layers[0].volume_regions()[0].volume_id(), 10);
    assert_eq!(layers[0].volume_regions()[0].bbox(), bbox(1.0));
    assert_eq!(layers[1].volume_regions().len(), 1);
    assert_eq!(layers[1].volume_regions()[0].volume_id(), 20);
    assert_eq!(layers[1].volume_regions()[0].bbox(), bbox(2.0));
}

#[test]
fn generate_model_part_volume_regions_skips_unsupported_volume_types() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut layers = vec![layer(vec![
        extent(10, 1.0),
        extent(20, 2.0),
        extent(30, 3.0),
    ])];
    let mut region_set = StagedGenerateRegionSet::new();

    staged_generate_model_part_volume_regions(
        &mut shell,
        &mut layers,
        &[
            volume(10, StagedModelVolumeType::Invalid, 100),
            volume(20, StagedModelVolumeType::SupportBlocker, 200),
            volume(30, StagedModelVolumeType::SupportEnforcer, 300),
        ],
        &mut region_set,
    );

    assert!(layers[0].volume_regions().is_empty());
    assert!(shell.all_regions().is_empty());
}

#[test]
fn generate_model_part_volume_regions_preserves_model_order_per_layer() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut layers = vec![
        layer(vec![extent(10, 1.0), extent(20, 2.0)]),
        layer(vec![extent(20, 3.0), extent(30, 4.0)]),
    ];
    let mut region_set = StagedGenerateRegionSet::new();

    staged_generate_model_part_volume_regions(
        &mut shell,
        &mut layers,
        &[
            volume(10, StagedModelVolumeType::ModelPart, 100),
            volume(20, StagedModelVolumeType::ModelPart, 200),
            volume(30, StagedModelVolumeType::ModelPart, 300),
        ],
        &mut region_set,
    );

    let first_layer_ids: Vec<u64> = layers[0]
        .volume_regions()
        .iter()
        .map(|region| region.volume_id())
        .collect();
    let second_layer_ids: Vec<u64> = layers[1]
        .volume_regions()
        .iter()
        .map(|region| region.volume_id())
        .collect();

    assert_eq!(first_layer_ids, vec![10, 20]);
    assert_eq!(second_layer_ids, vec![20, 30]);
}

#[test]
fn generate_model_part_volume_regions_reuses_equal_configs() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut layers = vec![layer(vec![extent(10, 1.0)]), layer(vec![extent(10, 2.0)])];
    let mut region_set = StagedGenerateRegionSet::new();

    staged_generate_model_part_volume_regions(
        &mut shell,
        &mut layers,
        &[volume(10, StagedModelVolumeType::ModelPart, 100)],
        &mut region_set,
    );

    assert_eq!(layers[0].volume_regions()[0].region_id(), 0);
    assert_eq!(layers[1].volume_regions()[0].region_id(), 0);
    assert_eq!(shell.all_regions().len(), 1);
}

#[test]
fn generate_model_part_volume_regions_creates_distinct_regions_for_distinct_configs() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut layers = vec![layer(vec![extent(10, 1.0), extent(20, 2.0)])];
    let mut region_set = StagedGenerateRegionSet::new();

    staged_generate_model_part_volume_regions(
        &mut shell,
        &mut layers,
        &[
            volume(10, StagedModelVolumeType::ModelPart, 100),
            volume(20, StagedModelVolumeType::ModelPart, 200),
        ],
        &mut region_set,
    );

    assert_eq!(layers[0].volume_regions()[0].region_id(), 0);
    assert_eq!(layers[0].volume_regions()[1].region_id(), 1);
    assert_eq!(shell.all_regions().len(), 2);
}

#[test]
fn generate_model_part_volume_regions_preserves_parent_and_bbox_identity() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut layers = vec![layer(vec![extent(42, 9.0)])];
    let mut region_set = StagedGenerateRegionSet::new();

    staged_generate_model_part_volume_regions(
        &mut shell,
        &mut layers,
        &[volume(42, StagedModelVolumeType::ModelPart, 400)],
        &mut region_set,
    );

    let region = &layers[0].volume_regions()[0];
    assert_eq!(region.volume_id(), 42);
    assert_eq!(region.parent(), -1);
    assert_eq!(region.region_id(), 0);
    assert_eq!(region.bbox(), bbox(9.0));
}
