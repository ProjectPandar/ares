use super::super::generate_modifier_parent_scan_state::{
    StagedGenerateModifierParentRegion, StagedGenerateModifierParentScanInput,
    staged_generate_modifier_parent_scan,
};
use super::super::model_volume_state::StagedModelVolumeType;
use super::super::volume_cache_state::{StagedExtentBox, StagedVolumeExtents};
fn bbox(a: f32, b: f32) -> StagedExtentBox {
    StagedExtentBox::new([a, 0.0, 0.0], [b, 1.0, 1.0])
}
fn extent(id: u64, a: f32, b: f32) -> StagedVolumeExtents {
    StagedVolumeExtents::new(id, bbox(a, b))
}
fn region(id: u64, t: StagedModelVolumeType, p: isize) -> StagedGenerateModifierParentRegion {
    StagedGenerateModifierParentRegion::new(id, t, p)
}
fn input(t: StagedModelVolumeType, b: StagedExtentBox) -> StagedGenerateModifierParentScanInput {
    StagedGenerateModifierParentScanInput::new(99, t, b)
}
#[test]
fn generate_modifier_parent_scan_records_descending_intersections() {
    let s = staged_generate_modifier_parent_scan(
        input(StagedModelVolumeType::ParameterModifier, bbox(0.5, 2.5)),
        &[
            region(10, StagedModelVolumeType::ModelPart, -1),
            region(20, StagedModelVolumeType::ModelPart, -1),
            region(30, StagedModelVolumeType::ParameterModifier, 1),
        ],
        &[
            extent(10, 10.0, 11.0),
            extent(20, 0.0, 1.0),
            extent(30, 2.0, 3.0),
        ],
    );
    assert!(!s.added());
    assert_eq!(s.parent_model_part_id(), -1);
    assert_eq!(s.scanned_parent_ids(), &[2, 1, 0]);
    assert_eq!(
        s.intersecting_parents()
            .iter()
            .map(|p| p.parent_region_id())
            .collect::<Vec<_>>(),
        vec![2, 1]
    );
}
#[test]
fn generate_modifier_parent_scan_skips_ineligible_parent_types() {
    let s = staged_generate_modifier_parent_scan(
        input(StagedModelVolumeType::ParameterModifier, bbox(0.0, 5.0)),
        &[
            region(10, StagedModelVolumeType::Invalid, -1),
            region(20, StagedModelVolumeType::NegativeVolume, -1),
            region(30, StagedModelVolumeType::SupportBlocker, -1),
            region(40, StagedModelVolumeType::ModelPart, -1),
        ],
        &[
            extent(10, 0.0, 1.0),
            extent(20, 0.0, 1.0),
            extent(30, 0.0, 1.0),
            extent(40, 0.0, 1.0),
        ],
    );
    assert_eq!(s.scanned_parent_ids(), &[3]);
    assert_eq!(s.intersecting_parents()[0].parent_region_id(), 3);
}
#[test]
fn generate_modifier_parent_scan_omits_disjoint_parent_bboxes() {
    let s = staged_generate_modifier_parent_scan(
        input(StagedModelVolumeType::ParameterModifier, bbox(5.0, 6.0)),
        &[
            region(10, StagedModelVolumeType::ModelPart, -1),
            region(20, StagedModelVolumeType::ModelPart, -1),
        ],
        &[extent(10, 0.0, 1.0), extent(20, 5.5, 7.0)],
    );
    assert_eq!(s.scanned_parent_ids(), &[1, 0]);
    assert_eq!(s.intersecting_parents().len(), 1);
    assert_eq!(s.intersecting_parents()[0].parent_region_id(), 1);
    assert_eq!(s.intersecting_parents()[0].parent_bbox(), bbox(5.5, 7.0));
}
#[test]
fn generate_modifier_parent_scan_extends_modifier_parent_chain_bbox() {
    let s = staged_generate_modifier_parent_scan(
        input(StagedModelVolumeType::ParameterModifier, bbox(0.0, 1.5)),
        &[
            region(10, StagedModelVolumeType::ModelPart, -1),
            region(20, StagedModelVolumeType::ParameterModifier, 0),
        ],
        &[extent(10, 0.0, 1.0), extent(20, 10.0, 11.0)],
    );
    assert_eq!(s.scanned_parent_ids(), &[1, 0]);
    assert_eq!(s.intersecting_parents()[0].parent_region_id(), 1);
    assert_eq!(s.intersecting_parents()[0].parent_bbox(), bbox(0.0, 11.0));
}
#[test]
fn generate_modifier_parent_scan_non_modifier_current_is_noop() {
    let s = staged_generate_modifier_parent_scan(
        input(StagedModelVolumeType::ModelPart, bbox(0.0, 1.0)),
        &[region(10, StagedModelVolumeType::ModelPart, -1)],
        &[extent(10, 0.0, 1.0)],
    );
    assert!(!s.added());
    assert_eq!(s.parent_model_part_id(), -1);
    assert!(s.scanned_parent_ids().is_empty());
    assert!(s.intersecting_parents().is_empty());
}
