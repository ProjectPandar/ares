use super::{IslandInfillEntity, assign_layer, contour_bounds, island_index};
use crate::{
    ExtrusionRole,
    geometry::{ExPolygon, Point, Polygon, Polyline},
    project_slice::fill_entities::{
        FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
    },
};

fn rectangle(minimum: i64, maximum: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(minimum, minimum),
            Point::new(maximum, minimum),
            Point::new(maximum, maximum),
            Point::new(minimum, maximum),
        ]),
        Vec::new(),
    )
}

fn path(start: Point, end: Point) -> FillExtrusionEntity {
    FillExtrusionEntity::Path(FillExtrusionPath {
        polyline: Polyline::new(vec![start, end]),
        fitting: Vec::new(),
        role: ExtrusionRole::SolidInfill,
        mm3_per_mm: 0.04,
        width: 0.4,
        height: 0.2,
    })
}

#[test]
fn task22o210_maximum_boundary_is_excluded_before_contour_test() {
    let slices = [rectangle(0, 10), rectangle(-10, 20)];
    let bounds = slices.iter().map(contour_bounds).collect::<Vec<_>>();

    assert_eq!(
        island_index(Point::new(10, 5), &slices, &bounds, &[0, 1]),
        1
    );
}

#[test]
fn sortable_fill_collections_flatten_before_island_print_ordering() {
    let mut layer = LayerFillEntities {
        collections: vec![
            FillExtrusionCollection {
                entities: vec![
                    path(Point::new(-8, -8), Point::new(-4, -8)),
                    path(Point::new(4, 8), Point::new(8, 8)),
                ],
                no_sort: false,
            },
            FillExtrusionCollection {
                entities: vec![path(Point::new(-2, 0), Point::new(2, 0))],
                no_sort: true,
            },
        ],
        ..LayerFillEntities::default()
    };

    let assigned = assign_layer(&mut layer, &[rectangle(-10, 10)]);
    let infills = &assigned.islands[0].infills;

    assert_eq!(infills.len(), 3);
    assert_eq!(
        infills
            .iter()
            .map(|infill| match infill {
                IslandInfillEntity::Fill(FillExtrusionEntity::Path(path)) => {
                    path.polyline.front()
                }
                IslandInfillEntity::Fill(FillExtrusionEntity::VariableWidth(_))
                | IslandInfillEntity::FillCollection(_)
                | IslandInfillEntity::Thin(_) => None,
            })
            .collect::<Vec<_>>(),
        vec![Some(Point::new(-8, -8)), Some(Point::new(4, 8)), None]
    );
    assert!(matches!(
        &infills[2],
        IslandInfillEntity::FillCollection(collection)
            if collection.no_sort && collection.entities.len() == 1
    ));
}
