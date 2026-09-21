use super::*;

fn point(x: f64, y: f64) -> Point2 {
    Point2::new(x, y)
}

#[test]
fn classifies_external_perimeter_role() {
    let role = ExtrusionRole::ExternalPerimeter;

    assert!(role.is_perimeter());
    assert!(role.is_external_perimeter());
    assert!(!role.is_internal_perimeter());
}

#[test]
fn classifies_internal_perimeter_role() {
    let role = ExtrusionRole::Perimeter;

    assert!(role.is_perimeter());
    assert!(role.is_internal_perimeter());
    assert!(!role.is_external_perimeter());
}

#[test]
fn classifies_bridge_roles() {
    assert!(ExtrusionRole::BridgeInfill.is_bridge());
    assert!(ExtrusionRole::InternalBridgeInfill.is_bridge());
    assert!(ExtrusionRole::OverhangPerimeter.is_bridge());
    assert!(!ExtrusionRole::InternalInfill.is_bridge());
}

#[test]
fn classifies_top_solid_infill_role() {
    let role = ExtrusionRole::TopSolidInfill;

    assert!(role.is_infill());
    assert!(role.is_solid_infill());
    assert!(role.is_top_surface());
    assert!(!ExtrusionRole::Ironing.is_top_surface());
    assert!(!ExtrusionRole::GapFill.is_infill());
}

#[test]
fn maps_print_path_roles_to_libslic3r_extrusion_roles() {
    assert_eq!(
        ExtrusionRole::from_print_path_role(PrintPathRole::ExternalPerimeter),
        ExtrusionRole::ExternalPerimeter
    );
    assert_eq!(
        ExtrusionRole::from_print_path_role(PrintPathRole::InternalPerimeter),
        ExtrusionRole::Perimeter
    );
    assert_eq!(
        ExtrusionRole::from_print_path_role(PrintPathRole::SparseInfill),
        ExtrusionRole::InternalInfill
    );
    assert_eq!(
        ExtrusionRole::from_print_path_role(PrintPathRole::SolidInfill),
        ExtrusionRole::SolidInfill
    );
    assert_eq!(
        ExtrusionRole::from_print_path_role(PrintPathRole::BottomSurface),
        ExtrusionRole::BottomSurface
    );
    assert_eq!(
        ExtrusionRole::from_print_path_role(PrintPathRole::TopSolidInfill),
        ExtrusionRole::TopSolidInfill
    );
    assert_eq!(
        ExtrusionRole::from_print_path_role(PrintPathRole::Bridge),
        ExtrusionRole::BridgeInfill
    );
    assert_eq!(
        ExtrusionRole::from_print_path_role(PrintPathRole::InternalBridge),
        ExtrusionRole::InternalBridgeInfill
    );
    assert_eq!(
        ExtrusionRole::from_print_path_role(PrintPathRole::GapFill),
        ExtrusionRole::GapFill
    );
    assert_eq!(
        ExtrusionRole::from_print_path_role(PrintPathRole::Skirt),
        ExtrusionRole::Skirt
    );
    assert_eq!(
        ExtrusionRole::from_print_path_role(PrintPathRole::Brim),
        ExtrusionRole::Brim
    );
    assert_eq!(
        ExtrusionRole::from_print_path_role(PrintPathRole::OverhangPerimeter),
        ExtrusionRole::OverhangPerimeter
    );
    assert_eq!(
        ExtrusionRole::from_print_path_role(PrintPathRole::SupportMaterial),
        ExtrusionRole::SupportMaterial
    );
    assert_eq!(
        ExtrusionRole::from_print_path_role(PrintPathRole::SupportMaterialInterface),
        ExtrusionRole::SupportMaterialInterface
    );
    assert_eq!(
        ExtrusionRole::from_print_path_role(PrintPathRole::Ironing),
        ExtrusionRole::Ironing
    );
}

#[test]
fn rejects_empty_extrusion_path() {
    assert_eq!(
        ExtrusionPath::new(ExtrusionRole::Perimeter, Vec::new()),
        Err(SliceError::InvalidInput(
            "extrusion path requires at least one point".to_owned()
        ))
    );
}

#[test]
fn collection_from_paths_preserves_entity_order_and_role() {
    let first = ExtrusionPath::new(
        ExtrusionRole::ExternalPerimeter,
        vec![point(0.0, 0.0), point(1.0, 0.0)],
    )
    .unwrap();
    let second = ExtrusionPath::new(
        ExtrusionRole::InternalInfill,
        vec![point(0.0, 1.0), point(1.0, 1.0)],
    )
    .unwrap();

    let collection = ExtrusionEntityCollection::from_paths(vec![first, second]);

    assert_eq!(collection.len(), 2);
    assert_eq!(
        collection.paths()[0].role(),
        ExtrusionRole::ExternalPerimeter
    );
    assert_eq!(collection.paths()[1].role(), ExtrusionRole::InternalInfill);
}
