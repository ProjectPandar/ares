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
            [-half, -half, 0.0],
            [half, -half, 0.0],
            [half, half, 0.0],
            [-half, half, 0.0],
            [-half, -half, 2.0 * half],
            [half, -half, 2.0 * half],
            [half, half, 2.0 * half],
            [-half, half, 2.0 * half],
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
    let octree =
        super::super::octree::build_octree(&vertices, &triangles, &[], 5.0, false, [0.0; 3]);
    let surface = square_surface(scale, 6.0);
    let polylines = fill_surface(&octree, &surface, 5.0, 5.0, 1, 1.0, 1.0, false, scale).unwrap();
    assert!(
        !polylines.is_empty(),
        "mid-plane through a full cube mesh must produce infill lines"
    );
    for polyline in &polylines {
        assert!(polyline.points().len() >= 2);
    }
}

#[test]
fn generated_endpoints_truncate_scaled_coordinates_toward_zero() {
    let (vertices, _) = cube_mesh(5.0);
    let octree = super::super::octree::build_octree(&vertices, &[], &[], 5.0, false, [0.0; 3]);
    let center = octree.root().center();
    let properties = [super::super::octree::CubeProperties {
        edge_length: 2.0,
        height: 4.0,
        diagonal_length: 2.123456789,
        line_z_distance: 1.0,
        line_xy_distance: 0.987654321,
    }];
    for scale in [CoordinateScale::Normal, CoordinateScale::LargeBed] {
        let mut context = FillContext::new(&octree, center[2], 0, 1000, scale);
        context.cubes_properties = &properties;
        generate_infill_lines_recursive(&mut context, octree.root(), 0, 0);
        let segment = context.temp_lines[0].unwrap();
        // Point::new_scale(Vec2d) casts scale_(v) to coord_t, unlike rounding.
        let y = center[1] + 0.987654321 - 1.0 / 2.0_f64.sqrt();
        assert_eq!(
            segment.a,
            Point::new(
                scale.checked_scale(center[0] + 1.0617283945).unwrap(),
                scale.checked_scale(y).unwrap(),
            )
        );
        assert_eq!(
            segment.b,
            Point::new(
                scale.checked_scale(center[0] - 1.0617283945).unwrap(),
                scale.checked_scale(y).unwrap(),
            )
        );
    }
}

#[test]
fn empty_above_the_octree() {
    let scale = CoordinateScale::Normal;
    let (vertices, triangles) = cube_mesh(5.0);
    let octree =
        super::super::octree::build_octree(&vertices, &triangles, &[], 0.5, false, [0.0; 3]);
    let surface = square_surface(scale, 6.0);
    let polylines = fill_surface(&octree, &surface, 500.0, 0.5, 1, 1.0, 1.0, false, scale).unwrap();
    assert!(
        polylines.is_empty(),
        "z far above the octree produces no lines"
    );
}
