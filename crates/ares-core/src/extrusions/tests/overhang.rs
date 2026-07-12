use super::*;

#[test]
fn overhang_perimeter_width_uses_external_wall_width() {
    let options = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.5, (0.36, 0.44), 0.4);

    assert_eq!(
        options.width_for_role(PrintPathRole::OverhangPerimeter),
        options.width_for_role(PrintPathRole::ExternalPerimeter)
    );
}

#[test]
fn overhang_flow_ratio_scales_only_overhang_extrusion() {
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.4, 0.4), 0.4);
    let scaled = base.with_overhang_flow_ratio(0.5);
    let base_e = base
        .extrusion_per_mm(PrintPathRole::OverhangPerimeter, 0.2)
        .unwrap();

    assert_eq!(
        (scaled
            .extrusion_per_mm(PrintPathRole::OverhangPerimeter, 0.2)
            .unwrap()
            * 1_000_000.0)
            .round(),
        (base_e * 0.5 * 1_000_000.0).round()
    );
    assert_eq!(
        (scaled
            .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
            .unwrap()
            * 1_000_000.0)
            .round(),
        (base
            .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
            .unwrap()
            * 1_000_000.0)
            .round()
    );
}
