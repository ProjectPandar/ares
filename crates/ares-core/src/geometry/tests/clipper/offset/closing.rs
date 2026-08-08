use super::helpers::polygon;
use crate::geometry::{ClipperError, ExPolygon, JoinType, closing_ex, offset2_ex};

const HI_RANGE: i64 = 0x3fff_ffff_ffff_ffff;
type ClosingFn = fn(&[ExPolygon], f32, JoinType, f64) -> Result<Vec<ExPolygon>, ClipperError>;
const CLOSING: ClosingFn = closing_ex;

fn square(min: i64, max: i64) -> ExPolygon {
    ExPolygon::new(
        polygon(&[(min, min), (max, min), (max, max), (min, max)]),
        Vec::new(),
    )
}

#[test]
fn task22o35_closing_is_the_exact_outward_then_inward_offset() {
    let input = [square(20, 80)];
    let expected = offset2_ex(&input, 7.0, -7.0, JoinType::Miter, 3.0);

    assert!(!expected.as_ref().unwrap().is_empty());
    assert_eq!(CLOSING(&input, 7.0, JoinType::Miter, 3.0), expected);
}

#[test]
fn task22o35_closing_requires_a_strictly_positive_radius() {
    for delta in [0.0, -1.0, f32::NAN] {
        assert!(std::panic::catch_unwind(|| CLOSING(&[], delta, JoinType::Miter, 3.0)).is_err());
    }
}

#[test]
fn task22o35_closing_preserves_total_collapse() {
    let input = [ExPolygon::new(polygon(&[(0, 0)]), Vec::new())];
    assert_eq!(
        CLOSING(&input, 5.0, JoinType::Miter, 3.0),
        offset2_ex(&input, 5.0, -5.0, JoinType::Miter, 3.0)
    );
}

#[test]
fn task22o35_closing_forwards_the_first_offset_coordinate_error() {
    let invalid = [ExPolygon::new(
        polygon(&[
            (HI_RANGE + 1, 0),
            (HI_RANGE + 2, 0),
            (HI_RANGE + 2, 10),
            (HI_RANGE + 1, 10),
        ]),
        Vec::new(),
    )];
    assert_eq!(
        CLOSING(&invalid, 1.0, JoinType::Miter, 3.0),
        Err(ClipperError::CoordinateOutOfRange)
    );
}
