use crate::load_project;

use super::{LayerProject, assert_bounded, error_message};

const EMPTY_RANGES: &str = "<objects/>";

#[test]
fn absent_optional_entry_leaves_every_object_without_ranges() {
    let project = load_project(LayerProject::with_build_order(&[42, 7]).bytes()).unwrap();
    let public_ranges: &[crate::LayerConfigRange] = project.objects()[0].layer_config_ranges();

    assert!(public_ranges.is_empty());
    assert!(
        project
            .objects()
            .iter()
            .all(|object| object.layer_config_ranges().is_empty())
    );
}

#[test]
fn one_ascii_case_variant_is_read_through_its_validated_exact_path() {
    let mut project = LayerProject::one_object();
    project.insert_ranges(
        "mEtAdAtA/LaYeR_CoNfIg_RaNgEs.XmL",
        r#"<objects><object id="1"><range min_z="0" max_z="1"><option opt_key="wall_loops">6</option></range></object></objects>"#,
    );

    let loaded = load_project(project.bytes()).unwrap();
    assert_eq!(
        loaded.objects()[0].layer_config_ranges()[0]
            .region_overrides()
            .wall_loops,
        Some(crate::OrcaInt(6))
    );
}

#[test]
fn multiple_ascii_case_variants_are_a_bounded_ambiguity_error() {
    let mut project = LayerProject::one_object();
    project.insert_ranges("Metadata/layer_config_ranges.xml", EMPTY_RANGES);
    project.insert_ranges("metadata/LAYER_CONFIG_RANGES.XML", EMPTY_RANGES);

    let message = error_message(project);

    assert!(message.contains("ambiguous"), "{message}");
    assert!(
        message.contains("Metadata/layer_config_ranges.xml"),
        "{message}"
    );
    assert_bounded(&message);
}

#[test]
fn optional_lookup_does_not_bypass_normalized_or_backslash_path_rejection() {
    for path in [
        "Metadata/./layer_config_ranges.xml",
        "Metadata\\layer_config_ranges.xml",
    ] {
        let mut project = LayerProject::one_object();
        project.insert_ranges(path, EMPTY_RANGES);
        let message = error_message(project);

        assert!(
            message.contains("invalid project archive"),
            "{path}: {message}"
        );
        assert_bounded(&message);
    }
}
