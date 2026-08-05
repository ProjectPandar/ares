use super::super::RegionSurfaceKind;

#[test]
fn task22o18_surface_kind_values_and_bridge_predicate_match_source() {
    assert_eq!(RegionSurfaceKind::Top as u8, 0);
    assert_eq!(RegionSurfaceKind::Bottom as u8, 1);
    assert_eq!(RegionSurfaceKind::BottomBridge as u8, 2);
    assert_eq!(RegionSurfaceKind::Internal as u8, 4);
    assert_eq!(RegionSurfaceKind::InternalSolid as u8, 5);
    assert_eq!(RegionSurfaceKind::InternalVoid as u8, 8);
    assert!(RegionSurfaceKind::BottomBridge.is_bridge());
    for kind in [
        RegionSurfaceKind::Top,
        RegionSurfaceKind::Bottom,
        RegionSurfaceKind::Internal,
        RegionSurfaceKind::InternalSolid,
        RegionSurfaceKind::InternalVoid,
    ] {
        assert!(!kind.is_bridge());
    }
}
