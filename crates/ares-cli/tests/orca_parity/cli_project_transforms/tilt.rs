use super::{PROJECT, verify, verify_project};
use crate::runner::profile_entry::plater_archive::rewrite_archive;

#[test]
fn rotate_x_without_grounding() {
    verify("rotate-x", None, 0, &["--rotate-x", "30"]);
}

#[test]
fn rotate_y_without_grounding() {
    verify("rotate-y", None, 0, &["--rotate-y", "-30"]);
}

#[test]
fn repeated_rotation_uses_first_position_and_final_angle() {
    verify(
        "rotate-first-position",
        None,
        0,
        &["--rotate-x", "15", "--rotate-y", "-30", "--rotate-x", "30"],
    );
}

#[test]
fn explicit_grounding_runs_after_selected_plate_arrange() {
    verify(
        "arrange-then-ground",
        Some("1"),
        0,
        &["--ensure-on-bed", "--rotate-x", "30", "--arrange", "1"],
    );
}

#[test]
fn explicit_false_leaves_the_rotated_project_sunk() {
    verify(
        "grounding-false",
        None,
        0,
        &["--ensure-on-bed", "--rotate-x", "30", "--ensure-on-bed=0"],
    );
}

#[test]
fn standalone_grounding_changes_instance_z() {
    let project = rewrite_archive(PROJECT, |name, text| {
        (name == "3D/3dmodel.model").then(|| {
            assert_eq!(text.matches("110 110 5\"").count(), 1);
            text.replace("110 110 5\"", "110 110 4\"")
        })
    });
    verify_project("ground-sunk", &project, None, 0, &["--ensure-on-bed=yes"]);
}

#[test]
fn object_auto_drop_off_overrides_legacy_on() {
    check_object_auto_drop("0", "1");
}

#[test]
fn object_auto_drop_on_overrides_legacy_off() {
    check_object_auto_drop("1", "0");
}

fn check_object_auto_drop(object: &str, legacy: &str) {
    let project = rewrite_archive(PROJECT, |name, text| match name {
        "3D/3dmodel.model" => {
            assert_eq!(text.matches(r#"auto_drop="1""#).count(), 1);
            Some(text.replace(r#"auto_drop="1""#, &format!(r#"auto_drop="{legacy}""#)))
        }
        "Metadata/model_settings.config" => {
            assert_eq!(text.matches(r#"<object id="2">"#).count(), 1);
            Some(text.replace(
                r#"<object id="2">"#,
                &format!(r#"<object id="2" auto_drop="{object}">"#),
            ))
        }
        _ => None,
    });
    verify_project(
        &format!("object-auto-drop-{object}"),
        &project,
        None,
        0,
        &["--rotate-y", "30", "--ensure-on-bed"],
    );
}
