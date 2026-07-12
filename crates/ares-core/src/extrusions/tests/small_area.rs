use super::*;

#[test]
fn default_small_area_model_matches_orca_pchip_vectors() {
    let compensation = compensation(true, true, true);

    for (length, expected) in [
        (0.1, 0.246996362897),
        (0.3, 0.545340214541),
        (1.0, 0.797266684388),
        (4.0, 0.939899413511),
        (7.5, 0.977423853211),
    ] {
        assert!(
            (compensation.multiplier(PrintPathRole::SolidInfill, false, length) - expected).abs()
                < 1e-9,
            "length {length}"
        );
    }
    assert_eq!(
        compensation.multiplier(PrintPathRole::SolidInfill, false, 0.0),
        1.0
    );
    assert_eq!(
        compensation.multiplier(PrintPathRole::SolidInfill, false, 10.1),
        1.0
    );
}

#[test]
fn small_area_multiplier_respects_role_and_pattern_gates() {
    let all = compensation(true, true, true);
    let bottom_only = compensation(true, false, false);
    let top_only = compensation(false, false, true);
    let no_bottom = compensation(false, true, true);
    let no_internal = compensation(true, false, true);
    let no_top = compensation(true, true, false);
    let none = compensation(false, false, false);

    assert!(all.multiplier(PrintPathRole::SolidInfill, true, 0.1) < 1.0);
    assert!(all.multiplier(PrintPathRole::TopSolidInfill, true, 0.1) < 1.0);
    assert!(all.multiplier(PrintPathRole::BottomSurface, true, 0.1) < 1.0);
    assert!(bottom_only.multiplier(PrintPathRole::SolidInfill, true, 0.1) < 1.0);
    assert!(bottom_only.multiplier(PrintPathRole::TopSolidInfill, true, 0.1) < 1.0);
    assert!(bottom_only.multiplier(PrintPathRole::BottomSurface, true, 0.1) < 1.0);
    assert_eq!(
        bottom_only.multiplier(PrintPathRole::SolidInfill, false, 0.1),
        1.0
    );
    assert!(top_only.multiplier(PrintPathRole::TopSolidInfill, false, 0.1) < 1.0);
    assert_eq!(
        no_bottom.multiplier(PrintPathRole::BottomSurface, true, 0.1),
        1.0
    );
    assert_eq!(
        no_internal.multiplier(PrintPathRole::SolidInfill, false, 0.1),
        1.0
    );
    assert_eq!(
        no_top.multiplier(PrintPathRole::TopSolidInfill, false, 0.1),
        1.0
    );
    assert_eq!(all.multiplier(PrintPathRole::SparseInfill, false, 0.1), 1.0);
    for role in [
        PrintPathRole::SolidInfill,
        PrintPathRole::TopSolidInfill,
        PrintPathRole::BottomSurface,
    ] {
        assert_eq!(
            none.multiplier(role, true, 0.1),
            1.0,
            "{role:?} first layer"
        );
        assert_eq!(
            none.multiplier(role, false, 0.1),
            1.0,
            "{role:?} later layer"
        );
    }
}

#[test]
fn small_area_segment_delta_scales_after_base_flow_ratios() {
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.4)
        .with_internal_solid_infill_flow_ratio(0.5);
    let scaled = base.with_small_area_infill_flow_compensation(compensation(true, true, true));

    let base_delta = base
        .extrusion_delta_for_segment(PrintPathRole::SolidInfill, 0.2, false, 0.1)
        .unwrap();
    let scaled_delta = scaled
        .extrusion_delta_for_segment(PrintPathRole::SolidInfill, 0.2, false, 0.1)
        .unwrap();

    assert!((scaled_delta / base_delta - 0.246996362897).abs() < 1e-9);
}

#[test]
fn generated_extrusion_moves_apply_small_area_multiplier_to_print_segments() {
    let layers = [Layer::new(1, 0.2, 0.4)];
    let moves = [LayerToolpathMoves::new(
        1,
        0.4,
        vec![
            ToolpathMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::SolidInfill,
                Point2::new(0.0, 0.0),
            ),
            ToolpathMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::SolidInfill,
                Point2::new(0.1, 0.0),
            ),
        ],
    )];
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.4);
    let scaled = base.with_small_area_infill_flow_compensation(compensation(true, true, true));

    let base_output = generate_extrusion_moves(&layers, &moves, base).unwrap();
    let scaled_output = generate_extrusion_moves(&layers, &moves, scaled).unwrap();

    assert!(
        scaled_output[0].total_extrusion_mm() < base_output[0].total_extrusion_mm(),
        "scaled {} base {}",
        scaled_output[0].total_extrusion_mm(),
        base_output[0].total_extrusion_mm()
    );
    assert_eq!(
        scaled_output[0].moves()[1].e_position(),
        Some(scaled_output[0].total_extrusion_mm())
    );
}

fn compensation(
    bottom_supported: bool,
    internal_supported: bool,
    top_supported: bool,
) -> SmallAreaInfillFlowCompensation {
    SmallAreaInfillFlowCompensation::parse(
        SmallAreaInfillFlowCompensation::default_model_entries()
            .iter()
            .map(|entry| (*entry).to_owned())
            .collect(),
        bottom_supported,
        internal_supported,
        top_supported,
    )
    .unwrap()
}
