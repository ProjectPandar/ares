use super::super::apply_scarf_joint_seam_state::{
    StagedApplyScarfJointSeamObject, StagedSeamScarfType, staged_apply_scarf_joint_seam,
};

fn object(seam: StagedSeamScarfType) -> StagedApplyScarfJointSeamObject {
    StagedApplyScarfJointSeamObject::new(seam, Vec::new(), Vec::new())
}

#[test]
fn apply_scarf_joint_seam_empty_objects_stays_false_without_set() {
    let result = staged_apply_scarf_joint_seam(&[]);

    assert_eq!(result.queried_key(), "seam_slope_type");
    assert!(!result.has_scarf_joint_seam());
    assert!(result.config_set().is_none());
}

#[test]
fn apply_scarf_joint_seam_records_source_keys() {
    let result = staged_apply_scarf_joint_seam(&[object(StagedSeamScarfType::External)]);

    assert_eq!(result.queried_key(), "seam_slope_type");
    assert_eq!(result.config_set().unwrap().key(), "has_scarf_joint_seam");
    assert!(result.config_set().unwrap().value());
}

#[test]
fn apply_scarf_joint_seam_object_external_sets_true() {
    let result = staged_apply_scarf_joint_seam(&[object(StagedSeamScarfType::External)]);

    assert!(result.has_scarf_joint_seam());
    assert!(result.config_set().is_some());
}

#[test]
fn apply_scarf_joint_seam_object_all_sets_true() {
    let result = staged_apply_scarf_joint_seam(&[object(StagedSeamScarfType::All)]);

    assert!(result.has_scarf_joint_seam());
    assert!(result.config_set().is_some());
}

#[test]
fn apply_scarf_joint_seam_object_none_stays_false_without_other_sources() {
    let result = staged_apply_scarf_joint_seam(&[object(StagedSeamScarfType::None)]);

    assert!(!result.has_scarf_joint_seam());
    assert!(result.config_set().is_none());
}

#[test]
fn apply_scarf_joint_seam_volume_missing_or_none_stays_false() {
    let result = staged_apply_scarf_joint_seam(&[StagedApplyScarfJointSeamObject::new(
        StagedSeamScarfType::None,
        vec![None, Some(StagedSeamScarfType::None)],
        Vec::new(),
    )]);

    assert!(!result.has_scarf_joint_seam());
    assert!(result.config_set().is_none());
}

#[test]
fn apply_scarf_joint_seam_volume_external_sets_true() {
    let result = staged_apply_scarf_joint_seam(&[StagedApplyScarfJointSeamObject::new(
        StagedSeamScarfType::None,
        vec![Some(StagedSeamScarfType::External)],
        Vec::new(),
    )]);

    assert!(result.has_scarf_joint_seam());
    assert!(result.config_set().is_some());
}

#[test]
fn apply_scarf_joint_seam_layer_range_all_sets_true() {
    let result = staged_apply_scarf_joint_seam(&[StagedApplyScarfJointSeamObject::new(
        StagedSeamScarfType::None,
        Vec::new(),
        vec![Some(StagedSeamScarfType::All)],
    )]);

    assert!(result.has_scarf_joint_seam());
    assert!(result.config_set().is_some());
}

#[test]
fn apply_scarf_joint_seam_multiple_active_sources_emit_one_set_record() {
    let result = staged_apply_scarf_joint_seam(&[StagedApplyScarfJointSeamObject::new(
        StagedSeamScarfType::External,
        vec![Some(StagedSeamScarfType::All)],
        vec![Some(StagedSeamScarfType::External)],
    )]);

    assert!(result.has_scarf_joint_seam());
    assert!(result.config_set().is_some());
}
