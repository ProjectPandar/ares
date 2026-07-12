use super::super::generate_painted_region_state::{
    StagedGeneratePaintedParentVolumeRegion, staged_generate_painted_regions,
};
use super::super::generate_regions_state::{
    StagedGeneratePrintObjectRegions, StagedGenerateRegionSet,
};
use super::super::model_volume_state::StagedModelVolumeType;

fn parent(
    volume_type: StagedModelVolumeType,
    marker: u64,
) -> StagedGeneratePaintedParentVolumeRegion {
    StagedGeneratePaintedParentVolumeRegion::new(volume_type, marker)
}

#[test]
fn generate_painted_region_appends_for_model_part_and_modifier_parents() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_painted_regions(
        &mut shell,
        &[
            parent(StagedModelVolumeType::ModelPart, 10),
            parent(StagedModelVolumeType::ParameterModifier, 20),
        ],
        &[3],
        &mut region_set,
    );

    assert_eq!(regions.len(), 2);
    assert_eq!(regions[0].extruder_id(), 3);
    assert_eq!(regions[0].parent(), 0);
    assert_eq!(regions[0].derived_config().marker(), 10);
    assert_eq!(regions[0].derived_config().filaments(), (3, 3, 3));
    assert_eq!(regions[1].extruder_id(), 3);
    assert_eq!(regions[1].parent(), 1);
    assert_eq!(regions[1].derived_config().marker(), 20);
    assert_eq!(shell.all_regions().len(), 2);
}

#[test]
fn generate_painted_region_skips_ineligible_parent_types() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_painted_regions(
        &mut shell,
        &[
            parent(StagedModelVolumeType::NegativeVolume, 10),
            parent(StagedModelVolumeType::SupportBlocker, 20),
            parent(StagedModelVolumeType::ModelPart, 30),
        ],
        &[4],
        &mut region_set,
    );

    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].parent(), 2);
    assert_eq!(regions[0].derived_config().marker(), 30);
    assert_eq!(shell.all_regions().len(), 1);
}

#[test]
fn generate_painted_region_preserves_nested_iteration_order() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_painted_regions(
        &mut shell,
        &[
            parent(StagedModelVolumeType::ModelPart, 10),
            parent(StagedModelVolumeType::ParameterModifier, 20),
        ],
        &[8, 9],
        &mut region_set,
    );

    let order: Vec<(u32, usize)> = regions
        .iter()
        .map(|region| (region.extruder_id(), region.parent()))
        .collect();
    assert_eq!(order, vec![(8, 0), (8, 1), (9, 0), (9, 1)]);
}

#[test]
fn generate_painted_region_reuses_equal_derived_configs() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_painted_regions(
        &mut shell,
        &[
            parent(StagedModelVolumeType::ModelPart, 10),
            parent(StagedModelVolumeType::ParameterModifier, 10),
        ],
        &[7],
        &mut region_set,
    );

    assert_eq!(regions[0].region_id(), 0);
    assert_eq!(regions[1].region_id(), 0);
    assert_eq!(shell.all_regions().len(), 1);
}

#[test]
fn generate_painted_region_empty_extruders_is_noop() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_painted_regions(
        &mut shell,
        &[parent(StagedModelVolumeType::ModelPart, 10)],
        &[],
        &mut region_set,
    );

    assert!(regions.is_empty());
    assert!(shell.all_regions().is_empty());
}

#[test]
fn generate_painted_region_empty_parents_is_noop() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let regions = staged_generate_painted_regions(&mut shell, &[], &[7], &mut region_set);

    assert!(regions.is_empty());
    assert!(shell.all_regions().is_empty());
}
