use super::{simplify_extrusion_path, simplify_fill_path};
use crate::geometry::{Point, Polyline};
use crate::project_slice::fill_entities::FillExtrusionPath;
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

#[test]
fn reversed_source_orientation_preserves_direction_sensitive_dp_vertex() {
    let points = [(15, 3), (12, 6), (9, 2), (6, 3), (3, 7), (0, 3)].map(|(x, y)| Point::new(x, y));
    let path = || FillExtrusionPath {
        polyline: Polyline::new(points.to_vec()),
        fitting: Vec::new(),
        role: crate::ExtrusionRole::BottomSurface,
        mm3_per_mm: 0.08,
        width: 0.4,
        height: 0.2,
    };
    let mut forward = path();
    let mut reversed = path();

    simplify_fill_path(&mut forward, 2.0, false);
    simplify_fill_path(&mut reversed, 2.0, true);

    assert_eq!(forward.polyline.points().len(), 5);
    assert_eq!(reversed.polyline.points().len(), 6);
    assert_eq!(reversed.polyline.points(), points);
}

const fn point(x: i64, y: i64) -> Point3 {
    Point3 { x, y, z: 0 }
}
