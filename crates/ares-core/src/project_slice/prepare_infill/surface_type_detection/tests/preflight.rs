use crate::{
    ObjectOptions, OrcaBool, OrcaFloat, OrcaInt, ProcessSupportType, ProjectSettings,
    project_slice::{
        prepare_infill::surface_type_detection::preflight::bottom_kind,
        region_slices::RegionSurfaceKind,
    },
};

fn options() -> ObjectOptions {
    ObjectOptions::from_base(&ProjectSettings::default().process.object)
}

#[test]
fn source_support_predicate_distinguishes_automatic_support_types() {
    let mut value = options();
    assert_eq!(bottom_kind(&value), RegionSurfaceKind::BottomBridge);
    value.enable_support = OrcaBool(true);
    value.support_top_z_distance = OrcaFloat(0.0);
    assert_eq!(bottom_kind(&value), RegionSurfaceKind::Bottom);
    value.bridge_no_support = OrcaBool(true);
    assert_eq!(bottom_kind(&value), RegionSurfaceKind::BottomBridge);
    value.support_type = ProcessSupportType::TreeAuto;
    value.support_interface_top_layers = OrcaInt(1);
    value.max_bridge_length = OrcaFloat(0.0);
    value.support_critical_regions_only = OrcaBool(false);
    assert_eq!(bottom_kind(&value), RegionSurfaceKind::Bottom);
    value.support_type = ProcessSupportType::TreeManual;
    assert_eq!(bottom_kind(&value), RegionSurfaceKind::BottomBridge);
}

#[test]
fn source_support_predicate_preserves_every_operand() {
    let mut value = options();
    value.enforce_support_layers = OrcaInt(1);
    value.support_top_z_distance = OrcaFloat(0.0);
    assert_eq!(bottom_kind(&value), RegionSurfaceKind::Bottom);
    value.support_top_z_distance = OrcaFloat(0.2);
    assert_eq!(bottom_kind(&value), RegionSurfaceKind::BottomBridge);
    value.support_top_z_distance = OrcaFloat(0.0);
    value.support_type = ProcessSupportType::TreeAuto;
    value.support_interface_top_layers = OrcaInt(0);
    value.max_bridge_length = OrcaFloat(0.0);
    value.support_critical_regions_only = OrcaBool(false);
    assert_eq!(bottom_kind(&value), RegionSurfaceKind::BottomBridge);
    value.support_interface_top_layers = OrcaInt(1);
    value.max_bridge_length = OrcaFloat(1.0);
    assert_eq!(bottom_kind(&value), RegionSurfaceKind::BottomBridge);
    value.max_bridge_length = OrcaFloat(0.0);
    value.support_critical_regions_only = OrcaBool(true);
    assert_eq!(bottom_kind(&value), RegionSurfaceKind::BottomBridge);
    value.support_critical_regions_only = OrcaBool(false);
    assert_eq!(bottom_kind(&value), RegionSurfaceKind::Bottom);
    value.support_type = ProcessSupportType::NormalManual;
    assert_eq!(bottom_kind(&value), RegionSurfaceKind::BottomBridge);
}
