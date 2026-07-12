use super::*;

#[tokio::test]
async fn slice_emits_skirt_artifacts_and_commands() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.25,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "is_infill_first": true,
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
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; skirt_count = 1")
            .count(),
        1
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; skirt_count = 0")
            .count(),
        1
    );
    assert!(
        output
            .lines()
            .any(|line| { line == ";SKIRT:-2.5,-2.5 -> 2.5,-2.5 -> 2.5,2.5 -> -2.5,2.5" })
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";PRINT_PATH:skirt:-2.5,-2.5 -> 2.5,-2.5 -> 2.5,2.5 -> -2.5,2.5")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";MOVE:travel:skirt:-2.5,-2.5")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";MOVE:print:skirt:2.5,-2.5")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";EXTRUSION:travel:skirt:-2.5,-2.5:")
    );
    assert!(
        output
            .lines()
            .any(|line| line.starts_with(";EXTRUSION:print:skirt:2.5,-2.5:"))
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";SPEED:travel:skirt:-2.5,-2.5:7200")
    );
    assert!(
        output
            .lines()
            .any(|line| line == ";SPEED:print:skirt:2.5,-2.5:3000")
    );
    assert!(output.lines().any(|line| line == "G1 X-2.5 Y-2.5 F7200"));
    assert!(
        output
            .lines()
            .any(|line| { line.starts_with("G1 X2.5 Y-2.5 E") })
    );
    assert_eq!(path_following_command_count(&output), 27);
    assert!(output.lines().any(|line| line == "G1 F3000"));
    assert!(standalone_feedrate_command_count(&output) > 0);
}

#[tokio::test]
async fn skirt_speed_zero_uses_external_perimeter_speed() {
    let options: SliceOptions = serde_json::from_value(json!({
        "skirt_speed": 0,
        "outer_wall_speed": 40,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }))
    .unwrap();
    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == ";SPEED:print:skirt:2.5,-2.5:2400")
    );
    assert!(
        output
            .lines()
            .any(|line| { line.starts_with("G1 X2.5 Y-2.5 E") })
    );
}

#[tokio::test]
async fn configured_skirt_speed_sets_skirt_feedrate() {
    let options: SliceOptions = serde_json::from_value(json!({
        "skirt_speed": 35,
        "outer_wall_speed": 60,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }))
    .unwrap();
    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(
        output
            .lines()
            .any(|line| line == ";SPEED:print:skirt:2.5,-2.5:2100")
    );
    assert!(output.lines().any(|line| line == "G1 F2100"));
}

#[tokio::test]
async fn enabled_draft_shield_emits_skirts_beyond_first_layer() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "draft_shield": "enabled",
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
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; skirt_count = 1")
            .count(),
        2
    );
    assert!(
        output
            .lines()
            .skip_while(|line| *line != ";LAYER:1")
            .any(|line| line.starts_with(";SKIRT:"))
    );
}

#[tokio::test]
async fn enabled_draft_shield_with_zero_loops_reaches_gcode() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "draft_shield": "enabled",
        "skirt_loops": 0,
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
    assert_eq!(
        output
            .lines()
            .filter(|line| *line == "; skirt_count = 1")
            .count(),
        2
    );
    assert!(
        output
            .lines()
            .skip_while(|line| *line != ";LAYER:1")
            .any(|line| line.starts_with(";PRINT_PATH:skirt:"))
    );
}

#[tokio::test]
async fn non_draft_shield_skirt_distance_is_measured_from_outer_brim() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "skirt_loops": 1,
        "skirt_height": 1,
        "skirt_distance": 1.0,
        "skirt_line_width": 0.4,
        "brim_width": 0.8,
        "brim_object_gap": 0.2,
        "brim_type": "outer_only",
        "sparse_infill_density": 0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }))
    .unwrap();
    let output = slice(square_pyramid_ascii_stl(), options).await.unwrap();
    let output = String::from_utf8(output).unwrap();

    let skirt = ";PRINT_PATH:skirt:-2.5,-2.5 -> 2.5,-2.5 -> 2.5,2.5 -> -2.5,2.5";
    let brim = ";PRINT_PATH:brim:-1.5,-1.5 -> 1.5,-1.5 -> 1.5,1.5 -> -1.5,1.5";
    let skirt_index = output.find(skirt).expect("expanded skirt print path");
    let brim_index = output.find(brim).expect("outer brim print path");

    assert!(skirt_index < brim_index);
    assert!(
        !output
            .lines()
            .any(|line| line == ";PRINT_PATH:skirt:-1.5,-1.5 -> 1.5,-1.5 -> 1.5,1.5 -> -1.5,1.5")
    );
}

#[tokio::test]
async fn initial_layer_print_height_changes_skirt_extrusion_amount() {
    let low = skirt_height_options(0.2);
    let high = skirt_height_options(0.32);

    let low_output =
        String::from_utf8(slice(square_pyramid_ascii_stl(), low).await.unwrap()).unwrap();
    let high_output =
        String::from_utf8(slice(square_pyramid_ascii_stl(), high).await.unwrap()).unwrap();

    let low_e = first_skirt_print_e_per_mm(&low_output);
    let high_e = first_skirt_print_e_per_mm(&high_output);

    assert!(high_e > low_e);
    assert!(
        (high_e / low_e - 1.6).abs() < 0.01,
        "low_e={low_e}, high_e={high_e}, ratio={}",
        high_e / low_e
    );
}

fn skirt_height_options(initial_layer_print_height: f64) -> SliceOptions {
    serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_print_height": initial_layer_print_height,
        "line_width": 10.0,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.25,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "is_infill_first": true,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }))
    .unwrap()
}

fn first_skirt_print_e_per_mm(output: &str) -> f64 {
    let mut start = None;
    for line in output.lines() {
        if let Some(line) = line.strip_prefix(";EXTRUSION:travel:skirt:") {
            start = skirt_point(line);
        } else if let Some(line) = line.strip_prefix(";EXTRUSION:print:skirt:") {
            let (end, e) = skirt_point_and_e(line).expect("first skirt print move with extrusion");
            let start = start.expect("first skirt travel move");
            let length = ((end.0 - start.0).powi(2) + (end.1 - start.1).powi(2)).sqrt();
            return e / length;
        }
    }
    panic!("first skirt print move with extrusion");
}

fn skirt_point(line: &str) -> Option<(f64, f64)> {
    let (point, _) = line.split_once(':').unwrap_or((line, ""));
    let (x, y) = point.split_once(',')?;
    Some((x.parse().ok()?, y.parse().ok()?))
}

fn skirt_point_and_e(line: &str) -> Option<((f64, f64), f64)> {
    let (point, e) = line.rsplit_once(':')?;
    Some((skirt_point(point)?, e.parse().ok()?))
}

fn path_following_command_count(output: &str) -> usize {
    output
        .lines()
        .filter(|line| line.starts_with(";MOVE:"))
        .count()
}

fn standalone_feedrate_command_count(output: &str) -> usize {
    output
        .lines()
        .filter(|line| line.starts_with("G1 F"))
        .count()
}
