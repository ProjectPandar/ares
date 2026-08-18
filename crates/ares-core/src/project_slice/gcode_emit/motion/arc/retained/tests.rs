use crate::geometry::CoordinateScale;
use crate::project_slice::perimeters::classic::materialize::{FittedArc, FittedMove};

#[test]
fn clipped_arc_endpoint_uses_scaled_integer_vector() {
    let mut points = [(1.0, 0.0), (0.8, 0.5)];
    let mut fitting = vec![FittedMove {
        start: 0,
        end: 1,
        arc: Some(FittedArc {
            center: (0.0, 0.0),
            radius: 1.0,
            length: std::f64::consts::FRAC_PI_2,
            clockwise: false,
        }),
    }];

    super::clip_end(&mut points, &mut fitting, CoordinateScale::Normal);
    assert_eq!(
        points[1],
        (
            CoordinateScale::Normal.unscale(847_998),
            CoordinateScale::Normal.unscale(529_998),
        )
    );
}
