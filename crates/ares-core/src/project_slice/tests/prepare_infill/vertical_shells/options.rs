use crate::project_slice::tests::support::KsrArchive;

use super::fixture;

fn cache_counts(archive: KsrArchive) -> (usize, usize, usize) {
    let prepared = fixture::prepare(archive.bytes());
    prepared
        .caches
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .fold((0, 0, 0), |counts, cache| {
            (
                counts.0 + cache.top_surfaces.len(),
                counts.1 + cache.bottom_surfaces.len(),
                counts.2 + cache.holes.len(),
            )
        })
}

#[test]
fn task22o19_real_3mf_ensure_modes_control_cache_population() {
    let active = cache_counts(KsrArchive::new());
    assert_eq!(active, (572, 713, 1_227));
    for mode in ["none", "ensure_critical_only", "ensure_moderate"] {
        let mut archive = KsrArchive::new();
        archive.replace_unique(
            "Metadata/project_settings.config",
            "\"ensure_vertical_shell_thickness\": \"ensure_all\"",
            &format!("\"ensure_vertical_shell_thickness\": \"{mode}\""),
        );
        assert_eq!(cache_counts(archive), (0, 0, 0));
    }
}

#[test]
fn task22o19_model_part_override_beats_global_mode() {
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
    assert_eq!(cache_counts(archive), (572, 713, 1_227));
}
