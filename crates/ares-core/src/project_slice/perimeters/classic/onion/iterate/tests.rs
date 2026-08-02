use crate::geometry::{ExPolygon, Point, Polygon};

use super::{IterationInput, apply, gap_deltas, offset2_deltas};
use crate::project_slice::perimeters::classic::onion::config::ValidatedOnionConfig;

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn rectangle(x0: i64, y0: i64, x1: i64, y1: i64) -> ExPolygon {
    ExPolygon::new(
        polygon(&[(x0, y0), (x1, y0), (x1, y1), (x0, y1)]),
        Vec::new(),
    )
}

fn config(
    gap: bool,
    density: i32,
    external_to_internal: i64,
    perimeter: i64,
    minimum: i64,
) -> ValidatedOnionConfig {
    ValidatedOnionConfig {
        sparse_infill_density: density,
        has_gap_fill: gap,
        minimum_spacing: minimum,
        external_to_internal_spacing: external_to_internal,
        perimeter_spacing: perimeter,
    }
}

fn run(
    loop_number: i32,
    normal: &[ExPolygon],
    smaller: &[ExPolygon],
    remaining: &[ExPolygon],
    config: ValidatedOnionConfig,
) -> super::IterationResult {
    apply(IterationInput {
        initial_loop_number: loop_number,
        loop_number,
        normal_first_offset: normal,
        smaller_first_offset: smaller,
        remaining,
        config,
    })
    .unwrap()
}

#[test]
fn task22o3_uses_source_exact_f64_then_f32_fixed_coordinate_deltas() {
    let distance = 33_554_435;
    let minimum = 16_777_219;
    let (first, second) = offset2_deltas(distance, minimum);
    let (gap_outer, gap_inner) = gap_deltas(distance);

    assert_eq!(
        first.to_bits(),
        (-((distance as f64 + minimum as f64 / 2.0 - 1.0) as f32)).to_bits()
    );
    assert_eq!(
        second.to_bits(),
        ((minimum as f64 / 2.0 - 1.0) as f32).to_bits()
    );
    assert_eq!(gap_outer, -((0.5_f64 * distance as f64) as f32));
    assert_eq!(gap_inner, (0.5_f64 * distance as f64 + 10.0) as f32);
    let (odd_first, odd_second) = offset2_deltas(1_001, 501);
    assert_ne!(odd_first, -((1_001.0_f64 + 501.0 / 2.0) as f32));
    assert_ne!(odd_second, (501.0_f64 / 2.0) as f32);
    assert_ne!(odd_second, (501 / 2 - 1) as f32);
    assert_ne!(gap_inner, (0.5_f64 * distance as f64 + 10_000_000.0) as f32);
}

