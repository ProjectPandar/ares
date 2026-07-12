use super::*;

#[tokio::test]
async fn min_skirt_length_adds_real_first_layer_skirt_loops() {
    let options: SliceOptions = serde_json::from_value(json!({
        "min_skirt_length": 1.0,
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
            .any(|line| line == "; total_skirt_path_count = 2")
    );
    assert!(output.lines().any(|line| line == "; skirt_count = 2"));
    assert!(
        output
            .lines()
            .any(|line| line == ";SKIRT:-2.5,-2.5 -> 2.5,-2.5 -> 2.5,2.5 -> -2.5,2.5")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";SKIRT:-2.95,-2.95 -> 2.95,-2.95 -> 2.95,2.95 -> -2.95,2.95")
    );
    assert_eq!(path_following_command_count(&output), 20);
    assert!(output.lines().any(|line| line == "G1 X-2.95 Y-2.95 F7200"));
    assert!(
        output
            .lines()
            .any(|line| line.starts_with("G1 X2.95 Y-2.95 E"))
    );
}

#[tokio::test]
async fn zero_skirt_loops_still_disable_skirt_with_min_skirt_length() {
    let options: SliceOptions = serde_json::from_value(json!({
        "min_skirt_length": 1.0,
        "skirt_loops": 0,
        "sparse_infill_density": 0
    }))
    .unwrap();

    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == "; total_skirt_path_count = 0")
    );
    assert!(!output.lines().any(|line| line.starts_with(";SKIRT:")));
}

#[tokio::test]
async fn impossible_min_skirt_length_is_rejected() {
    let options: SliceOptions = serde_json::from_value(json!({
        "min_skirt_length": 1.0e12,
        "skirt_loops": 1,
        "sparse_infill_density": 0
    }))
    .unwrap();

    let err = slice(square_pyramid_ascii_stl(), options)
        .await
        .unwrap_err();

    assert!(
        matches!(err, SliceError::InvalidInput(message) if message.contains("min_skirt_length would require more than 10000 skirt loops"))
    );
}

#[tokio::test]
async fn per_object_skirt_type_preserves_min_skirt_length_on_current_object() {
    let options: SliceOptions = serde_json::from_value(json!({
        "skirt_type": "perobject",
        "min_skirt_length": 1.0,
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
            .any(|line| line == "; total_skirt_path_count = 2")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";SKIRT:-2.95,-2.95 -> 2.95,-2.95 -> 2.95,2.95 -> -2.95,2.95")
    );
}

fn path_following_command_count(output: &str) -> usize {
    output
        .lines()
        .filter(|line| line.starts_with(";MOVE:"))
        .count()
}
