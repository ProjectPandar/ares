use crate::{
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::prepare_infill::{
        vertical_shell_filtering::{self, GeometryStep, filter},
        vertical_shell_regularization::types::VerticalShellRegularization,
        vertical_shell_trimming::types::VerticalShellTrim,
    },
};

use super::{empty_record, rectangle};

#[test]
fn task22o23_below_equal_above_both_thresholds_freeze_strict_branches() {
    for (area, visibility, expansion, survivors) in [
        (31_499_999, false, true, 0),
        (31_500_000, true, false, 1),
        (31_500_001, true, false, 1),
        (167_999_999, true, false, 1),
        (168_000_000, false, false, 1),
        (168_000_001, false, false, 1),
    ] {
        vertical_shell_filtering::reset_geometry_hooks();
        let output = run(ExPolygon::new(rectangle(0, 0, area, 1), Vec::new()));
        let events = vertical_shell_filtering::geometry_events();
        assert_eq!(
            events.contains(&GeometryStep::VisibilityDifference),
            visibility,
            "area {area} visibility branch"
        );
        assert_eq!(
            events.contains(&GeometryStep::CandidateExpansion),
            expansion,
            "area {area} expansion branch"
        );
        assert_eq!(output.filtered_shell.len(), survivors, "area {area}");
    }
}

#[test]
fn task22o23_negative_signed_area_rejects_final_absolute_value() {
    let candidate = ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(0, 1),
            Point::new(200_000_000, 1),
            Point::new(200_000_000, 0),
        ]),
        Vec::new(),
    );
    assert!(candidate.area() < 0.0);
    vertical_shell_filtering::reset_geometry_hooks();
    let output = run(candidate);
    assert!(output.filtered_shell.is_empty());
    let events = vertical_shell_filtering::geometry_events();
    assert!(!events.contains(&GeometryStep::VisibilityDifference));
    assert!(events.contains(&GeometryStep::CandidateExpansion));
}

#[test]
fn task22o23_odd_and_above_exact_f32_spacing_freeze_every_product_bit() {
    let captured = [
        vertical_shell_filtering::threshold_bits(400_001, CoordinateScale::Normal),
        vertical_shell_filtering::threshold_bits(16_777_217, CoordinateScale::Normal),
    ];
    assert_eq!(
        captured,
        [
            [
                1_221_399_585,
                1_500_000,
                1_236_736_768,
                8_000_000,
                1_257_513_984,
                1_393_733_381,
                1_413_714_950,
                4_636_737_291_354_636_289,
            ],
            [
                1_267_099_238,
                1_500_000,
                1_236_736_768,
                8_000_000,
                1_257_513_984,
                1_438_663_359,
                1_459_629_184,
                4_636_737_291_354_636_289,
            ],
        ]
    );
}

fn run(
    candidate: ExPolygon,
) -> crate::project_slice::prepare_infill::vertical_shell_filtering::types::VerticalShellTinyFilter
{
    filter::filter_record(
        filter::RecordOperands {
            trim: &VerticalShellTrim {
                shell: vec![rectangle(0, 0, 1, 1)],
            },
            regularization: &VerticalShellRegularization {
                regularized_shell: vec![candidate],
            },
            current: &empty_record(),
            previous_lslices: None,
            next_lslices: None,
        },
        20,
        CoordinateScale::Normal,
    )
    .unwrap()
}
