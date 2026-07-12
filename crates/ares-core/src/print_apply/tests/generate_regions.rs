use super::super::generate_regions_state::{
    StagedGenerateLayerRange, StagedGenerateModelVolumePaint, StagedGeneratePrintObjectRegions,
    StagedGeneratePrintObjectRegionsLayer, StagedGeneratePrintRegion,
    StagedGenerateRegionConfigKey, StagedGenerateRegionSet,
    staged_generate_print_object_regions_layer_range_shell,
    staged_generate_print_object_regions_update_volume_bboxes_call,
};
use super::super::mesh_state::StagedLayerHeightRange;
use super::super::volume_cache_state::{StagedExtentBox, StagedVolumeExtents};

fn range(first: f64, second: f64) -> StagedLayerHeightRange {
    StagedLayerHeightRange::new(first, second)
}

fn model_range(first: f64, second: f64, config_id: u64) -> StagedGenerateLayerRange {
    StagedGenerateLayerRange::new(range(first, second), Some(config_id))
}

fn extent(volume_id: u64, marker: f32) -> StagedVolumeExtents {
    StagedVolumeExtents::new(
        volume_id,
        StagedExtentBox::new([marker, 0.0, 0.0], [marker, 1.0, 1.0]),
    )
}

fn old_layer(
    first: f64,
    second: f64,
    config_id: u64,
    volumes: Vec<StagedVolumeExtents>,
    region_id: u64,
) -> StagedGeneratePrintObjectRegionsLayer {
    StagedGeneratePrintObjectRegionsLayer::new(
        range(first, second),
        Some(config_id),
        volumes,
        (
            vec![region_id],
            vec![region_id + 100],
            vec![region_id + 200],
        ),
    )
}

fn old_shell() -> StagedGeneratePrintObjectRegions {
    StagedGeneratePrintObjectRegions::new(
        vec![
            StagedGeneratePrintRegion::new(0, StagedGenerateRegionConfigKey::new(101, 0)),
            StagedGeneratePrintRegion::new(1, StagedGenerateRegionConfigKey::new(102, 0)),
        ],
        vec![
            old_layer(0.0, 0.2, 1, vec![extent(10, 1.0)], 201),
            old_layer(0.2, 0.4, 2, vec![extent(20, 2.0)], 202),
        ],
        700,
        vec![11, 22],
    )
}

#[test]
fn generate_print_object_regions_layer_range_shell_fresh_creates_model_ranges_and_transform() {
    let shell = staged_generate_print_object_regions_layer_range_shell(
        None,
        &[model_range(0.0, 0.2, 11), model_range(0.2, 0.4, 22)],
        900,
    );

    assert!(shell.all_regions().is_empty());
    assert!(shell.cached_volume_ids().is_empty());
    assert_eq!(shell.trafo_bboxes(), 900);
    assert_eq!(shell.layer_ranges().len(), 2);
    assert_eq!(
        shell.layer_ranges()[0].layer_height_range(),
        range(0.0, 0.2)
    );
    assert_eq!(shell.layer_ranges()[0].config_id(), Some(11));
    assert!(shell.layer_ranges()[0].volumes().is_empty());
    assert_eq!(
        shell.layer_ranges()[1].layer_height_range(),
        range(0.2, 0.4)
    );
    assert_eq!(shell.layer_ranges()[1].config_id(), Some(22));
}

#[test]
fn generate_print_object_regions_layer_range_shell_old_empty_ranges_uses_fresh_path() {
    let old = StagedGeneratePrintObjectRegions::new(
        vec![StagedGeneratePrintRegion::new(
            0,
            StagedGenerateRegionConfigKey::new(101, 0),
        )],
        Vec::new(),
        700,
        vec![10],
    );

    let shell = staged_generate_print_object_regions_layer_range_shell(
        Some(old),
        &[model_range(0.0, 0.2, 11)],
        900,
    );

    assert!(shell.all_regions().is_empty());
    assert_eq!(shell.cached_volume_ids(), &[10]);
    assert_eq!(shell.trafo_bboxes(), 900);
    assert_eq!(shell.layer_ranges().len(), 1);
    assert_eq!(shell.layer_ranges()[0].config_id(), Some(11));
}

