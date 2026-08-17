use super::super::{chain_extrusion_paths, reorder_extrusion_paths};
use crate::project_slice::perimeters::classic::materialize::{
    ExtrusionPath, ExtrusionRole, Point3, Polyline3,
};

#[test]
fn task22o8_empty_and_single_path_cases_match_source() {
    assert!(chain_extrusion_paths(&[], Some([0, 0])).is_empty());
    let paths = vec![path(&[(0, 0, 3), (10, 0, 4)], 1.25, 0.4, 0.2)];
    assert_eq!(
        chain_extrusion_paths(&paths, Some([0, 0])),
        vec![(0, false)]
    );
    assert_eq!(
        chain_extrusion_paths(&paths, Some([11, 0])),
        vec![(0, true)]
    );

    let base = 4_000_000_000_000_000_000_i64;
    let large = vec![path(
        &[(base, base, 0), (base + 1, base, 0)],
        1.25,
        0.4,
        0.2,
    )];
    assert_eq!(
        chain_extrusion_paths(&large, Some([base + 2, base])),
        vec![(0, true)]
    );
}

#[test]
fn task22o8_multi_path_chain_moves_reorders_and_reverses_complete_polylines() {
    let mut paths = vec![
        path(&[(0, 0, 1), (2, 0, 2)], 1.0, 0.41, 0.21),
        path(&[(8, 0, 3), (6, 0, 4), (4, 0, 5)], 2.0, 0.42, 0.22),
        path(&[(10, 0, 6), (12, 0, 7)], 3.0, 0.43, 0.23),
    ];
    let chain = chain_extrusion_paths(&paths, Some([0, 0]));
    assert_eq!(chain, vec![(0, false), (1, true), (2, false)]);
    reorder_extrusion_paths(&mut paths, &chain);
    assert_eq!(xyz(&paths[0]), vec![(0, 0, 1), (2, 0, 2)]);
    assert_eq!(xyz(&paths[1]), vec![(4, 0, 5), (6, 0, 4), (8, 0, 3)]);
    assert_eq!(xyz(&paths[2]), vec![(10, 0, 6), (12, 0, 7)]);
    assert_eq!(paths[1].role, ExtrusionRole::Perimeter);
    assert_eq!(
        (paths[1].mm3_per_mm, paths[1].width, paths[1].height),
        (2.0, 0.42, 0.22)
    );
}

#[test]
fn task22o8_large_coordinates_convert_to_f64_before_distance_arithmetic() {
    let base = 4_000_000_000_000_000_000_i64;
    let paths = vec![
        path(&[(base, base, 0), (base + 1_024, base, 0)], 1.0, 1.0, 1.0),
        path(
            &[(base + 3_072, base, 0), (base + 2_048, base, 0)],
            1.0,
            1.0,
            1.0,
        ),
    ];
    assert_eq!(
        chain_extrusion_paths(&paths, Some([base, base])),
        vec![(0, false), (1, true)]
    );
}

#[test]
fn task22o8_arbitrary_chain_is_deterministic() {
    let paths = vec![
        path(&[(4, 7, 0), (9, 1, 0)], 1.0, 1.0, 1.0),
        path(&[(-3, 8, 0), (2, -5, 0)], 1.0, 1.0, 1.0),
        path(&[(12, 3, 0), (6, 11, 0)], 1.0, 1.0, 1.0),
        path(&[(-8, -2, 0), (-1, 4, 0)], 1.0, 1.0, 1.0),
    ];
    let expected = chain_extrusion_paths(&paths, Some([3, 6]));
    for _ in 0..20 {
        assert_eq!(chain_extrusion_paths(&paths, Some([3, 6])), expected);
    }
}

pub(super) fn path(
    points: &[(i64, i64, i64)],
    mm3_per_mm: f64,
    width: f32,
    height: f32,
) -> ExtrusionPath {
    ExtrusionPath {
        polyline: Polyline3 {
            points: points.iter().map(|&(x, y, z)| Point3 { x, y, z }).collect(),
            fitting: Vec::new(),
        },
        role: ExtrusionRole::Perimeter,
        mm3_per_mm,
        width,
        height,
    }
}

fn xyz(path: &ExtrusionPath) -> Vec<(i64, i64, i64)> {
    path.polyline
        .points
        .iter()
        .map(|p| (p.x, p.y, p.z))
        .collect()
}