#[test]
fn task22o3_depth_one_spacing_then_perimeter_spacing_builds_ordered_shells() {
    let first = vec![rectangle(0, 0, 20_000, 20_000)];
    let result = run(2, &first, &[], &first, config(false, 0, 1_000, 2_000, 500));

    assert_eq!(
        result
            .shells
            .iter()
            .map(|shell| shell.depth)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    let bounds = result
        .shells
        .iter()
        .map(|shell| {
            shell.normal[0]
                .contour()
                .points()
                .iter()
                .map(|point| point.x())
                .min()
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(bounds, [0, 1_000, 3_000]);
    assert_eq!(result.effective_loop_number, 2);
    assert_eq!(result.last, result.shells[2].normal);
}

#[test]
fn task22o3_gap_only_iteration_keeps_last_and_does_not_store_extra_shell() {
    let first = vec![rectangle(0, 0, 20_000, 20_000)];
    let without_extra = run(2, &first, &[], &first, config(true, 0, 1_000, 1_000, 500));
    let with_extra = run(2, &first, &[], &first, config(true, 20, 1_000, 1_000, 500));

    assert_eq!(with_extra.shells.len(), 3);
    assert_eq!(with_extra.last, without_extra.last);
    assert!(with_extra.gaps.len() >= without_extra.gaps.len());
}

#[test]
fn task22o3_appends_gap_before_gap_only_termination() {
    let mixed = vec![
        rectangle(0, 0, 20_000, 20_000),
        rectangle(30_000, 0, 31_500, 10_000),
    ];
    let result = run(0, &mixed, &[], &mixed, config(true, 20, 1_000, 1_000, 500));

    assert_eq!(result.shells.len(), 1);
    assert_eq!(result.last, mixed);
    assert!(!result.gaps.is_empty());
}

#[test]
fn task22o3_appends_gap_before_collapse_then_clears_last() {
    let narrow = vec![rectangle(0, 0, 1_500, 10_000)];
    let result = run(
        2,
        &narrow,
        &[],
        &narrow,
        config(true, 20, 1_000, 1_000, 500),
    );

    assert_eq!(result.effective_loop_number, 0);
    assert_eq!(result.shells.len(), 1);
    assert!(result.last.is_empty());
    assert!(!result.gaps.is_empty());
    assert_eq!(
        result.gaps,
        run(
            2,
            &narrow,
            &[],
            &narrow,
            config(true, 20, 1_000, 1_000, 500)
        )
        .gaps,
    );
}

#[test]
fn task22o3_no_loop_and_depth_zero_collapse_have_no_raw_shell() {
    let first = vec![rectangle(0, 0, 10_000, 10_000)];
    let no_loop = run(-1, &first, &[], &first, config(true, 20, 1_000, 1_000, 500));
    let collapsed = run(2, &[], &[], &[], config(true, 20, 1_000, 1_000, 500));

    for result in [no_loop, collapsed] {
        assert_eq!(result.effective_loop_number, -1);
        assert!(result.shells.is_empty());
        assert!(result.last.is_empty());
        assert!(result.gaps.is_empty());
    }
}

#[test]
fn task22o3_density_zero_and_disabled_gap_stop_without_extra_iteration() {
    let first = vec![rectangle(0, 0, 10_000, 10_000)];
    let zero_density = run(0, &first, &[], &first, config(true, 0, 1_000, 1_000, 500));
    let disabled = run(0, &first, &[], &first, config(false, 20, 1_000, 1_000, 500));

    assert_eq!(zero_density.shells.len(), 1);
    assert!(zero_density.gaps.is_empty());
    assert_eq!(disabled.shells.len(), 1);
    assert!(disabled.gaps.is_empty());

    let narrow = vec![rectangle(0, 0, 1_500, 10_000)];
    let negative = run(
        0,
        &narrow,
        &[],
        &narrow,
        config(true, -1, 1_000, 1_000, 500),
    );
    assert!(negative.last.is_empty());
}

#[test]
fn task22o3_preserves_normal_and_smaller_depth_zero_but_remaining_seeds_last() {
    let normal = vec![rectangle(0, 0, 8_000, 8_000)];
    let smaller = vec![rectangle(10_000, 0, 12_000, 2_000)];
    let remaining = vec![rectangle(20_000, 0, 26_000, 3_000)];
    let result = run(
        0,
        &normal,
        &smaller,
        &remaining,
        config(false, 0, 1_000, 1_000, 500),
    );

    assert_eq!(result.shells[0].normal, normal);
    assert_eq!(result.shells[0].smaller_width, smaller);
    assert_eq!(result.last, remaining);

    let smaller_only = run(1, &[], &smaller, &[], config(false, 0, 1_000, 1_000, 500));
    assert_eq!(smaller_only.shells[0].smaller_width, smaller);
    assert_eq!(smaller_only.effective_loop_number, 0);
    assert!(smaller_only.last.is_empty());
}

#[test]
fn task22o3_keeps_depth_zero_holes_and_disjoint_source_order_raw() {
    let with_hole = ExPolygon::new(
        polygon(&[(0, 0), (8_000, 0), (8_000, 8_000), (0, 8_000)]),
        vec![polygon(&[
            (2_000, 2_000),
            (2_000, 6_000),
            (6_000, 6_000),
            (6_000, 2_000),
        ])],
    );
    let disjoint = rectangle(10_000, 0, 12_000, 2_000);
    let first = vec![with_hole, disjoint];
    let result = run(0, &first, &[], &first, config(false, 0, 1_000, 1_000, 500));

    assert_eq!(result.shells[0].normal, first);
    assert_eq!(result.shells[0].normal[0].holes().len(), 1);
    assert_eq!(result.shells[0].normal[1].holes().len(), 0);
}
