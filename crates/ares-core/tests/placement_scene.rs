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
        "id": "Acme",
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
    let request = ProfileSliceRequest {
        filename: "square.stl",
        catalog_json: &catalog_json,
        selection_json: &selection_json,
        plate: None,
        overrides: &overrides,
        placement: Placement::Center,
    };
    let project = profile_placed_project(stl(), request).await.unwrap();

    // The instance transform must place the model at the bed center (128,128)
    // with Z grounded.
    let object = &project.objects()[0];
    let instance = &object.instances()[0];
    let transform = instance.transform();
    let translation = transform_translation(&transform);
    assert!(
        (translation[0] - 128.0).abs() < 1e-6,
        "bed center X: {translation:?}"
    );
    assert!(
        (translation[1] - 128.0).abs() < 1e-6,
        "bed center Y: {translation:?}"
    );

    // Acceptance: the placed scene's mesh bbox center must match the model
    // position in the G-code produced by slicing the same input with the
    // same request. The G-code's first-layer extrusion perimeter extends
    // outward by half the extrusion width from the model edge, so the
    // perimeter bbox center equals the model bbox center; 0.5 mm covers
    // the line width (0.42 mm nozzle) and first-layer rounding.
    let gcode = ares_core::slice_with_profiles(
        stl(),
        ProfileSliceRequest {
            filename: "square.stl",
            catalog_json: &catalog_json,
            selection_json: &selection_json,
            plate: None,
            overrides: &overrides,
            placement: Placement::Center,
        },
        ares_core::GenerationMetadata::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let gcode_text = String::from_utf8(gcode).unwrap();
    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    for (x, y) in gcode_text
        .lines()
        .filter_map(|l| l.strip_prefix("G1 "))
        .filter_map(extrusion_xy)
    {
        min[0] = min[0].min(x);
        max[0] = max[0].max(x);
        min[1] = min[1].min(y);
        max[1] = max[1].max(y);
    }
    let gcode_center = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];

    // Mesh bbox center after the placed instance transform.
    let volume = &object.volumes()[0];
    let mesh = volume.mesh();
    let mut mesh_min = [f64::INFINITY; 2];
    let mut mesh_max = [f64::NEG_INFINITY; 2];
    for vertex in mesh.vertices() {
        let p = transform.transform_point(*vertex);
        mesh_min[0] = mesh_min[0].min(p.x);
        mesh_max[0] = mesh_max[0].max(p.x);
        mesh_min[1] = mesh_min[1].min(p.y);
        mesh_max[1] = mesh_max[1].max(p.y);
    }
    let mesh_center = [
        (mesh_min[0] + mesh_max[0]) * 0.5,
        (mesh_min[1] + mesh_max[1]) * 0.5,
    ];

    assert!(
        (gcode_center[0] - mesh_center[0]).abs() < 0.5,
        "X center: gcode={gcode_center:?} mesh={mesh_center:?}"
    );
    assert!(
        (gcode_center[1] - mesh_center[1]).abs() < 0.5,
        "Y center: gcode={gcode_center:?} mesh={mesh_center:?}"
    );

    // Bed shape is available through the composed settings.
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
    let translation = transform_translation(&transform);
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

/// Extracts the X/Y coordinates from a G1 extrusion move.
fn extrusion_xy(rest: &str) -> Option<(f64, f64)> {
    let mut x = None;
    let mut y = None;
    for word in rest.split_whitespace() {
        if let Some(v) = word.strip_prefix('X') {
            x = v.parse::<f64>().ok();
        } else if let Some(v) = word.strip_prefix('Y') {
            y = v.parse::<f64>().ok();
        }
    }
    match (x, y) {
        (Some(x), Some(y)) => Some((x, y)),
        _ => None,
    }
}
