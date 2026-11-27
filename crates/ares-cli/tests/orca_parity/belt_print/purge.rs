//! Belt purge option handling with one configured filament (`BeltPurge.cpp`).

use serde_json::{Value, json};

#[test]
fn enabled_purge_without_marked_object_matches_native() {
    let project = super::project(
        true,
        false,
        &[("enable_belt_purge_tower", json!("1"))],
        false,
    );
    super::compare_project_with_native("enabled-without-prism", project, |native| {
        assert_prints_without_purge(native);
        assert!(!native.contains(";TYPE:Brim\n"));
    });
}

#[test]
fn marked_objects_suppress_belt_brim_with_either_purge_setting() {
    for (name, enabled, marker) in [
        ("disabled-unmarked", "0", None),
        ("disabled-marked", "0", Some("1")),
        ("enabled-unmarked", "1", None),
        ("enabled-marked", "1", Some("1")),
    ] {
        compare(
            name,
            "45",
            marker,
            &[("enable_belt_purge_tower", json!(enabled))],
            marker.is_none(),
        );
    }
}

#[test]
fn explicit_object_marker_overrides_inherited_process_marker() {
    let settings = [("belt_purge_tower_object", json!("1"))];
    compare("inherited-marker", "45", None, &settings, false);
    compare("cleared-marker", "45", Some("0"), &settings, true);
}

#[test]
fn supplied_prism_mesh_matches_native_at_different_width_settings() {
    for (name, width) in [("width-narrow", "10"), ("width-wide", "60")] {
        compare(
            name,
            "45",
            Some("1"),
            &[
                ("enable_belt_purge_tower", json!("1")),
                ("belt_purge_tower_width", json!(width)),
            ],
            false,
        );
    }
}

#[test]
fn purge_marker_preserves_the_belt_brim_tilt_window() {
    for (angle_name, angle, in_brim_window) in [
        ("0p5", "0.5", false),
        ("1", "1", true),
        ("85", "85", true),
        ("86", "86", false),
        ("neg45", "-45", true),
    ] {
        for (label, marker) in [("unmarked", None), ("marked", Some("1"))] {
            compare(
                &format!("{angle_name}-{label}"),
                angle,
                marker,
                &[],
                in_brim_window && marker.is_none(),
            );
        }
    }
}

#[test]
fn belt_does_not_emit_a_classic_prime_tower_when_enabled() {
    // Smooth timelapse would activate a classic tower even with only one
    // filament on a Cartesian printer (`Print::has_wipe_tower`).
    compare(
        "no-classic-tower",
        "45",
        None,
        &[
            ("enable_belt_purge_tower", json!("1")),
            ("enable_prime_tower", json!("1")),
            ("timelapse_type", json!("1")),
        ],
        true,
    );
}

#[test]
fn enabling_belt_purge_still_rejects_support_without_gcode() {
    super::rejects(
        "support",
        &[
            ("enable_belt_purge_tower", json!("1")),
            ("enable_support", json!("1")),
        ],
    );
}

#[test]
fn enabling_belt_purge_still_rejects_multiple_filaments_without_gcode() {
    super::rejects(
        "materials",
        &[
            ("enable_belt_purge_tower", json!("1")),
            ("filament_diameter", json!(["1.75", "1.75"])),
            ("filament_map", json!(["1", "1"])),
        ],
    );
}

#[test]
fn enabling_belt_purge_still_rejects_multiple_instances_without_gcode() {
    let input = super::project_with(
        true,
        false,
        &[("enable_belt_purge_tower", json!("1"))],
        false,
        |name, bytes| {
            let second_instance = match name {
                "3D/3dmodel.model" => duplicate_build_item,
                "Metadata/model_settings.config" => duplicate_plate_instance,
                _ => return bytes,
            };
            second_instance(std::str::from_utf8(&bytes).unwrap()).into_bytes()
        },
    );
    super::rejects_project("instances", input);
}

fn compare(
    label: &str,
    angle: &str,
    object_marker: Option<&str>,
    edits: &[(&str, Value)],
    has_brim: bool,
) {
    let mut settings = vec![
        ("belt_slice_rotation_angle", json!(angle)),
        ("brim_type", json!("outer_only")),
        ("brim_width", json!("5")),
    ];
    settings.extend_from_slice(edits);
    let input = super::project_with(true, false, &settings, false, |name, bytes| {
        if let Some(marker) = object_marker.filter(|_| name == "Metadata/model_settings.config") {
            let config = std::str::from_utf8(&bytes).unwrap();
            let object_setting = "<metadata key=\"extruder\" value=\"1\"/>";
            assert_eq!(config.matches(object_setting).count(), 1);
            return config
                .replace(
                    object_setting,
                    &format!(
                        "{object_setting}\n    <metadata key=\"belt_purge_tower_object\" value=\"{marker}\"/>"
                    ),
                )
                .into_bytes();
        }
        bytes
    });
    super::compare_project_with_native(label, input, |native| {
        assert_prints_without_purge(native);
        assert_eq!(native.contains(";TYPE:Brim\n"), has_brim, "{label}");
    });
}

fn assert_prints_without_purge(native: &str) {
    assert!(native.contains(";TYPE:Outer wall\n"));
    assert!(!native.contains(";TYPE:Prime tower\n"));
    assert!(!native.contains("; WIPE_TOWER_START\n"));
    assert!(!native.contains("; PURGING FINISHED\n"));
}

fn duplicate_build_item(model: &str) -> String {
    let item = model.lines().find(|line| line.contains("<item ")).unwrap();
    assert_eq!(model.matches("<item ").count(), 1);
    assert!(item.contains("110 110 0") && item.contains("00000002-b1ec"));
    let copy = item
        .replace("110 110 0", "140 110 0")
        .replace("00000002-b1ec", "00000102-b1ec");
    model.replace(item, &format!("{item}\n{copy}"))
}

fn duplicate_plate_instance(config: &str) -> String {
    let (prefix, rest) = config.split_once("    <model_instance>").unwrap();
    let (instance, suffix) = rest.split_once("</model_instance>").unwrap();
    let copy = instance
        .replace(
            "key=\"instance_id\" value=\"0\"",
            "key=\"instance_id\" value=\"1\"",
        )
        .replace(
            "key=\"identify_id\" value=\"14\"",
            "key=\"identify_id\" value=\"15\"",
        );
    format!(
        "{prefix}    <model_instance>{instance}</model_instance>\n    <model_instance>{copy}</model_instance>{suffix}"
    )
}
