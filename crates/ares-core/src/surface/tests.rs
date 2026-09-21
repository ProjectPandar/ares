use super::*;
use crate::Point2;

#[test]
fn top_surface_is_top_external_and_solid() {
    let surface_type = SurfaceType::Top;

    assert!(surface_type.is_top());
    assert!(surface_type.is_external());
    assert!(surface_type.is_solid());
    assert!(!surface_type.is_bottom());
    assert!(!surface_type.is_bridge());
    assert!(!surface_type.is_internal());
}

#[test]
fn bottom_bridge_surface_is_bottom_bridge_external_and_solid() {
    let surface_type = SurfaceType::BottomBridge;

    assert!(surface_type.is_bottom());
    assert!(surface_type.is_bridge());
    assert!(surface_type.is_external());
    assert!(surface_type.is_solid());
    assert!(!surface_type.is_top());
    assert!(!surface_type.is_internal_bridge());
}

#[test]
fn internal_bridge_surface_is_bridge_internal_solid_and_internal_bridge() {
    let surface_type = SurfaceType::InternalBridge;

    assert!(surface_type.is_bridge());
    assert!(surface_type.is_internal());
    assert!(surface_type.is_solid());
    assert!(surface_type.is_internal_bridge());
    assert!(!surface_type.is_external());
    assert!(!surface_type.is_solid_infill());
}

#[test]
fn internal_surface_is_internal_and_not_solid() {
    let surface_type = SurfaceType::Internal;

    assert!(surface_type.is_internal());
    assert!(!surface_type.is_solid());
    assert!(!surface_type.is_external());
    assert!(!surface_type.is_bridge());
}

#[test]
fn new_surface_preserves_contour_and_uses_libslic3r_metadata_defaults() {
    let contour = triangle_contour();
    let surface = Surface::new(SurfaceType::Internal, contour.clone());

    assert_eq!(surface.surface_type(), SurfaceType::Internal);
    assert_eq!(surface.contour(), &contour);
    assert_eq!(surface.thickness(), -1.0);
    assert_eq!(surface.thickness_layers(), 1);
    assert_eq!(surface.bridge_angle(), -1.0);
    assert_eq!(surface.extra_perimeters(), 0);
}

fn triangle_contour() -> Contour {
    Contour::new(vec![
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 0.0),
        Point2::new(0.0, 1.0),
    ])
}
