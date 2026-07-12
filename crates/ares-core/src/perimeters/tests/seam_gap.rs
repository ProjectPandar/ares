use super::*;
use crate::{
    LayerBrims, LayerGapFills, LayerInfills, LayerSkirts, PrintPathInput, PrintPathRole,
    ShellLayerOptions, generate_print_paths,
};
use serde_json::json;

#[test]
fn seam_gap_defaults_to_ten_percent_of_external_width() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": 0.4,
    }))
    .unwrap();

    assert_approx_eq(options.perimeter_options().unwrap().seam_gap_mm(), 0.04);
}

#[test]
fn parses_numeric_and_percent_seam_gap() {
    let numeric: SliceOptions = serde_json::from_value(json!({
        "line_width": 0.4,
        "seam_gap": 0.25,
    }))
    .unwrap();
    let percent: SliceOptions = serde_json::from_value(json!({
        "line_width": 0.4,
        "seam_gap": "50%",
    }))
    .unwrap();

    assert_approx_eq(numeric.perimeter_options().unwrap().seam_gap_mm(), 0.25);
    assert_approx_eq(percent.perimeter_options().unwrap().seam_gap_mm(), 0.2);
}

#[test]
fn parses_zero_seam_gap() {
    let options: SliceOptions = serde_json::from_value(json!({
        "seam_gap": 0,
    }))
    .unwrap();

    assert_approx_eq(options.perimeter_options().unwrap().seam_gap_mm(), 0.0);
}

#[test]
fn rejects_invalid_seam_gap_values() {
    for value in [
        json!(-0.1),
        json!("bad"),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
        json!("NaN%"),
        json!(true),
        json!(null),
        json!([]),
        json!({}),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ "seam_gap": value })).unwrap();

        assert!(matches!(
            options.perimeter_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

#[test]
fn generated_perimeter_paths_carry_seam_gap() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];
    let options = PerimeterOptions::new(
        2,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::OuterInner,
    )
    .with_seam_gap_mm(0.25);

    let perimeters = generate_perimeters(&layers, options).unwrap();
    assert_eq!(perimeters[0].paths().len(), 2);
    for path in perimeters[0].paths() {
        assert_approx_eq(path.seam_gap_mm(), 0.25);
    }

    let print_paths = generate_print_paths(
        PrintPathInput::new(
            &[LayerSkirts::new(0, 0.2, Vec::new())],
            &[LayerBrims::new(0, 0.2, Vec::new())],
            &perimeters,
            &[LayerGapFills::new(0, 0.2, Vec::new())],
            &[LayerInfills::new(0, 0.2, Vec::new())],
        ),
        ShellLayerOptions::new(1, 1),
        false,
        false,
    )
    .unwrap();

    assert_eq!(
        print_paths[0].paths()[0].role(),
        PrintPathRole::ExternalPerimeter
    );
    assert_eq!(
        print_paths[0].paths()[1].role(),
        PrintPathRole::InternalPerimeter
    );
    for path in print_paths[0].paths() {
        assert_approx_eq(path.seam_gap_mm(), 0.25);
    }
}

fn assert_approx_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-9,
        "expected {actual} to approximately equal {expected}",
    );
}
