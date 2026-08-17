use crate::{
    geometry::CoordinateScale,
    project_slice::perimeters::classic::{
        gap_extrusion::{GapFillCollection, GapFillEntity, coverage},
        materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
    },
};

fn path(y: i64) -> ExtrusionPath {
    ExtrusionPath {
        polyline: Polyline3 {
            points: vec![
                Point3 { x: 0, y, z: 0 },
                Point3 {
                    x: 1_000_000,
                    y,
                    z: 0,
                },
            ],
            fitting: Vec::new(),
            candidate_points: Vec::new(),
        },
        role: ExtrusionRole::GapFill,
        mm3_per_mm: 0.08,
        width: 0.4,
        height: 0.2,
    }
}

#[test]
fn task22o14_coverage_delegates_paths_loops_and_entities_in_order() {
    let collection = GapFillCollection {
        entities: vec![
            GapFillEntity::Path(path(0)),
            GapFillEntity::Loop(vec![path(1_000_000), path(2_000_000)]),
        ],
    };
    let covered = coverage::covered_polygons(&collection, CoordinateScale::Normal).unwrap();
    assert_eq!(covered.len(), 3);
    assert_eq!(
        covered[0]
            .points()
            .iter()
            .map(|p| (p.x(), p.y()))
            .collect::<Vec<_>>(),
        vec![
            (1_000_000, 200_010),
            (0, 200_010),
            (0, -200_010),
            (1_000_000, -200_010)
        ]
    );
}

#[test]
fn task22o14_coverage_uses_scale_specific_float_delta() {
    let collection = GapFillCollection {
        entities: vec![GapFillEntity::Path(path(0))],
    };
    let normal = coverage::covered_polygons(&collection, CoordinateScale::Normal).unwrap();
    let large = coverage::covered_polygons(&collection, CoordinateScale::LargeBed).unwrap();
    assert_eq!(normal[0].points()[0].y().abs(), 200_010);
    assert_eq!(large[0].points()[0].y().abs(), 20_010);
}
