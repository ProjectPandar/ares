//! The exact model intersection (`ModelInstance::intersects_bed_exclude_region`), ported from
//! `test_exclusion_volumes.cpp:542-601` through `bed_exclusion::model_part_intersects_region`. The
//! cases are GUI-only upstream, so these unit tests are the oracle. The red preview asserts (the
//! `intersection_mesh` output) are left out with the preview itself.

use ares_core::bed_exclusion::{BedExcludeRegion, Purpose, model_part_intersects_region};
use ares_core::{Point3d, Transform3d};

use super::rectangle;

/// `region` (`:42-47`): a collision volume over a rectangle with a z range.
fn region([min_x, min_y, max_x, max_y]: [f64; 4], [z_min, z_max]: [f64; 2]) -> BedExcludeRegion {
    BedExcludeRegion {
        polygon: rectangle(min_x, min_y, max_x, max_y),
        z_min,
        z_max,
        purpose: Purpose::CollisionVolume,
        has_z_range: true,
    }
}

/// An affine transform of a row-major linear part and an offset.
fn affine(linear: [[f64; 3]; 3], [x, y, z]: [f64; 3]) -> Transform3d {
    let values = [
        linear[0][0],
        linear[1][0],
        linear[2][0],
        linear[0][1],
        linear[1][1],
        linear[2][1],
        linear[0][2],
        linear[1][2],
        linear[2][2],
        x,
        y,
        z,
    ];
    let text = values.map(|value| format!("{value:?}")).join(" ");
    Transform3d::parse_3mf(&text).unwrap()
}

const IDENTITY: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// `CubeModel` (`:80-92`): `make_cube(size)` (`its_make_cube`, `TriangleMesh.cpp:902-912`) added as
/// a model part. `add_volume` centres it (`Model.hpp:445`, `center_geometry_after_creation`), so
/// the part keeps the cube about its centre with the volume offset there.
struct CubeModel {
    vertices: Vec<Point3d>,
    indices: Vec<[u32; 3]>,
    volume: Transform3d,
    instance: Transform3d,
}

impl CubeModel {
    fn new(offset: [f64; 3], [x, y, z]: [f64; 3]) -> Self {
        let (x, y, z) = (
            f64::from(x as f32),
            f64::from(y as f32),
            f64::from(z as f32),
        );
        let corners = [
            [x, y, 0.0],
            [x, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, y, 0.0],
            [x, y, z],
            [0.0, y, z],
            [0.0, 0.0, z],
            [x, 0.0, z],
        ];
        let centre = [0.5 * x, 0.5 * y, 0.5 * z];
        Self {
            vertices: corners
                .iter()
                .map(|p| Point3d::new(p[0] - centre[0], p[1] - centre[1], p[2] - centre[2]))
                .collect(),
            indices: vec![
                [0, 1, 2],
                [0, 2, 3],
                [4, 5, 6],
                [4, 6, 7],
                [0, 4, 7],
                [0, 7, 1],
                [1, 7, 6],
                [1, 6, 2],
                [2, 6, 5],
                [2, 5, 3],
                [4, 0, 3],
                [4, 3, 5],
            ],
            volume: affine(IDENTITY, centre),
            instance: affine(IDENTITY, offset),
        }
    }

    fn intersects(&self, region: &BedExcludeRegion) -> bool {
        let matrix = self.instance.then(self.volume);
        model_part_intersects_region(&self.vertices, &self.indices, matrix, region)
    }
}

#[test]
fn exact_model_intersection_respects_xy_and_z() {
    // :542-562
    let cube = CubeModel::new([30.0, 40.0, 0.0], [20.0, 20.0, 20.0]);
    assert!(cube.intersects(&region([45.0, 45.0, 55.0, 55.0], [5.0, 15.0])));
    assert!(!cube.intersects(&region([45.0, 45.0, 55.0, 55.0], [25.0, 30.0])));
    assert!(!cube.intersects(&region([60.0, 70.0, 70.0, 80.0], [0.0, 20.0])));
}

#[test]
fn exact_model_intersection_follows_instance_and_volume_transforms() {
    // :564-572: the instance turned π/4 about Z (`set_rotation`), then scaled (1.5, 0.5, 1)
    // (`set_scaling_factor`): offset · R · S.
    let mut cube = CubeModel::new([50.0, 50.0, 0.0], [20.0, 20.0, 20.0]);
    let (sin, cos) = std::f64::consts::FRAC_PI_4.sin_cos();
    let rotation_scale = [
        [cos * 1.5, -sin * 0.5, 0.0],
        [sin * 1.5, cos * 0.5, 0.0],
        [0.0, 0.0, 1.0],
    ];
    cube.instance = affine(rotation_scale, [50.0, 50.0, 0.0]);
    assert!(cube.intersects(&region([50.0, 50.0, 55.0, 55.0], [0.0, 20.0])));
    assert!(!cube.intersects(&region([100.0, 100.0, 110.0, 110.0], [0.0, 20.0])));
}

#[test]
fn exact_model_intersection_detects_a_prism_enclosed_by_a_solid() {
    // :574-601, without the preview and the multi-region wrapper.
    let block = CubeModel::new([200.0, 300.0, 0.0], [100.0, 100.0, 60.0]);
    assert!(block.intersects(&region([230.0, 330.0, 260.0, 358.0], [8.0, 22.0])));
    assert!(!block.intersects(&region([230.0, 330.0, 260.0, 358.0], [65.0, 75.0])));
    assert!(block.intersects(&region([190.0, 290.0, 310.0, 410.0], [-5.0, 65.0])));
}
