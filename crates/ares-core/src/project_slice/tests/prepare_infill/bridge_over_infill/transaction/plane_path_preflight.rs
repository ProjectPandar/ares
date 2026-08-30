use crate::project_slice::{
    prepare_infill::bridge_over_infill::transaction, tests::support::KsrArchive,
};

#[test]
fn plane_path_solid_patterns_pass_anchor_projection_preflight() {
    for pattern in ["hilbertcurve", "archimedeanchords", "octagramspiral"] {
        let mut archive = KsrArchive::new();
        archive.replace_unique(
            "Metadata/project_settings.config",
            "\"top_surface_pattern\": \"monotonicline\"",
            &format!("\"top_surface_pattern\": \"{pattern}\""),
        );
        archive.replace_unique(
            "Metadata/project_settings.config",
            "\"internal_solid_infill_pattern\": \"monotonic\"",
            &format!("\"internal_solid_infill_pattern\": \"{pattern}\""),
        );

        let prepared = transaction::prepare(super::super::prepare(archive)).unwrap();

        transaction::dispose(prepared);
    }
}
