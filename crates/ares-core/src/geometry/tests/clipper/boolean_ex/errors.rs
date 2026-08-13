use super::*;

#[test]
fn task22j_boolean_ex_forwards_subject_and_clip_coordinate_errors() {
    let invalid = vec![ExPolygon::new(
        polygon(&[(HI_RANGE + 1, 0), (0, 1), (0, 2)]),
        Vec::new(),
    )];
    let valid = vec![rectangle(0, 0, 10, 10)];

    assert_eq!(
        difference_ex(&invalid, &valid),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        difference_ex(&valid, &invalid),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        intersection_ex(&invalid, &valid),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        intersection_ex(&valid, &invalid),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        intersection_polygons_ex(&[polygon(&[(HI_RANGE + 1, 0), (0, 1), (0, 2)])], &valid,),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        intersection_polygons_ex(&[valid[0].contour().clone()], &invalid),
        Err(ClipperError::CoordinateOutOfRange)
    );
}
