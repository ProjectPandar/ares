use super::helpers::{execute, polygons};
use crate::geometry::clipper::{ClipOperation, ClipperOptions, FillRule};

#[test]
fn task22f_preserve_collinear_option_changes_complete_ordered_output() {
    let input: &[&[(i64, i64)]] = &[&[(0, 0), (10, 0), (20, 0), (20, 20), (0, 20)]];
    let without = execute(
        polygons(input),
        Vec::new(),
        ClipOperation::Union,
        (FillRule::NonZero, FillRule::NonZero),
        ClipperOptions::default(),
    );
    let with = execute(
        polygons(input),
        Vec::new(),
        ClipOperation::Union,
        (FillRule::NonZero, FillRule::NonZero),
        ClipperOptions {
            preserve_collinear: true,
            ..ClipperOptions::default()
        },
    );

    assert_eq!(without, polygons(&[&[(20, 20), (0, 20), (0, 0), (20, 0)]]));
    assert_eq!(
        with,
        polygons(&[&[(20, 20), (0, 20), (0, 0), (10, 0), (20, 0)]])
    );
}

#[test]
fn task22f_reverse_solution_option_reverses_fixed_output_orientation() {
    let input: &[&[(i64, i64)]] = &[&[(0, 0), (40, 0), (40, 40), (0, 40)]];
    let forward = execute(
        polygons(input),
        Vec::new(),
        ClipOperation::Union,
        (FillRule::NonZero, FillRule::NonZero),
        ClipperOptions::default(),
    );
    let reversed = execute(
        polygons(input),
        Vec::new(),
        ClipOperation::Union,
        (FillRule::NonZero, FillRule::NonZero),
        ClipperOptions {
            reverse_solution: true,
            ..ClipperOptions::default()
        },
    );

    assert_eq!(forward, polygons(&[&[(40, 40), (0, 40), (0, 0), (40, 0)]]));
    assert_eq!(reversed, polygons(&[&[(0, 0), (0, 40), (40, 40), (40, 0)]]));
}
