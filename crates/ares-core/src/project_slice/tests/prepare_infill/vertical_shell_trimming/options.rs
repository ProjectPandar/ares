use crate::project_slice::{prepare_infill::vertical_shell_trimming, tests::support::KsrArchive};

use super::fixture;

#[test]
fn task22o21_real_3mf_inactive_modes_have_empty_trims_and_zero_events() {
    for mode in ["none", "ensure_critical_only", "ensure_moderate"] {
        let mut archive = KsrArchive::new();
        archive.replace_unique(
            "Metadata/project_settings.config",
            "\"ensure_vertical_shell_thickness\": \"ensure_all\"",
            &format!("\"ensure_vertical_shell_thickness\": \"{mode}\""),
        );
        vertical_shell_trimming::reset_geometry_hooks();
        let output = fixture::prepare(archive.bytes());
        assert!(output.trims.iter().all(|object| {
            object
                .records
                .iter()
                .flatten()
                .all(|trim| trim.shell.is_empty())
        }));
        assert!(vertical_shell_trimming::geometry_events().is_empty());
    }
}

#[test]
fn task22o21_real_3mf_active_mode_produces_characterized_trims() {
    vertical_shell_trimming::reset_geometry_hooks();
    let output = fixture::prepare(KsrArchive::new().bytes());
    assert!(
        output.trims[0]
            .records
            .iter()
            .flatten()
            .any(|trim| !trim.shell.is_empty())
    );
    assert!(!vertical_shell_trimming::geometry_events().is_empty());
}

#[test]
fn task22o21_real_3mf_full_density_activates_reachable_solid_append() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"sparse_infill_density\": \"15%\"",
        "\"sparse_infill_density\": \"100%\"",
    );
    vertical_shell_trimming::reset_geometry_hooks();
    let output = fixture::prepare(archive.bytes());
    assert!(
        output
            .objects
            .iter()
            .zip(&output.trims)
            .any(|(object, trims)| {
                object
                    .records
                    .iter()
                    .zip(&trims.records)
                    .any(|(record, trim)| match (record, trim) {
                        (Some(record), Some(trim)) => appended_solid(record, trim),
                        (None, None) => false,
                        _ => unreachable!("O21 records remain aligned"),
                    })
            })
    );
    assert!(
        vertical_shell_trimming::geometry_events()
            .contains(&vertical_shell_trimming::GeometryStep::SolidAppend)
    );
}

fn appended_solid(
    record: &crate::project_slice::prepare_infill::surface_type_detection::types::PreparedSurfaceTypeRecord,
    trim: &vertical_shell_trimming::types::VerticalShellTrim,
) -> bool {
    let solids = record
        .fill_surfaces
        .iter()
        .filter_map(|surface| {
            let (kind, expolygon, _, _, _, _) = surface.as_parts();
            (kind == crate::project_slice::region_slices::RegionSurfaceKind::InternalSolid)
                .then_some(expolygon)
        })
        .flat_map(|expolygon| std::iter::once(expolygon.contour()).chain(expolygon.holes()))
        .cloned()
        .collect::<Vec<_>>();
    !solids.is_empty() && trim.shell.ends_with(&solids)
}

#[test]
fn task22o21_model_part_ensure_precedence_activates_trimming() {
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
    vertical_shell_trimming::reset_geometry_hooks();
    let output = fixture::prepare(archive.bytes());
    assert!(
        output.trims[0]
            .records
            .iter()
            .flatten()
            .any(|trim| !trim.shell.is_empty())
    );
    assert!(!vertical_shell_trimming::geometry_events().is_empty());
}