#[test]
fn generate_print_object_regions_layer_range_shell_reuse_clears_regions_and_refreshes_config() {
    let shell = staged_generate_print_object_regions_layer_range_shell(
        Some(old_shell()),
        &[model_range(0.0, 0.2, 111), model_range(0.2, 0.4, 222)],
        900,
    );

    assert!(shell.all_regions().is_empty());
    assert_eq!(shell.cached_volume_ids(), &[11, 22]);
    assert_eq!(shell.trafo_bboxes(), 700);
    assert_eq!(shell.layer_ranges()[0].config_id(), Some(111));
    assert_eq!(shell.layer_ranges()[0].volumes(), &[extent(10, 1.0)]);
    assert!(shell.layer_ranges()[0].volume_regions().is_empty());
    assert!(shell.layer_ranges()[0].painted_regions().is_empty());
    assert!(
        shell.layer_ranges()[0]
            .fuzzy_skin_painted_regions()
            .is_empty()
    );
    assert_eq!(shell.layer_ranges()[1].config_id(), Some(222));
    assert_eq!(shell.layer_ranges()[1].volumes(), &[extent(20, 2.0)]);
    assert!(shell.layer_ranges()[1].volume_regions().is_empty());
    assert!(shell.layer_ranges()[1].painted_regions().is_empty());
    assert!(
        shell.layer_ranges()[1]
            .fuzzy_skin_painted_regions()
            .is_empty()
    );
}

#[test]
#[should_panic]
fn generate_print_object_regions_layer_range_shell_reuse_panics_on_count_mismatch() {
    staged_generate_print_object_regions_layer_range_shell(
        Some(old_shell()),
        &[model_range(0.0, 0.2, 111)],
        900,
    );
}

#[test]
#[should_panic]
fn generate_print_object_regions_layer_range_shell_reuse_panics_on_range_mismatch() {
    staged_generate_print_object_regions_layer_range_shell(
        Some(old_shell()),
        &[model_range(0.0, 0.3, 111), model_range(0.2, 0.4, 222)],
        900,
    );
}

#[test]
fn generate_print_object_regions_update_volume_bboxes_call_single_extruder_painted_clamps_negative_offset()
 {
    let call = staged_generate_print_object_regions_update_volume_bboxes_call(
        &old_shell(),
        &[StagedGenerateModelVolumePaint::new(10, true)],
        1,
        -0.25,
    );

    assert!(!call.is_mm_painted());
    assert_eq!(call.offset(), 0.0);
}

#[test]
fn generate_print_object_regions_update_volume_bboxes_call_multi_extruder_unpainted_uses_positive_offset()
 {
    let call = staged_generate_print_object_regions_update_volume_bboxes_call(
        &old_shell(),
        &[
            StagedGenerateModelVolumePaint::new(10, false),
            StagedGenerateModelVolumePaint::new(20, false),
        ],
        2,
        0.35,
    );

    assert!(!call.is_mm_painted());
    assert_eq!(call.offset(), 0.35);
}

#[test]
fn generate_print_object_regions_update_volume_bboxes_call_multi_extruder_painted_zeroes_offset() {
    let call = staged_generate_print_object_regions_update_volume_bboxes_call(
        &old_shell(),
        &[
            StagedGenerateModelVolumePaint::new(10, false),
            StagedGenerateModelVolumePaint::new(20, true),
        ],
        2,
        0.35,
    );

    assert!(call.is_mm_painted());
    assert_eq!(call.offset(), 0.0);
}

