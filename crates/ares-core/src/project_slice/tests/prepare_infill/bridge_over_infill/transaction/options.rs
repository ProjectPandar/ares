use super::{assert_error_mutation, assert_unsupported_mutation};
use crate::{
    SliceError,
    project_slice::{prepare_infill::bridge_over_infill::transaction, tests::support::KsrArchive},
};

#[test]
fn task22o71_deferred_pattern_and_second_pass_fail_without_fallback() {
    assert_unsupported_mutation(
        "\"sparse_infill_pattern\": \"crosshatch\"",
        "\"sparse_infill_pattern\": \"honeycomb\"",
        "sparse_infill_pattern",
    );
    assert_unsupported_mutation(
        "\"enable_extra_bridge_layer\": \"disabled\"",
        "\"enable_extra_bridge_layer\": \"internal_bridge_only\"",
        "enable_extra_bridge_layer",
    );
}

#[test]
fn task22o71_adaptive_octree_pattern_fails_even_without_candidates() {
    for pattern in ["adaptivecubic", "supportcubic"] {
        let mut archive = KsrArchive::new();
        archive.replace_unique(
            "Metadata/project_settings.config",
            "\"sparse_infill_pattern\": \"crosshatch\"",
            &format!("\"sparse_infill_pattern\": \"{pattern}\""),
        );
        let mut raw = super::super::prepare(archive);
        for object in &mut raw.objects {
            object.surfaces_by_layer.clear();
        }
        super::assert_unsupported_raw(raw, "sparse_infill_pattern");
    }
}

#[test]
fn task22o71_inactive_adaptive_density_is_a_noop_without_candidates() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"sparse_infill_pattern\": \"crosshatch\"",
        "\"sparse_infill_pattern\": \"adaptivecubic\"",
    );
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"sparse_infill_density\": \"15%\"",
        "\"sparse_infill_density\": \"0%\"",
    );
    let mut raw = super::super::prepare(archive);
    raw.objects[0].surfaces_by_layer.clear();

    transaction::dispose(transaction::prepare(raw).unwrap());
}

#[test]
fn task22o71_zero_sparse_density_preserves_real_bridge_candidates() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"sparse_infill_density\": \"15%\"",
        "\"sparse_infill_density\": \"0%\"",
    );

    let raw = super::super::prepare(archive);
    assert!(
        raw.objects
            .iter()
            .any(|object| !object.surfaces_by_layer.is_empty())
    );

    let prepared = transaction::prepare(raw).unwrap();
    let snapshot = super::snapshot(&prepared);
    assert!(!snapshot.bridge_layers.is_empty());
    assert!(snapshot.bridge_surfaces > 0);
    assert!(snapshot.bridge_expolygon_points > snapshot.bridge_surfaces);
    transaction::dispose(prepared);
}

#[test]
fn task22o71_adaptive_pattern_on_an_empty_object_is_a_noop() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"sparse_infill_pattern\": \"crosshatch\"",
        "\"sparse_infill_pattern\": \"supportcubic\"",
    );
    let mut raw = super::super::prepare(archive);
    raw.objects[0].surfaces_by_layer.clear();
    for record in raw.predecessor.predecessor.objects[0]
        .records
        .iter_mut()
        .flatten()
    {
        record.fill_surfaces.clear();
    }

    transaction::dispose(transaction::prepare(raw).unwrap());
}

#[test]
fn task22o71_unported_anchor_density_and_lengths_fail_without_fallback() {
    assert_unsupported_mutation(
        "\"top_surface_density\": \"100%\"",
        "\"top_surface_density\": \"0%\"",
        "top_surface_density",
    );
    assert_unsupported_mutation(
        "\"infill_anchor_max\": \"20\"",
        "\"infill_anchor_max\": \"0\"",
        "infill_anchor_max",
    );
    assert_unsupported_mutation(
        "\"infill_anchor\": \"400%\"",
        "\"infill_anchor\": \"-1\"",
        "infill_anchor",
    );
}

#[test]
fn task22o71_unported_anchor_rotation_templates_fail_without_fallback() {
    assert_unsupported_mutation(
        "\"sparse_infill_rotate_template\": \"\"",
        "\"sparse_infill_rotate_template\": \"+45N2\"",
        "sparse_infill_rotate_template",
    );
    assert_unsupported_mutation(
        "\"solid_infill_rotate_template\": \"\"",
        "\"solid_infill_rotate_template\": \"+45N2\"",
        "solid_infill_rotate_template",
    );
}

#[test]
fn task22o71_unported_anchor_direction_controls_fail_without_fallback() {
    assert_unsupported_mutation(
        "\"fill_multiline\": \"1\"",
        "\"fill_multiline\": \"2\"",
        "fill_multiline",
    );
}

#[test]
fn task22o71_unported_anchor_group_params_fail_without_fallback() {
    assert_unsupported_mutation(
        "\"top_surface_pattern\": \"monotonicline\"",
        "\"top_surface_pattern\": \"gyroid\"",
        "top_surface_pattern",
    );
    assert_unsupported_mutation(
        "\"internal_solid_infill_pattern\": \"monotonic\"",
        "\"internal_solid_infill_pattern\": \"gyroid\"",
        "internal_solid_infill_pattern",
    );
    assert_unsupported_mutation(
        "\"top_surface_filament_id\": \"0\"",
        "\"top_surface_filament_id\": \"2\"",
        "bridge_over_infill_anchor_extruder_order",
    );
}

#[test]
fn task22o71_invalid_nominal_anchor_flow_returns_an_error_instead_of_panicking() {
    assert_error_mutation(
        "\"sparse_infill_line_width\": \"0.45\"",
        "\"sparse_infill_line_width\": \"0.001\"",
        SliceError::InvalidInput("invalid external perimeter flow spacing".to_owned()),
    );
    assert_error_mutation(
        "\"sparse_infill_line_width\": \"0.45\"",
        ONE_ULP_ABOVE_MINIMUM_POSITIVE_FLOW_WIDTH,
        SliceError::InvalidInput("invalid Orca option sparse_infill_line_width".to_owned()),
    );
}

// This keeps Flow spacing positive but makes the scaled CrossHatch grid truncate to zero.
const ONE_ULP_ABOVE_MINIMUM_POSITIVE_FLOW_WIDTH: &str =
    "\"sparse_infill_line_width\": \"0.04292037\"";
