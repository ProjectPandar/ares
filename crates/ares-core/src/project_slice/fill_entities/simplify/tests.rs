use super::simplify_extrusion_path;
use crate::project_slice::perimeters::classic::materialize::{
    ExtrusionPath, ExtrusionRole, Point3, Polyline3,
};

#[test]
fn post_clip_corner_removes_sub_resolution_penultimate_point() {
    let mut path = ExtrusionPath {
        polyline: Polyline3 {
            points: vec![
                point(-3_903_983, 3_886_298),
                point(-3_976_475, 3_928_151),
                point(-3_903_983, 3_970_004),
                point(-3_899_819, 3_962_791),
            ],
            fitting: Vec::new(),
        },
        role: ExtrusionRole::SolidInfill,
        can_reverse: true,
        mm3_per_mm: 0.08,
        width: 0.4,
        height: 0.2,
    };

    simplify_extrusion_path(&mut path, 12_000.0);

    assert_eq!(
        path.polyline.points,
        [
            point(-3_903_983, 3_886_298),
            point(-3_976_475, 3_928_151),
            point(-3_899_819, 3_962_791),
        ]
    );
}

const fn point(x: i64, y: i64) -> Point3 {
    Point3 { x, y, z: 0 }
}
