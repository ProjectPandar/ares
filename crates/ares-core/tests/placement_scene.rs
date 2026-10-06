//! `profile_placed_project`: loads with profiles and applies placement,
//! returning the same project a center-placed slice would use. The scene
//! transforms must match the G-code model position.

use ares_core::{Placement, ProfileSliceRequest, profile_placed_project};
use serde_json::{Value, json};

fn catalog_parts() -> (Value, Value) {
    let machine = json!({
        "type": "machine", "name": "Acme A1 0.4 nozzle", "from": "system",
        "instantiation": "true", "nozzle_diameter": [0.4],
        "printable_area": ["0x0", "256x0", "256x256", "0x256"],
        "use_relative_e_distances": false
    });
    let process = json!({
        "type": "process", "name": "0.20mm Standard @A1", "from": "system",
        "instantiation": "true", "layer_height": "0.2"
    });
    let filament = json!({
        "type": "filament", "name": "Acme PLA", "from": "system",
        "instantiation": "true", "filament_diameter": [1.75]
    });
    let vendor = json!({
        "index": {
            "name": "Acme",
            "machine_list": [{"sub_path": "machine.json"}],
            "process_list": [{"sub_path": "process.json"}],
            "filament_list": [{"sub_path": "filament.json"}]
        },
        "files": {
            "machine.json": machine,
            "process.json": process,
            "filament.json": filament
        }
    });
    let catalog = json!({"vendors": [vendor], "imports": []});
    let selection = json!({
        "machine": "Acme A1 0.4 nozzle",
        "process": "0.20mm Standard @A1",
        "filaments": ["Acme PLA"]
    });
    (catalog, selection)
}

fn stl() -> Vec<u8> {
    b"solid square\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0.2\nvertex 0 1 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 -1 0.2\nvertex 1 0 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex -1 0 0.2\nvertex 0 -1 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 1 0.2\nvertex -1 0 0.2\nendloop\nendfacet\nendsolid square".to_vec()
}

#[tokio::test]
async fn placed_scene_centers_the_model_on_the_bed() {
    let (catalog, selection) = catalog_parts();
    let catalog_json = catalog.to_string();
    let selection_json = selection.to_string();
    let overrides = ares_core::ConfigOverrides::default();
    let project = profile_placed_project(
        stl(),
        ProfileSliceRequest {
            filename: "square.stl",
            catalog_json: &catalog_json,
            selection_json: &selection_json,
            plate: None,
            overrides: &overrides,
            placement: Placement::Center,
        },
    )
    .await
    .unwrap();

    // The instance transform must place the model at the bed center (128,128)
    // with Z grounded. Verify via the object's instance transform applied to
    // the mesh center.
    let object = &project.objects()[0];
    let instance = &object.instances()[0];
    let transform = instance.transform();
    // The mesh is centered around (0,0) after from_stl_with_profiles shifts;
    // the instance transform carries the placement offset.
    let translation = crate::transform_translation(&transform);
    assert!(
        (translation[0] - 128.0).abs() < 1e-6,
        "bed center X: {translation:?}"
    );
    assert!(
        (translation[1] - 128.0).abs() < 1e-6,
        "bed center Y: {translation:?}"
    );
    // The bed shape is available through the composed settings.
    let area = &project.settings().printer.remaining.printable_area;
    assert_eq!(area.0.len(), 4, "bed shape has 4 corners");
}

#[tokio::test]
async fn placed_scene_none_keeps_original_position() {
    let (catalog, selection) = catalog_parts();
    let catalog_json = catalog.to_string();
    let selection_json = selection.to_string();
    let overrides = ares_core::ConfigOverrides::default();
    let project = profile_placed_project(
        stl(),
        ProfileSliceRequest {
            filename: "square.stl",
            catalog_json: &catalog_json,
            selection_json: &selection_json,
            plate: None,
            overrides: &overrides,
            placement: Placement::None,
        },
    )
    .await
    .unwrap();
    // With no placement, the STL's own coordinates are preserved.
    let object = &project.objects()[0];
    let instance = &object.instances()[0];
    let transform = instance.transform();
    let translation = crate::transform_translation(&transform);
    // The mesh spans roughly [-1,1]; the volume's own center is at origin.
    // from_stl_with_profiles puts the instance at the mesh center offset.
    // For a mesh centered near origin, the translation is near origin.
    assert!(
        translation[0].abs() < 2.0 && translation[1].abs() < 2.0,
        "no placement: near origin, got {translation:?}"
    );
}

fn transform_translation(matrix: &ares_core::Transform3d) -> [f64; 3] {
    let p = matrix.transform_point(ares_core::Point3d::new(0.0, 0.0, 0.0));
    [p.x, p.y, p.z]
}
