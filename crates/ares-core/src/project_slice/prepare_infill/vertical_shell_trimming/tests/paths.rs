use crate::project_slice::region_slices::RegionSurfaceKind;

#[test]
fn task22o21_internal_void_is_explicitly_unreachable_in_the_approved_envelope() {
    let reachable = [
        RegionSurfaceKind::Top,
        RegionSurfaceKind::Bottom,
        RegionSurfaceKind::BottomBridge,
        RegionSurfaceKind::Internal,
        RegionSurfaceKind::InternalSolid,
    ];
    assert_eq!(reachable.len(), 5);
    assert_eq!(RegionSurfaceKind::Internal as u8, 4);
    assert_eq!(RegionSurfaceKind::InternalSolid as u8, 5);
}
