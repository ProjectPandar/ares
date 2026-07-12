use super::*;

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

#[test]
fn solid_infill_uses_internal_solid_width_and_flow() {
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.3)
        .with_internal_solid_infill_line_width(0.6);
    let scaled = base.with_internal_solid_infill_flow_ratio(0.5);

    assert_eq!(scaled.width_for_role(PrintPathRole::SparseInfill), 0.3);
    assert_eq!(scaled.width_for_role(PrintPathRole::SolidInfill), 0.6);
    assert_ne!(
        round_6(
            scaled
                .extrusion_per_mm(PrintPathRole::SolidInfill, 0.2)
                .unwrap()
        ),
        round_6(
            scaled
                .extrusion_per_mm(PrintPathRole::SparseInfill, 0.2)
                .unwrap()
        )
    );
    assert_eq!(
        round_6(
            scaled
                .extrusion_per_mm(PrintPathRole::SolidInfill, 0.2)
                .unwrap()
        ),
        round_6(
            base.extrusion_per_mm(PrintPathRole::SolidInfill, 0.2)
                .unwrap()
                * 0.5
        )
    );
}

#[test]
fn solid_infill_keeps_first_layer_line_width_and_flow_treatment() {
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.3)
        .with_internal_solid_infill_line_width(0.6);
    let scaled = base
        .with_initial_layer_line_width(0.8)
        .with_first_layer_flow_ratio(0.25);

    assert_eq!(
        round_6(
            scaled
                .extrusion_per_mm_for_layer(PrintPathRole::SolidInfill, 0.2, true)
                .unwrap()
        ),
        round_6(
            ExtrusionOptions::new_for_tests(0.4, 2.0, 0.8, (0.0, 0.0), 0.3)
                .extrusion_per_mm(PrintPathRole::SolidInfill, 0.2)
                .unwrap()
                * 0.25
        )
    );
}
