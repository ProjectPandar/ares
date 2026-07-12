use super::*;

#[tokio::test]
async fn pellet_mode_emits_effective_filament_diameter_in_header() {
    let output = pellet_output(json!({
        "pellet_modded_printer": true,
        "pellet_flow_coefficient": std::f64::consts::FRAC_1_PI,
        "filament_diameter": [9.99]
    }))
    .await;

    assert!(output.lines().any(|line| line == "; filament_diameter = 2"));
    assert!(
        !output
            .lines()
            .any(|line| line == "; filament_diameter = 9.99")
    );
}

#[tokio::test]
async fn pellet_mode_changes_generated_extrusion_values() {
    let filament = pellet_output(json!({
        "filament_diameter": [1.0]
    }))
    .await;
    let pellet = pellet_output(json!({
        "pellet_modded_printer": true,
        "pellet_flow_coefficient": std::f64::consts::FRAC_1_PI,
        "filament_diameter": [1.0]
    }))
    .await;

    let filament_e = first_extrusion_e(&filament);
    let pellet_e = first_extrusion_e(&pellet);

    assert_close(filament_e / pellet_e, 4.0);
    assert_ne!(extrusion_lines(&filament), extrusion_lines(&pellet));
}

async fn pellet_output(extra: serde_json::Value) -> String {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "sparse_infill_density": 0,
            "filament_max_volumetric_speed": 0.0,
            "slow_down_for_layer_cooling": false
        }),
        extra,
    );
    String::from_utf8(slice(square_pyramid_ascii_stl(), options).await.unwrap()).unwrap()
}

fn extrusion_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with("G1 X") && line.contains(" E"))
        .collect()
}

fn first_extrusion_e(output: &str) -> f64 {
    extrusion_lines(output)[0]
        .split_whitespace()
        .find_map(|word| word.strip_prefix('E'))
        .unwrap()
        .parse()
        .unwrap()
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 0.0001,
        "{actual} != {expected}"
    );
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
