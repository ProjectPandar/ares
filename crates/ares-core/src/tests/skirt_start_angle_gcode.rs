use super::*;

#[tokio::test]
async fn skirt_start_angle_reorders_first_skirt_artifacts_and_commands() {
    let options: SliceOptions = serde_json::from_value(json!({
        "skirt_start_angle": 45,
        "skirt_loops": 1,
        "skirt_height": 1,
        "sparse_infill_density": 0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
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
            .any(|line| line == ";SKIRT:2.5,2.5 -> -2.5,2.5 -> -2.5,-2.5 -> 2.5,-2.5")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";PRINT_PATH:skirt:2.5,2.5 -> -2.5,2.5 -> -2.5,-2.5 -> 2.5,-2.5")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";MOVE:travel:skirt:2.5,2.5")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";MOVE:print:skirt:-2.5,2.5")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";EXTRUSION:travel:skirt:2.5,2.5:")
    );
    assert!(
        output
            .lines()
            .any(|line| line.starts_with(";EXTRUSION:print:skirt:-2.5,2.5:"))
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";SPEED:travel:skirt:2.5,2.5:7200")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";SPEED:print:skirt:-2.5,2.5:3000")
    );
    assert!(output.lines().any(|line| line == "G1 X2.5 Y2.5 F7200"));
    assert!(
        output
            .lines()
            .any(|line| line.starts_with("G1 X-2.5 Y2.5 E"))
    );
}

#[tokio::test]
async fn default_skirt_start_angle_preserves_existing_start_corner() {
    let output = slice(square_pyramid_ascii_stl(), SliceOptions::default())
        .await
        .unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == ";SKIRT:-2.5,-2.5 -> 2.5,-2.5 -> 2.5,2.5 -> -2.5,2.5")
    );
    assert!(output.lines().any(|line| line == "G1 X-2.5 Y-2.5 F7200"));
}

#[tokio::test]
async fn per_object_skirt_type_preserves_skirt_start_angle_on_current_object() {
    let options: SliceOptions = serde_json::from_value(json!({
        "skirt_type": "perobject",
        "skirt_start_angle": 45,
        "skirt_loops": 1,
        "skirt_height": 1,
        "sparse_infill_density": 0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }))
    .unwrap();

    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == ";SKIRT:2.5,2.5 -> -2.5,2.5 -> -2.5,-2.5 -> 2.5,-2.5")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";PRINT_PATH:skirt:2.5,2.5 -> -2.5,2.5 -> -2.5,-2.5 -> 2.5,-2.5")
    );
}
