use super::assert_error_mutation;
use crate::{
    SliceError,
    project_slice::{prepare_infill::bridge_over_infill::transaction, tests::support::KsrArchive},
};

#[test]
fn task22o71_honeycomb_pattern_admitted() {
    // `honeycomb` is an implemented sparse-anchoring arm (the deferred-pattern
    // gate is gone); the transaction admits it.
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"sparse_infill_pattern\": \"crosshatch\"",
        "\"sparse_infill_pattern\": \"honeycomb\"",
    );
    let mut raw = super::super::prepare(archive);
    for object in &mut raw.objects {
        object.surfaces_by_layer.clear();
    }
    let prepared = transaction::prepare(raw);
    assert!(prepared.is_ok(), "honeycomb should pass the transaction");
    transaction::dispose(prepared.unwrap());
}

#[test]
fn task22o71_adaptive_patterns_admitted_without_candidates() {
    // The octree gate is gone (FillAdaptive port); with no fill surfaces
    // the transaction completes for both adaptive patterns.
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
        let prepared = transaction::prepare(raw);
        assert!(
            prepared.is_ok(),
            "{pattern} should pass the transaction without fill surfaces"
        );
        transaction::dispose(prepared.unwrap());
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
fn task22o71_zero_top_density_reaches_empty_fill_grouping() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"top_surface_density\": \"100%\"",
        "\"top_surface_density\": \"0%\"",
    );

    let prepared = transaction::prepare(super::super::prepare(archive)).unwrap();

    transaction::dispose(prepared);
}

#[test]
fn task22o71_multiline_reaches_the_fill_grouping_stage() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"fill_multiline\": \"1\"",
        "\"fill_multiline\": \"2\"",
    );

    let prepared = transaction::prepare(super::super::prepare(archive)).unwrap();

    transaction::dispose(prepared);
}

#[test]
fn task22o71_divergent_role_filament_ids_are_admitted() {
    // `PrintObject::bridge_over_infill` (`PrintObject.cpp:2468+`) is
    // extruder-agnostic: rewritten surfaces stay in their LayerRegion and
    // emit on the region's role filament (`Fill.cpp:865`,
    // `Fill.cpp:920-924`), so a `top_surface_filament_id` differing from
    // the sparse/solid filament is a supported configuration.
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"top_surface_filament_id\": \"0\"",
        "\"top_surface_filament_id\": \"2\"",
    );
    let prepared = transaction::prepare(super::super::prepare(archive))
        .expect("divergent role filament ids must be admitted");
    transaction::dispose(prepared);
}

#[test]
fn task22o71_invalid_nominal_anchor_flow_returns_an_error_instead_of_panicking() {
    assert_error_mutation(
        "\"sparse_infill_line_width\": \"0.45\"",
        "\"sparse_infill_line_width\": \"0.001\"",
        SliceError::InvalidInput("Flow::spacing() produced negative spacing. Did you set some extrusion width too small?".to_owned()),
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
