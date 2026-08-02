use super::super::{Line, Point, ThickLine};

#[test]
fn task22o12_thick_line_constructors_preserve_endpoints_and_widths() {
    let a = Point::new(-3, 7);
    let b = Point::new(11, -5);
    assert_eq!(
        ThickLine::new(a, b),
        ThickLine {
            a,
            b,
            a_width: 0.0,
            b_width: 0.0,
        }
    );
    assert_eq!(
        ThickLine::with_widths(a, b, 1.25, 9.5),
        ThickLine {
            a,
            b,
            a_width: 1.25,
            b_width: 9.5,
        }
    );
}

#[test]
fn task22o13_line_projection_intersection_length_and_orientation_are_literal() {
    let horizontal = Line::new(Point::new(0, 0), Point::new(10, 0));
    assert_eq!(horizontal.projection(Point::new(4, 7)), Point::new(4, 0));
    assert_eq!(horizontal.distance_to(Point::new(4, 7)), 7.0);
    assert_eq!(horizontal.length(), 10.0);
    assert_eq!(horizontal.orientation().to_bits(), 0.0_f64.to_bits());
    assert_eq!(
        horizontal.intersection(Line::new(Point::new(4, -5), Point::new(4, 5))),
        Some(Point::new(4, 0))
    );
    assert_eq!(
        horizontal.intersection(Line::new(Point::new(0, 2), Point::new(10, 2))),
        None
    );
}

#[test]
fn task22o13_line_distance_keeps_source_sqrt_order_and_zero_length_behavior() {
    let diagonal = Line::new(Point::new(-3, -4), Point::new(5, 11));
    assert_eq!(diagonal.length().to_bits(), 17.0_f64.to_bits());
    assert_eq!(
        Line::new(Point::new(2, -3), Point::new(2, -3)).distance_to(Point::new(5, 1)),
        5.0
    );
    assert_eq!(
        Line::new(Point::new(0, 0), Point::new(5, 2)).projection(Point::new(2, 2)),
        Point::new(2, 0)
    );
}

#[test]
fn task22o13_line_relative_distance_negative_orientation_and_cast_are_literal() {
    const N: i64 = 1_i64 << 53;
    assert_eq!(
        Line::new(Point::new(N, 0), Point::new(N + 2, 2))
            .distance_to(Point::new(N + 1, 0))
            .to_bits(),
        0x3fe6_a09e_667f_3bcd
    );
    assert_eq!(
        Line::new(Point::new(N, 0), Point::new(N + 10, 10))
            .distance_to(Point::new(N + 5, 7))
            .to_bits(),
        0x3ff6_a09e_667f_3bcd
    );
    assert_eq!(
        Line::new(Point::new(0, 0), Point::new(1, -1))
            .orientation()
            .to_bits(),
        0x4015_fdbb_e9bb_a775_u64
    );
    assert_eq!(
        Line::new(Point::new(0, 0), Point::new(10, 0))
            .intersection(Line::new(Point::new(2, -1), Point::new(3, 1))),
        Some(Point::new(2, 0))
    );
}
