use serde_json::json;

use super::support::{KsrArchive, metadata};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";

#[tokio::test]
async fn later_layer_small_perimeter_uses_configured_relative_speed() {
    let mut archive = KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(PROJECT_SETTINGS)).unwrap();
    let settings = settings.as_object_mut().unwrap();
    settings.insert("small_perimeter_threshold".to_owned(), json!("10000"));
    settings.insert("small_perimeter_speed".to_owned(), json!("50%"));
    settings.insert("outer_wall_speed".to_owned(), json!("200"));
    settings.insert("slow_down_for_layer_cooling".to_owned(), json!(["0", "0"]));
    archive.insert_text(PROJECT_SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(&archive.bytes(), metadata())
        .await
        .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let layers = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| **line == "; CHANGE_LAYER" || **line == ";LAYER_CHANGE")
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let second = &lines[layers[1]..layers[2]];
    let inner = second
        .iter()
        .position(|line| *line == "; FEATURE: Inner wall" || *line == ";TYPE:Inner wall")
        .unwrap();
    let feed = second[inner..]
        .iter()
        .find(|line| line.starts_with("G1 F"))
        .copied();

    assert_eq!(feed, Some("G1 F6000"));
}
