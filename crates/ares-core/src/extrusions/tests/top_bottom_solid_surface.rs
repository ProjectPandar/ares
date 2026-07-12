use super::*;

fn round_6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

#[test]
fn top_surface_width_uses_top_width_then_line_width_then_nozzle_auto() {
    let auto = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.0, (0.0, 0.0), 0.3);
    assert_eq!(auto.width_for_role(PrintPathRole::TopSolidInfill), 0.4);

    let line = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.5, (0.0, 0.0), 0.3);
    assert_eq!(line.width_for_role(PrintPathRole::TopSolidInfill), 0.5);

    let top = line.with_top_surface_line_width(0.7);
    assert_eq!(top.width_for_role(PrintPathRole::TopSolidInfill), 0.7);
    assert_eq!(top.width_for_role(PrintPathRole::SolidInfill), 0.5);
}

#[test]
fn top_and_bottom_flow_ratios_scale_only_their_surface_roles() {
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.3);
    let scaled = base
        .with_top_solid_infill_flow_ratio(0.5)
        .with_bottom_solid_infill_flow_ratio(0.25);

    assert_eq!(
        round_6(
            scaled
                .extrusion_per_mm(PrintPathRole::TopSolidInfill, 0.2)
                .unwrap()
        ),
        round_6(
            base.extrusion_per_mm(PrintPathRole::TopSolidInfill, 0.2)
                .unwrap()
                * 0.5
        )
    );
    assert_eq!(
        round_6(
            scaled
                .extrusion_per_mm(PrintPathRole::BottomSurface, 0.2)
                .unwrap()
        ),
        round_6(
            base.extrusion_per_mm(PrintPathRole::BottomSurface, 0.2)
                .unwrap()
                * 0.25
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
        )
    );
}

#[test]
fn bottom_surface_composes_initial_layer_width_and_first_layer_flow() {
    let options = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.3)
        .with_initial_layer_line_width(0.8)
        .with_first_layer_flow_ratio(0.5)
        .with_bottom_solid_infill_flow_ratio(0.25);

    let expected = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.8, (0.0, 0.0), 0.3)
        .extrusion_per_mm(PrintPathRole::SolidInfill, 0.2)
        .unwrap()
        * 0.5
        * 0.25;

    assert_eq!(
        round_6(
            options
                .extrusion_per_mm_for_layer(PrintPathRole::BottomSurface, 0.2, true)
                .unwrap()
        ),
        round_6(expected)
    );
}
