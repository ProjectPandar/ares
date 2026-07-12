use super::*;

#[tokio::test]
async fn combined_skirt_type_emits_combined_skirt_gcode() {
    let options: SliceOptions = serde_json::from_value(json!({
        "skirt_type": "combined",
        "skirt_loops": 1,
        "skirt_height": 1,
        "sparse_infill_density": 0
    }))
    .unwrap();
    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; total_skirt_path_count = 1")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";SKIRT:-2.5,-2.5 -> 2.5,-2.5 -> 2.5,2.5 -> -2.5,2.5")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";PRINT_PATH:skirt:-2.5,-2.5 -> 2.5,-2.5 -> 2.5,2.5 -> -2.5,2.5")
    );
}

#[tokio::test]
async fn per_object_skirt_type_emits_current_object_skirt_gcode() {
    let options: SliceOptions = serde_json::from_value(json!({
        "skirt_type": "perobject",
        "skirt_loops": 1,
        "skirt_height": 1,
        "sparse_infill_density": 0
    }))
    .unwrap();
    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; total_skirt_path_count = 1")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";SKIRT:-2.5,-2.5 -> 2.5,-2.5 -> 2.5,2.5 -> -2.5,2.5")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";PRINT_PATH:skirt:-2.5,-2.5 -> 2.5,-2.5 -> 2.5,2.5 -> -2.5,2.5")
    );
}
