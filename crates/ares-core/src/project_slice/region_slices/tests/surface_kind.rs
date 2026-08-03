use super::super::RegionSurfaceKind;

#[test]
fn task22o17_surface_kind_values_and_bridge_predicate_match_source() {
    assert_eq!(RegionSurfaceKind::Top as u8, 0);
    assert_eq!(RegionSurfaceKind::Bottom as u8, 1);
    assert_eq!(RegionSurfaceKind::BottomBridge as u8, 2);
    assert_eq!(RegionSurfaceKind::Internal as u8, 4);
    assert!(RegionSurfaceKind::BottomBridge.is_bridge());
    for kind in [
        RegionSurfaceKind::Top,
        RegionSurfaceKind::Bottom,
        RegionSurfaceKind::Internal,
    ] {
        assert!(!kind.is_bridge());
    }
}
