use super::*;

#[tokio::test]
async fn single_loop_draft_shield_emits_outer_loop_after_first_layer() {
    let options: SliceOptions = serde_json::from_value(json!({
        "draft_shield": "enabled",
        "single_loop_draft_shield": true,
        "skirt_loops": 2,
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
            .any(|line| line == "; total_skirt_path_count = 3")
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; skirt_count = 2")
            .count(),
        1
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; skirt_count = 1")
            .count(),
        1
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";SKIRT:-2.95,-2.95 -> 2.95,-2.95 -> 2.95,2.95 -> -2.95,2.95")
    );
    assert!(output.lines().any(
        |line| line == ";PRINT_PATH:skirt:-2.95,-2.95 -> 2.95,-2.95 -> 2.95,2.95 -> -2.95,2.95"
    ));
    assert!(
        output
            .lines()
            .any(|line| line == ";MOVE:travel:skirt:-2.95,-2.95")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";MOVE:print:skirt:2.95,-2.95")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";EXTRUSION:travel:skirt:-2.95,-2.95:")
    );
    assert!(
        output
            .lines()
            .any(|line| line.starts_with(";EXTRUSION:print:skirt:2.95,-2.95:"))
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";SPEED:travel:skirt:-2.95,-2.95:7200")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";SPEED:print:skirt:2.95,-2.95:3000")
    );
    assert!(output.lines().any(|line| line == "G1 X-2.95 Y-2.95 F7200"));
    assert!(
        output
            .lines()
            .any(|line| line.starts_with("G1 X2.95 Y-2.95 E"))
    );
}

#[tokio::test]
async fn disabled_single_loop_draft_shield_preserves_later_multi_loop_output() {
    let options: SliceOptions = serde_json::from_value(json!({
        "draft_shield": "enabled",
        "single_loop_draft_shield": false,
        "skirt_loops": 2,
        "skirt_height": 1,
        "sparse_infill_density": 0
    }))
    .unwrap();

    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; total_skirt_path_count = 4")
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; skirt_count = 2")
            .count(),
        2
    );
}

#[tokio::test]
async fn per_object_skirt_type_preserves_single_loop_draft_shield_on_current_object() {
    let options: SliceOptions = serde_json::from_value(json!({
        "skirt_type": "perobject",
        "draft_shield": "enabled",
        "single_loop_draft_shield": true,
        "skirt_loops": 2,
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
            .any(|line| line == "; total_skirt_path_count = 3")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";SKIRT:-2.95,-2.95 -> 2.95,-2.95 -> 2.95,2.95 -> -2.95,2.95")
    );
}
