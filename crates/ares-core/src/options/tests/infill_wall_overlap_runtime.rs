use super::super::*;
use serde_json::json;

#[test]
fn infill_wall_overlap_defaults_to_raw_percentages() {
    let infill = SliceOptions::default().infill_options().unwrap();

    assert_eq!(infill.infill_wall_overlap_percent(), 15.0);
    assert_eq!(infill.top_bottom_infill_wall_overlap_percent(), 25.0);
}

#[test]
fn parses_infill_wall_overlap_percent_forms() {
    for value in [json!(20), json!("20"), json!("20%")] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "infill_wall_overlap": value })).unwrap();

        assert_eq!(
            options.infill_options().unwrap().infill_wall_overlap_percent(),
            20.0
        );
    }
}

#[test]
fn parses_top_bottom_infill_wall_overlap_percent_forms() {
    for value in [json!(20), json!("20"), json!("20%")] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "top_bottom_infill_wall_overlap": value })).unwrap();

        assert_eq!(
            options
                .infill_options()
                .unwrap()
                .top_bottom_infill_wall_overlap_percent(),
            20.0
        );
    }
}

#[test]
fn rejects_invalid_infill_wall_overlap_percentages() {
    for value in [
        json!(-0.1),
        json!("NaN"),
        json!("inf"),
        json!("bad"),
        json!(false),
        json!(null),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "infill_wall_overlap": value })).unwrap();

        let err = options.infill_options().unwrap_err();

        assert!(err.to_string().contains("infill_wall_overlap"), "{err}");
    }
}

#[test]
fn rejects_invalid_top_bottom_infill_wall_overlap_percentages() {
    for value in [
        json!(-0.1),
        json!("NaN"),
        json!("inf"),
        json!("bad"),
        json!(false),
        json!(null),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "top_bottom_infill_wall_overlap": value })).unwrap();

        let err = options.infill_options().unwrap_err();

        assert!(
            err.to_string()
                .contains("top_bottom_infill_wall_overlap"),
            "{err}"
        );
    }
}
