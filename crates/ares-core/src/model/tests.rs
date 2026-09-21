use super::*;
use crate::InputFormat;

#[test]
fn model_reports_z_bounds() {
    let model = Model::new(
        InputFormat::Stl,
        vec![Triangle::new([
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.2),
            Point3::new(0.0, 1.0, 0.4),
        ])],
    );
    let bounds = model.z_bounds().unwrap();

    assert_eq!(bounds.min, 0.0);
    assert_eq!(bounds.max, 0.4);
}

#[test]
fn model_rejects_non_finite_z_bounds() {
    let model = Model::new(
        InputFormat::Stl,
        vec![Triangle::new([
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, f32::NAN),
            Point3::new(0.0, 1.0, 0.4),
        ])],
    );

    assert_eq!(model.z_bounds(), None);
}

#[test]
fn model_reports_xy_bounds() {
    let model = Model::new(
        InputFormat::Stl,
        vec![
            Triangle::new([
                Point3::new(-2.0, 1.0, 0.0),
                Point3::new(3.0, -4.0, 0.2),
                Point3::new(0.0, 2.0, 0.4),
            ]),
            Triangle::new([
                Point3::new(1.0, 5.0, 0.0),
                Point3::new(-1.0, -3.0, 0.2),
                Point3::new(2.0, 0.0, 0.4),
            ]),
        ],
    );
    let bounds = model.xy_bounds().unwrap();

    assert_eq!(bounds.min_x, -2.0);
    assert_eq!(bounds.max_x, 3.0);
    assert_eq!(bounds.min_y, -4.0);
    assert_eq!(bounds.max_y, 5.0);
}