#[test]
fn generate_print_object_regions_update_volume_bboxes_call_empty_volumes_are_not_mm_painted() {
    let call =
        staged_generate_print_object_regions_update_volume_bboxes_call(&old_shell(), &[], 2, 0.35);

    assert!(!call.is_mm_painted());
    assert_eq!(call.offset(), 0.35);
}

#[test]
fn generate_print_object_regions_update_volume_bboxes_call_preserves_shell_call_inputs() {
    let call = staged_generate_print_object_regions_update_volume_bboxes_call(
        &old_shell(),
        &[StagedGenerateModelVolumePaint::new(10, false)],
        2,
        0.35,
    );

    assert_eq!(call.trafo_bboxes(), 700);
    assert_eq!(call.cached_volume_ids(), &[11, 22]);
    assert_eq!(call.layer_range_count(), 2);
}

#[test]
fn generate_print_object_regions_region_set_empty_insert_creates_first_region() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let region_id =
        region_set.get_create_region(&mut shell, StagedGenerateRegionConfigKey::new(10, 1));

    assert_eq!(region_id, 0);
    assert_eq!(shell.all_regions().len(), 1);
    assert_eq!(shell.all_regions()[0].id(), 0);
    assert_eq!(shell.all_regions()[0].config_hash(), 10);
    assert_eq!(region_set.region_ids_in_lookup_order(), &[0]);
}

#[test]
fn generate_print_object_regions_region_set_reuses_equal_config() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();
    let config = StagedGenerateRegionConfigKey::new(10, 1);

    let first_id = region_set.get_create_region(&mut shell, config);
    let second_id = region_set.get_create_region(&mut shell, config);

    assert_eq!(first_id, 0);
    assert_eq!(second_id, 0);
    assert_eq!(shell.all_regions().len(), 1);
    assert_eq!(region_set.region_ids_in_lookup_order(), &[0]);
}

#[test]
fn generate_print_object_regions_region_set_hash_collision_creates_distinct_region() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let first_id =
        region_set.get_create_region(&mut shell, StagedGenerateRegionConfigKey::new(10, 1));
    let second_id =
        region_set.get_create_region(&mut shell, StagedGenerateRegionConfigKey::new(10, 2));

    assert_eq!(first_id, 0);
    assert_eq!(second_id, 1);
    assert_eq!(shell.all_regions().len(), 2);
    assert_eq!(region_set.region_ids_in_lookup_order(), &[0, 1]);
}

#[test]
fn generate_print_object_regions_region_set_keeps_lookup_sorted_and_all_regions_appended() {
    let mut shell = StagedGeneratePrintObjectRegions::new(Vec::new(), Vec::new(), 700, Vec::new());
    let mut region_set = StagedGenerateRegionSet::new();

    let high_id =
        region_set.get_create_region(&mut shell, StagedGenerateRegionConfigKey::new(20, 2));
    let low_id =
        region_set.get_create_region(&mut shell, StagedGenerateRegionConfigKey::new(10, 9));
    let middle_id =
        region_set.get_create_region(&mut shell, StagedGenerateRegionConfigKey::new(20, 1));

    assert_eq!((high_id, low_id, middle_id), (0, 1, 2));
    let all_region_ids: Vec<u64> = shell
        .all_regions()
        .iter()
        .map(|region| region.id())
        .collect();
    assert_eq!(all_region_ids, vec![0, 1, 2]);
    assert_eq!(region_set.region_ids_in_lookup_order(), &[1, 2, 0]);
}

#[test]
fn generate_print_object_regions_region_set_existing_regions_assign_next_id() {
    let mut shell = old_shell();
    let mut region_set = StagedGenerateRegionSet::new();

    let region_id =
        region_set.get_create_region(&mut shell, StagedGenerateRegionConfigKey::new(10, 1));

    assert_eq!(region_id, 2);
    assert_eq!(shell.all_regions().len(), 3);
    assert_eq!(shell.all_regions()[2].id(), 2);
}
