use serde_json::json;

use super::support::{KsrArchive, metadata};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";

#[tokio::test]
async fn first_layer_bounds_cover_the_outer_skirt_extrusion_width() {
    let mut archive = KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(PROJECT_SETTINGS)).unwrap();
    let settings = settings.as_object_mut().unwrap();
    settings.insert(
        "machine_start_gcode".to_owned(),
        json!(";BOUNDS {first_layer_print_min[0]},{first_layer_print_min[1]} {first_layer_print_max[0]},{first_layer_print_max[1]}"),
    );
    settings.insert("machine_end_gcode".to_owned(), json!(";END"));
    settings.insert("skirt_loops".to_owned(), json!("2"));
    settings.insert("skirt_height".to_owned(), json!("1"));
    settings.insert("skirt_distance".to_owned(), json!("0"));
    settings.insert("min_skirt_length".to_owned(), json!("0"));
    archive.insert_text(PROJECT_SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(&archive.bytes(), metadata())
        .await
        .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == ";BOUNDS 94.625,80.9779 170.553,151.906")
    );
}
