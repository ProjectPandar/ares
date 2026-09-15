use super::*;
use crate::geometry::{CoordinateScale, Polygon};

fn square_surface(scale: CoordinateScale, half_mm: f64) -> ExPolygon {
    let h = scale.checked_scale_rounded(half_mm).unwrap();
    let contour = Polygon::new(vec![
        Point::new(-h, -h),
        Point::new(h, -h),
        Point::new(h, h),
        Point::new(-h, h),
    ]);
    ExPolygon::new(contour, Vec::new())
}

fn cube_mesh(half: f64) -> (Vec<[f64; 3]>, Vec<[u32; 3]>) {
    (
        vec![
            [-half, -half, -half],
            [half, -half, -half],
            [half, half, -half],
            [-half, half, -half],
            [-half, -half, half],
            [half, -half, half],
            [half, half, half],
            [-half, half, half],
        ],
        vec![
            [0, 1, 2],
            [0, 2, 3],
            [4, 6, 5],
            [4, 7, 6],
            [0, 4, 5],
            [0, 5, 1],
            [1, 5, 6],
            [1, 6, 2],
            [2, 6, 7],
            [2, 7, 3],
            [3, 7, 4],
            [3, 4, 0],
        ],
    )
}

#[test]
fn generates_lines_on_midplane() {
    let scale = CoordinateScale::Normal;
    let (vertices, triangles) = cube_mesh(5.0);
    let octree = super::super::octree::build_octree(&vertices, &triangles, &[], 0.5, false);
    let surface = square_surface(scale, 6.0);
    let polylines = fill_surface(&octree, &surface, 0.0, 0.5, 1, 1.0, 1.0, false, scale).unwrap();
    assert!(
        !polylines.is_empty(),
        "mid-plane through a full cube mesh must produce infill lines"
    );
    for polyline in &polylines {
        assert!(polyline.points().len() >= 2);
    }
}

#[test]
fn empty_above_the_octree() {
    let scale = CoordinateScale::Normal;
    let (vertices, triangles) = cube_mesh(5.0);
    let octree = super::super::octree::build_octree(&vertices, &triangles, &[], 0.5, false);
    let surface = square_surface(scale, 6.0);
    let polylines = fill_surface(&octree, &surface, 100.0, 0.5, 1, 1.0, 1.0, false, scale).unwrap();
    assert!(
        polylines.is_empty(),
        "z far above the octree produces no lines"
    );
}
