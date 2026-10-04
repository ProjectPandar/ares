use super::*;
use crate::geometry::Point;

fn line(a: (f64, f64), b: (f64, f64), owner: usize, support: bool) -> OwnedLine {
    let scale = CoordinateScale::Normal;
    let point = |(x, y)| {
        Point::new(
            scale.checked_scale(x).unwrap(),
            scale.checked_scale(y).unwrap(),
        )
    };
    OwnedLine {
        line: Line::new(point(a), point(b)),
        owner,
        support,
    }
}

#[test]
fn conflicts_preserve_instance_identity_and_support_thresholds() {
    let scale = CoordinateScale::Normal;
    let first = line((-5.0, 0.0), (5.0, 0.0), 0, false);
    let mut second = line((0.0, -5.0), (0.0, 5.0), 0, false);
    assert!(find(&[first, second], scale).is_none());
    let mut first = line((-5.0, 0.0), (5.0, 0.0), 0, false);
    second = line((0.0, -5.0), (0.0, 5.0), 1, false);
    assert!(intersects(&first, &second, scale));
    first.support = true;
    second.support = true;
    assert!(!intersects(&first, &second, scale));
    second = line((-2.0, 0.0), (2.0, 0.0), 1, true);
    assert!(
        intersects(&first, &second, scale),
        "collinear overlap precedes the support threshold"
    );
}

#[test]
fn endpoint_contacts_are_not_crossings() {
    let scale = CoordinateScale::Normal;
    let first = line((0.0, 0.0), (10.0, 0.0), 0, false);
    for x in [0.0, 0.005, 0.01] {
        let second = line((x, -1.0), (x, 1.0), 1, false);
        assert!(!intersects(&first, &second, scale));
    }
    assert!(intersects(
        &first,
        &line((0.011, -1.0), (0.011, 1.0), 1, false),
        scale
    ));
}

#[test]
fn rasterization_preserves_truncating_negative_cells_and_axis_completion() {
    let source = line((-2.5, -1.5), (-0.5, 2.5), 0, false);
    let mut cells = Vec::new();
    rasterize(source.line, 1_000_000, &mut cells);
    assert_eq!(
        cells,
        [(-2, -1), (-2, 0), (-2, 1), (-1, 1), (-1, 2), (0, 2)]
    );
}
