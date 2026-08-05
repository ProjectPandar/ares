use crate::project_slice::{
    prepare_infill::vertical_shell_regularization, tests::support::KsrArchive,
};

use super::{fixture, metamorphic::regularization_digest};

#[test]
fn task22o22_real_3mf_active_and_inactive_modes_follow_o21_gate() {
    for mode in ["none", "ensure_critical_only", "ensure_moderate"] {
        let mut archive = KsrArchive::new();
        archive.replace_unique(
            "Metadata/project_settings.config",
            "\"ensure_vertical_shell_thickness\": \"ensure_all\"",
            &format!("\"ensure_vertical_shell_thickness\": \"{mode}\""),
        );
        vertical_shell_regularization::reset_geometry_hooks();
        let output = fixture::prepare(archive.bytes());
        assert!(output.regularizations.iter().all(|object| {
            object
                .records
                .iter()
                .flatten()
                .all(|record| record.regularized_shell.is_empty())
        }));
        assert!(vertical_shell_regularization::geometry_events().is_empty());
    }

    vertical_shell_regularization::reset_geometry_hooks();
    let active = fixture::prepare(KsrArchive::new().bytes());
    assert!(active.regularizations.iter().any(|object| {
        object
            .records
            .iter()
            .flatten()
            .any(|record| !record.regularized_shell.is_empty())
    }));
    assert!(!vertical_shell_regularization::geometry_events().is_empty());
}

#[test]
fn task22o22_model_part_ensure_precedence_activates_regularization() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"ensure_vertical_shell_thickness\": \"ensure_all\"",
        "\"ensure_vertical_shell_thickness\": \"none\"",
    );
    archive.replace(
        "Metadata/model_settings.config",
        "    <part id=\"1\" subtype=\"normal_part\">",
        "    <part id=\"1\" subtype=\"normal_part\">\n      <metadata key=\"ensure_vertical_shell_thickness\" value=\"ensure_all\"/>",
    );
    vertical_shell_regularization::reset_geometry_hooks();
    let output = fixture::prepare(archive.bytes());
    assert!(
        output.regularizations[0]
            .records
            .iter()
            .flatten()
            .any(|record| !record.regularized_shell.is_empty())
    );
    assert!(!vertical_shell_regularization::geometry_events().is_empty());
}

#[test]
fn task22o22_typed_line_width_changes_spacing_radii_and_ordered_output() {
    let baseline = fixture::prepare(KsrArchive::new().bytes());
    let baseline_spacings = spacings(&baseline);
    let baseline_digest = regularization_digest(&baseline.regularizations);

    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"internal_solid_infill_line_width\": \"0.42\"",
        "\"internal_solid_infill_line_width\": \"0.55\"",
    );
    let changed = fixture::prepare(archive.bytes());
    let changed_spacings = spacings(&changed);
    assert_ne!(changed_spacings, baseline_spacings);
    let changed_index = baseline_spacings
        .iter()
        .zip(&changed_spacings)
        .position(|(before, after)| before != after)
        .unwrap();
    let baseline_bits = vertical_shell_regularization::radii_bits(baseline_spacings[changed_index]);
    let changed_bits = vertical_shell_regularization::radii_bits(changed_spacings[changed_index]);
    assert_ne!(changed_bits, baseline_bits);
    assert_eq!(changed_bits, expected_bits(changed_spacings[changed_index]));
    assert_ne!(
        regularization_digest(&changed.regularizations),
        baseline_digest
    );
}

fn spacings(
    output: &vertical_shell_regularization::PreparedPostVerticalShellRegularization,
) -> Vec<i64> {
    output.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .records
        .iter()
        .flatten()
        .map(|record| record.solid_infill_spacing)
        .collect()
}

fn expected_bits(spacing: i64) -> [u32; 7] {
    let minimum = (spacing as f32) * 1.05_f32;
    let ensure = 0.5_f32 * 0.65_f32 * minimum;
    let sparse = 0.5_f32 * 1.2_f32 * minimum;
    let overlap = 0.2_f32 * minimum;
    [
        minimum.to_bits(),
        ensure.to_bits(),
        sparse.to_bits(),
        overlap.to_bits(),
        (-ensure).to_bits(),
        (ensure + sparse).to_bits(),
        (-(sparse - overlap)).to_bits(),
    ]
}
