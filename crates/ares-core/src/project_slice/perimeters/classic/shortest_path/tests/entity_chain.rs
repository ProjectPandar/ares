use crate::{
    ExtrusionRole,
    geometry::{Point, Polyline},
    project_slice::{
        fill_entities::{FillExtrusionCollection, FillExtrusionPath},
        perimeters::classic::{
            gap_extrusion::GapFillEntity,
            materialize::{ExtrusionPath, ExtrusionRole as PerimeterRole, Point3, Polyline3},
            shortest_path::{ChainEntity, chain_and_reorder_entities},
        },
    },
};

#[test]
fn task22o96_constrained_chain_never_starts_by_reversing_no_sort_collection() {
    let mut entities = vec![
        collection(&[(0, 100)], true),
        collection(&[(110, 120)], false),
    ];

    chain_and_reorder_entities(&mut entities, Point::new(99, 0));

    assert_eq!(endpoints(&entities), vec![(110, 120), (0, 100)]);
}

#[test]
fn task22o96_reversible_collection_and_gap_path_reverse_from_explicit_cursor() {
    let mut fills = vec![collection(&[(0, 100)], false)];
    chain_and_reorder_entities(&mut fills, Point::new(99, 0));
    assert_eq!(endpoints(&fills), vec![(100, 0)]);

    let mut gaps = vec![gap_path(0, 100)];
    chain_and_reorder_entities(&mut gaps, Point::new(99, 0));
    assert_eq!(gaps[0].first_point(), Point::new(100, 0));
    assert_eq!(gaps[0].last_point(), Point::new(0, 0));
}

#[test]
fn task22o96_chained_path_from_respects_no_sort_and_reorders_sortable_paths() {
    let fixed = collection(&[(0, 10), (100, 110)], true).chained_path_from(Point::new(109, 0));
    assert_eq!(path_endpoints(&fixed), vec![(0, 10), (100, 110)]);

    let sortable = collection(&[(0, 10), (100, 110)], false).chained_path_from(Point::new(109, 0));
    assert_eq!(path_endpoints(&sortable), vec![(110, 100), (10, 0)]);
}

#[test]
fn task22o96_gap_loop_reports_closed_endpoints_and_never_reverses() {
    let mut loop_entity = GapFillEntity::Loop(vec![gap_extrusion_path(5, 5)]);
    assert_eq!(loop_entity.first_point(), Point::new(5, 0));
    assert_eq!(loop_entity.last_point(), Point::new(5, 0));
    assert!(ChainEntity::can_reverse(&loop_entity));

    loop_entity.reverse();

    assert_eq!(loop_entity.first_point(), Point::new(5, 0));
    assert_eq!(loop_entity.last_point(), Point::new(5, 0));
}

fn collection(endpoints: &[(i64, i64)], no_sort: bool) -> FillExtrusionCollection {
    FillExtrusionCollection {
        paths: endpoints
            .iter()
            .map(|&(first, last)| FillExtrusionPath {
                polyline: Polyline::new(vec![Point::new(first, 0), Point::new(last, 0)]),
                role: ExtrusionRole::InternalInfill,
                mm3_per_mm: 1.0,
                width: 1.0,
                height: 1.0,
            })
            .collect(),
        no_sort,
    }
}

fn endpoints(collections: &[FillExtrusionCollection]) -> Vec<(i64, i64)> {
    collections
        .iter()
        .map(|collection| (collection.first_point().x(), collection.last_point().x()))
        .collect()
}

fn path_endpoints(collection: &FillExtrusionCollection) -> Vec<(i64, i64)> {
    collection
        .paths
        .iter()
        .map(|path| {
            (
                path.polyline.front().unwrap().x(),
                path.polyline.back().unwrap().x(),
            )
        })
        .collect()
}

fn gap_path(first: i64, last: i64) -> GapFillEntity {
    GapFillEntity::Path(gap_extrusion_path(first, last))
}

fn gap_extrusion_path(first: i64, last: i64) -> ExtrusionPath {
    ExtrusionPath {
        polyline: Polyline3 {
            points: vec![
                Point3 {
                    x: first,
                    y: 0,
                    z: 0,
                },
                Point3 {
                    x: last,
                    y: 0,
                    z: 0,
                },
            ],
        },
        role: PerimeterRole::GapFill,
        mm3_per_mm: 1.0,
        width: 1.0,
        height: 1.0,
    }
}
