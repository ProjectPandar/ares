use super::application::{verify, width};
use serde_json::{Value, json};
use std::io::{Cursor, Write};

fn project(fields: Value) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file(
        "Metadata/project_settings.config",
        zip::write::SimpleFileOptions::default(),
    )
    .unwrap();
    zip.write_all(&serde_json::to_vec(&fields).unwrap())
        .unwrap();
    zip.finish().unwrap().into_inner()
}

fn fields(value: Value) -> Value {
    json!({"line_width": value, "nozzle_diameter": ["0.4"], "layer_height": "0.2",
        "thick_bridges": "0", "thick_internal_bridges": "0", "internal_solid_infill_line_width": "0"})
}

#[test]
fn resolution_floor_proof_preserves_subthreshold_requests() {
    for requested in ["0", "0.0005", "0.000999"] {
        let proof = super::application::verify_value(
            &project(json!({"resolution": "0.001"})),
            "resolution",
            &json!(requested),
        )
        .unwrap();
        assert_eq!(proof["requested"], requested);
        assert_eq!(proof["effective"], "0.001");
        assert_eq!(proof["normalization"]["kind"], "resolution_floor");
    }
}

#[test]
fn resolution_floor_proof_rejects_wrong_exports_and_preserves_other_values() {
    for effective in [
        json!("0"),
        json!("0.002"),
        json!(["0.001"]),
        json!(0.001),
        Value::Null,
    ] {
        assert!(
            super::application::verify_value(
                &project(json!({"resolution": effective})),
                "resolution",
                &json!("0")
            )
            .is_err()
        );
    }
    assert!(
        super::application::verify_value(&project(json!({})), "resolution", &json!("0")).is_err()
    );
    for requested in ["0.001", "0.001001", "0.01", "2"] {
        let proof = super::application::verify_value(
            &project(json!({"resolution": requested})),
            "resolution",
            &json!(requested),
        )
        .unwrap();
        assert!(proof.get("normalization").is_none());
    }
    for (key, requested) in [
        ("resolution", "0.002"),
        ("resolution", "-1"),
        ("resolution", "NaN"),
        ("layer_height", "0"),
    ] {
        assert!(
            super::application::verify_value(
                &project(json!({key: "0.001"})),
                key,
                &json!(requested)
            )
            .is_err()
        );
    }
}

#[test]
fn obsolete_silent_mode_proof_preserves_requests_and_requires_false_default() {
    for requested in [json!("0"), json!("1")] {
        let proof = super::application::verify_value(
            &project(json!({"silent_mode": "0"})),
            "silent_mode",
            &requested,
        )
        .unwrap();
        assert_eq!(proof["requested"], requested);
        assert_eq!(proof["effective"], "0");
        assert_eq!(proof["normalization"]["kind"], "obsolete_input");
        assert_eq!(proof["normalization"]["expected_effective"], "0");
    }
}

#[test]
fn obsolete_silent_mode_proof_rejects_wrong_effective_values() {
    for effective in [json!("1"), json!(false), json!(["0"]), Value::Null] {
        assert!(
            super::application::verify_value(
                &project(json!({"silent_mode": effective})),
                "silent_mode",
                &json!("1"),
            )
            .is_err()
        );
    }
    assert!(
        super::application::verify_value(&project(json!({})), "silent_mode", &json!("1")).is_err()
    );
    assert!(
        super::application::verify_value(
            &project(json!({"support_air_filtration": "0"})),
            "support_air_filtration",
            &json!("1"),
        )
        .is_err()
    );
}

#[test]
fn exported_bytes_require_exact_typed_width_not_physical_equivalence() {
    let dependencies = json!({"nozzle_diameter": ["0.4"], "layer_height": "0.2"});
    for value in [json!("0.6"), json!("150%"), json!("0"), json!("0%")] {
        let result = verify(
            &project(fields(value.clone())),
            "line_width",
            &value,
            dependencies.as_object().unwrap(),
        )
        .unwrap();
        assert_eq!(result["requested"], value);
        assert_eq!(result["effective"], value);
    }
    for (requested, effective) in [
        ("0.6", "150%"),
        ("150%", "0.6"),
        ("150", "150%"),
        ("0", "0%"),
        ("0%", "0"),
    ] {
        assert!(
            verify(
                &project(fields(json!(effective))),
                "line_width",
                &json!(requested),
                dependencies.as_object().unwrap()
            )
            .is_err()
        );
    }
    assert_ne!(width(&json!("0")).unwrap(), width(&json!("0%")).unwrap());
}

#[test]
fn missing_wrong_shape_wrong_owner_default_and_dependency_changes_fail_application() {
    let expected = json!("0.6");
    let dependencies = json!({"nozzle_diameter": ["0.4"], "layer_height": "0.2"});
    for value in [
        json!("0"),
        json!("0.45"),
        json!(["0.6"]),
        json!(0.6),
        Value::Null,
    ] {
        assert!(
            verify(
                &project(fields(value)),
                "line_width",
                &expected,
                dependencies.as_object().unwrap()
            )
            .is_err()
        );
    }
    let mut missing = fields(expected.clone());
    missing.as_object_mut().unwrap().remove("line_width");
    missing["machine_settings"] = json!({"line_width": "0.6"});
    assert!(
        verify(
            &project(missing),
            "line_width",
            &expected,
            dependencies.as_object().unwrap()
        )
        .is_err()
    );
    for (key, value) in [
        ("nozzle_diameter", json!(["0.6"])),
        ("layer_height", json!("0.3")),
    ] {
        let mut changed = fields(expected.clone());
        changed[key] = value;
        assert!(
            verify(
                &project(changed),
                "line_width",
                &expected,
                dependencies.as_object().unwrap()
            )
            .is_err()
        );
    }
    let mut missing_dependency = fields(expected.clone());
    missing_dependency
        .as_object_mut()
        .unwrap()
        .remove("thick_bridges");
    assert!(
        verify(
            &project(missing_dependency),
            "line_width",
            &expected,
            dependencies.as_object().unwrap()
        )
        .is_err()
    );
}
