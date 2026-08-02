use super::super::{Point, ThickLine};

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
