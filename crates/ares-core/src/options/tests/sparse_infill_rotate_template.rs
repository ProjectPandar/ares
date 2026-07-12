use super::super::*;
use serde_json::json;

#[test]
fn sparse_infill_rotate_template_defaults_empty() {
    let infill = SliceOptions::default().infill_options().unwrap();

    assert!(infill.sparse_infill_rotate_template_degrees().is_empty());
}

#[test]
fn parses_plain_sparse_infill_rotate_template() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_rotate_template": "0, 90"
    }))
    .unwrap();

    assert_eq!(
        options
            .infill_options()
            .unwrap()
            .sparse_infill_rotate_template_degrees(),
        &[0.0, 90.0]
    );
}

#[test]
fn rejects_non_plain_sparse_infill_rotate_template_values() {
    for value in [
        json!("+5"),
        json!("-5"),
        json!("0 90"),
        json!("90,"),
        json!("bad"),
        json!(90),
        json!(null),
        json!([0, 90]),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "sparse_infill_rotate_template": value })).unwrap();

        assert!(matches!(
            options.infill_options(),
            Err(SliceError::InvalidInput(message))
                if message.contains("sparse_infill_rotate_template")
        ));
    }
}
