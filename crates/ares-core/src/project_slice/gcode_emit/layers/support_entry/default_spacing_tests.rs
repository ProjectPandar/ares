use super::default_perimeter_spacing;
use crate::{
    OrcaFloat, OrcaFloats, load_project, project::effective_config::resolve_bounded_project_config,
};

// Failure modes: using model flow spacing, selected support nozzle, inactive
// configured slots, or truncating the scaled f32 mean through millimetres.
// The qualified history probe constructs support cache at scaled spacing400000
// before object travel reuses it (dn62si3..., 2026-10-08).
#[test]
fn support_default_spacing_averages_only_object_printing_nozzles_in_scaled_float() {
    let project = load_project(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/parity/validation/tiq2-type2-enabled.3mf"
    )))
    .unwrap();
    let resolved = resolve_bounded_project_config(&project).unwrap();
    let object = &resolved.objects[0];
    let source = &project.objects()[object.source_object_index];
    let mut settings = resolved.views.full;
    for (diameters, expected) in [
        (vec![0.4, 0.4, 1.2], 400_000.0),
        (vec![0.4, 0.6, 1.2], 500_000.0),
        (vec![0.4, 0.5, 1.2], 450_000.0),
    ] {
        settings.project.print.nozzle_diameter =
            OrcaFloats(diameters.into_iter().map(OrcaFloat).collect());
        assert_eq!(
            default_perimeter_spacing(&settings, object, source, false),
            expected
        );
    }
}
