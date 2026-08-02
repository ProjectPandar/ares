use crate::{SliceError, slice_project};

use super::super::super::support::{KsrArchive, metadata};

#[tokio::test]
async fn task22o1_preflight_rejects_each_activated_deferred_classic_branch() {
    let cases = [
        (
            "\"wall_generator\": \"classic\"",
            "\"wall_generator\": \"arachne\"",
            "wall_generator",
        ),
        (
            "\"spiral_mode\": \"0\"",
            "\"spiral_mode\": \"1\"",
            "spiral_mode",
        ),
        (
            "\"fuzzy_skin\": \"disabled_fuzzy\"",
            "\"fuzzy_skin\": \"external\"",
            "fuzzy_skin",
        ),
        (
            "\"detect_thin_wall\": \"0\"",
            "\"detect_thin_wall\": \"1\"",
            "detect_thin_wall",
        ),
        (
            "\"alternate_extra_wall\": \"0\"",
            "\"alternate_extra_wall\": \"1\"",
            "alternate_extra_wall",
        ),
        (
            "\"only_one_wall_first_layer\": \"0\"",
            "\"only_one_wall_first_layer\": \"1\"",
            "only_one_wall_first_layer",
        ),
        (
            "\"overhang_reverse\": \"0\"",
            "\"overhang_reverse\": \"1\"",
            "overhang_reverse",
        ),
        (
            "\"wall_sequence\": \"inner wall/outer wall\"",
            "\"wall_sequence\": \"outer wall/inner wall\"",
            "wall_sequence",
        ),
        (
            "\"brim_type\": \"auto_brim\"",
            "\"brim_type\": \"outer_only\"",
            "brim_type",
        ),
        (
            "\"extra_perimeters_on_overhangs\": \"0\"",
            "\"extra_perimeters_on_overhangs\": \"1\"",
            "extra_perimeters_on_overhangs",
        ),
        (
            "\"counterbore_hole_bridging\": \"none\"",
            "\"counterbore_hole_bridging\": \"partiallybridge\"",
            "counterbore_hole_bridging",
        ),
    ];

    for (from, to, key) in cases {
        let mut archive = KsrArchive::new();
        archive.replace_unique("Metadata/project_settings.config", from, to);
        assert_eq!(
            slice_project(archive.bytes(), metadata())
                .await
                .unwrap_err(),
            SliceError::UnsupportedProjectFeature(key.to_owned()),
            "{key}"
        );
    }
}

#[tokio::test]
async fn task22o1_preflight_keeps_earlier_raft_gate_precedence() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\t\"raft_layers\": \"0\",",
        "\t\"raft_layers\": \"1\",",
    );
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\t\"wall_generator\": \"classic\",",
        "\t\"wall_generator\": \"arachne\",",
    );
    assert_eq!(
        slice_project(archive.bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::UnsupportedProjectFeature("raft_layers".to_owned())
    );
}
