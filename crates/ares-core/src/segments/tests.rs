use super::*;
use crate::{InputFormat, Layer, Model, Point3, Triangle};

#[test]
fn slice_layers_intersects_triangle_at_layer_z() {
    let model = single_sloped_triangle();
    let layers = [Layer::new(0, 0.2, 0.2)];

    let slices = slice_layers(&model, &layers).unwrap();

    assert_eq!(slices.len(), 1);
    assert_eq!(slices[0].layer_id(), 0);
    assert_eq!(slices[0].print_z(), 0.2);
    assert_eq!(
        slices[0].segments(),
        &[Segment2::new(Point2::new(0.5, 0.0), Point2::new(0.0, 0.5))]
    );
}

#[test]
fn slice_layers_deduplicates_vertex_on_plane_points() {
    let model = Model::new(
        InputFormat::Stl,
        vec![Triangle::new([
            Point3::new(0.0, 0.0, 0.2),
            Point3::new(1.0, 0.0, 0.4),
            Point3::new(0.0, 1.0, 0.0),
        ])],
    );
    let layers = [Layer::new(0, 0.2, 0.2)];

    let slices = slice_layers(&model, &layers).unwrap();

    assert_eq!(
        slices[0].segments(),
        &[Segment2::new(Point2::new(0.0, 0.0), Point2::new(0.5, 0.5))]
    );
}

#[test]
fn intersect_edge_promotes_xy_before_interpolation() {
    let point = intersect_edge(
        Point3::new(-f32::MAX, 0.0, 0.0),
        Point3::new(f32::MAX, 0.0, 0.4),
        f64::from(0.4_f32) / 2.0,
    )
    .unwrap()
    .unwrap();

    assert_eq!(point, Point2::new(0.0, 0.0));
}

#[test]
fn slice_layers_ignores_coplanar_triangles_and_preserves_empty_layers() {
    let model = Model::new(
        InputFormat::Stl,
        vec![Triangle::new([
            Point3::new(0.0, 0.0, 0.2),
            Point3::new(1.0, 0.0, 0.2),
            Point3::new(0.0, 1.0, 0.2),
        ])],
    );
    let layers = [Layer::new(0, 0.2, 0.2)];

    let slices = slice_layers(&model, &layers).unwrap();

    assert_eq!(slices.len(), 1);
    assert!(slices[0].segments().is_empty());
}

#[test]
fn slice_layers_orders_segments_deterministically() {
    let model = Model::new(
        InputFormat::Stl,
        vec![
            Triangle::new([
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(2.0, 0.0, 0.4),
                Point3::new(1.0, 1.0, 0.4),
            ]),
            Triangle::new([
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, 0.4),
                Point3::new(0.0, 1.0, 0.4),
            ]),
        ],
    );
    let layers = [Layer::new(0, 0.2, 0.2)];

    let slices = slice_layers(&model, &layers).unwrap();

    assert_eq!(slices[0].segments()[0].start(), Point2::new(0.0, 0.5));
    assert_eq!(slices[0].segments()[1].start(), Point2::new(1.0, 0.5));
}

fn single_sloped_triangle() -> Model {
    Model::new(
        InputFormat::Stl,
        vec![Triangle::new([
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.4),
            Point3::new(0.0, 1.0, 0.4),
        ])],
    )
}
