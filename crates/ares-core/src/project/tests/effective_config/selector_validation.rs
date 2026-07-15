use crate::{OrcaInt, Project, load_project};

use super::{ProjectParts, assert_invalid_key, valid_project, valid_settings, validate};

#[test]
fn wipe_tower_filament_zero_is_allowed_and_negative_is_rejected() {
    let project = valid_project();
    let settings = valid_settings(2, 2);
    validate(&settings, &project).unwrap();

    let mut settings = valid_settings(2, 2);
    settings.process.print.wipe_tower_filament = OrcaInt(-1);
    assert_invalid_key(validate(&settings, &project), "wipe_tower_filament");
}

#[test]
fn wipe_tower_filament_checks_distinct_unequal_physical_and_logical_bounds() {
    let project = valid_project();

    let mut valid = valid_settings(4, 2);
    valid.process.print.wipe_tower_filament = OrcaInt(2);
    validate(&valid, &project).unwrap();

    let mut physical_invalid = valid_settings(2, 4);
    physical_invalid.process.print.wipe_tower_filament = OrcaInt(2);
    assert_invalid_key(validate(&physical_invalid, &project), "wipe_tower_filament");

    let mut logical_invalid = valid_settings(4, 2);
    logical_invalid.process.print.wipe_tower_filament = OrcaInt(3);
    assert_invalid_key(validate(&logical_invalid, &project), "wipe_tower_filament");
}

#[test]
fn process_support_selectors_reject_negatives_but_accept_above_logical() {
    let project = valid_project();
    for key in ["support_filament", "support_interface_filament"] {
        let mut negative = valid_settings(1, 1);
        match key {
            "support_filament" => negative.process.object.support_filament = OrcaInt(-1),
            "support_interface_filament" => {
                negative.process.object.support_interface_filament = OrcaInt(-1);
            }
            _ => unreachable!(),
        }
        assert_invalid_key(validate(&negative, &project), key);

        let mut above_logical = valid_settings(1, 1);
        match key {
            "support_filament" => above_logical.process.object.support_filament = OrcaInt(2),
            "support_interface_filament" => {
                above_logical.process.object.support_interface_filament = OrcaInt(2);
            }
            _ => unreachable!(),
        }
        validate(&above_logical, &project).unwrap();
    }
}

#[test]
fn every_object_support_override_rejects_negatives_but_accepts_above_logical() {
    let settings = valid_settings(1, 1);
    for key in ["support_filament", "support_interface_filament"] {
        let negative = project_with_object_option(key, -1);
        assert_invalid_key(validate(&settings, &negative), key);

        let above_logical = project_with_object_option(key, 2);
        validate(&settings, &above_logical).unwrap();
    }
}

#[test]
fn raw_object_extruder_enforces_domain_at_archive_call_site() {
    assert_raw_extruder_domain(project_with_object_option);
}

#[test]
fn raw_volume_extruder_enforces_domain_at_archive_call_site() {
    assert_raw_extruder_domain(project_with_volume_option);
}

#[test]
fn raw_layer_range_extruder_enforces_domain_at_archive_call_site() {
    assert_raw_extruder_domain(project_with_layer_extruder);
}

fn assert_raw_extruder_domain(project_with_value: fn(&str, i32) -> Project) {
    let settings = valid_settings(1, 2);
    for value in [-1, 3] {
        let project = project_with_value("extruder", value);
        assert_invalid_key(validate(&settings, &project), "extruder");
    }

    for value in [0, 2] {
        let project = project_with_value("extruder", value);
        validate(&settings, &project).unwrap();
    }
}

fn project_with_object_option(key: &str, value: i32) -> Project {
    let mut parts = ProjectParts::valid();
    let replacement = format!(r#"<object id="2"><metadata key="{key}" value="{value}"/>"#);
    parts.replace(
        "Metadata/model_settings.config",
        r#"<object id="2">"#,
        &replacement,
    );
    load_project(parts.bytes()).unwrap()
}

fn project_with_volume_option(key: &str, value: i32) -> Project {
    let mut parts = ProjectParts::valid();
    let replacement =
        format!(r#"<part id="1" subtype="normal_part"><metadata key="{key}" value="{value}"/>"#);
    parts.replace(
        "Metadata/model_settings.config",
        r#"<part id="1" subtype="normal_part">"#,
        &replacement,
    );
    load_project(parts.bytes()).unwrap()
}

fn project_with_layer_extruder(_: &str, value: i32) -> Project {
    let mut parts = ProjectParts::valid();
    parts.insert_text(
        "Metadata/layer_config_ranges.xml",
        &format!(
            r#"<objects><object id="1"><range min_z="0" max_z="1"><option opt_key="extruder">{value}</option></range></object></objects>"#
        ),
    );
    load_project(parts.bytes()).unwrap()
}
