use super::chain::{chain_points, chain_segments_constrained_with_optional_start};
use crate::{
    geometry::{Coord, Point, ThickPolyline},
    project_slice::{
        fill_entities::{FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath},
        perimeters::classic::gap_extrusion::GapFillEntity,
    },
};

pub(in crate::project_slice) trait ChainEntity {
    fn first_point(&self) -> Point;
    fn last_point(&self) -> Point;
    fn can_reverse(&self) -> bool;
    fn reverse(&mut self);
    fn is_loop(&self) -> bool {
        false
    }
}

pub(in crate::project_slice) fn chain_and_reorder_entities<T: ChainEntity>(
    entities: &mut Vec<T>,
    start_near: Point,
) {
    let _ = chain_and_reorder_entities_flips(entities, Some(start_near));
}

pub(in crate::project_slice) fn chain_and_reorder_entities_with_optional_start<T: ChainEntity>(
    entities: &mut Vec<T>,
    start_near: Option<Point>,
) {
    let _ = reorder_entities_flips(entities, start_near, false);
}

/// Chains and improves in place, reporting the reversal decision per
/// ORIGINAL entity index (`ShortestPath.cpp:1429–1462`). Upstream
/// `GCode.cpp:8952,9021` applies these to the SHARED layer entities, so the
/// reversals persist into later islands/instances of the same source object.
pub(in crate::project_slice) fn chain_and_reorder_entities_flips<T: ChainEntity>(
    entities: &mut Vec<T>,
    start_near: Option<Point>,
) -> Vec<bool> {
    reorder_entities_flips(entities, start_near, true)
}

fn reorder_entities_flips<T: ChainEntity>(
    entities: &mut Vec<T>,
    start_near: Option<Point>,
    improve: bool,
) -> Vec<bool> {
    let endpoints = entities
        .iter()
        .map(|entity| {
            [
                coordinates(entity.first_point()),
                coordinates(entity.last_point()),
            ]
        })
        .collect::<Vec<_>>();
    let can_reverse = entities
        .iter()
        .map(|entity| entity.is_loop() || entity.can_reverse())
        .collect::<Vec<_>>();
    let mut chain = chain_segments_constrained_with_optional_start(
        &endpoints,
        start_near.map(coordinates),
        &can_reverse,
    );
    if improve {
        let points = endpoints
            .iter()
            .flat_map(|ends| ends.iter().map(|p| [p[0] as f64, p[1] as f64]))
            .collect::<Vec<_>>();
        super::improve::improve(
            &mut chain,
            &points,
            &can_reverse,
            start_near.map(|p| [p.x() as f64, p.y() as f64]),
        );
    }
    let mut source = std::mem::take(entities)
        .into_iter()
        .map(Some)
        .collect::<Vec<_>>();
    let mut flips = vec![false; source.len()];
    entities.reserve(chain.len());
    for (index, reverse) in chain {
        let mut entity = source[index].take().expect("chain indices are unique");
        if reverse && !entity.is_loop() {
            entity.reverse();
            flips[index] = true;
        }
        entities.push(entity);
    }
    flips
}

impl FillExtrusionCollection {
    pub(in crate::project_slice) fn first_point(&self) -> Point {
        self.entities[0].first_point()
    }

    pub(in crate::project_slice) fn last_point(&self) -> Point {
        self.entities
            .last()
            .expect("fill collection is nonempty")
            .last_point()
    }

    pub(in crate::project_slice) fn reverse(&mut self) {
        for entity in &mut self.entities {
            entity.reverse();
        }
        self.entities.reverse();
    }

    pub(in crate::project_slice) fn chained_path_from(mut self, start_near: Point) -> Self {
        if !self.no_sort {
            chain_and_reorder_entities(&mut self.entities, start_near);
        }
        self
    }
}

impl ChainEntity for FillExtrusionCollection {
    fn first_point(&self) -> Point {
        FillExtrusionCollection::first_point(self)
    }

    fn last_point(&self) -> Point {
        FillExtrusionCollection::last_point(self)
    }

    fn can_reverse(&self) -> bool {
        !self.no_sort
    }

    fn reverse(&mut self) {
        FillExtrusionCollection::reverse(self);
    }
}

impl FillExtrusionPath {
    fn first_point(&self) -> Point {
        self.polyline.front().expect("fill path is nonempty")
    }

    fn last_point(&self) -> Point {
        self.polyline.back().expect("fill path is nonempty")
    }

    fn reverse(&mut self) {
        self.polyline.reverse();
    }
}

impl ChainEntity for FillExtrusionPath {
    fn first_point(&self) -> Point {
        FillExtrusionPath::first_point(self)
    }

    fn last_point(&self) -> Point {
        FillExtrusionPath::last_point(self)
    }

    fn can_reverse(&self) -> bool {
        true
    }

    fn reverse(&mut self) {
        FillExtrusionPath::reverse(self);
    }
}

impl ChainEntity for FillExtrusionEntity {
    fn first_point(&self) -> Point {
        match self {
            Self::Path(path) | Self::Oriented(path) => path.first_point(),
            Self::VariableWidth(entity) => entity.first_point(),
        }
    }

    fn last_point(&self) -> Point {
        match self {
            Self::Path(path) | Self::Oriented(path) => path.last_point(),
            Self::VariableWidth(entity) => entity.last_point(),
        }
    }

    fn can_reverse(&self) -> bool {
        match self {
            Self::Path(_) => true,
            Self::Oriented(_) => false,
            Self::VariableWidth(entity) => entity.can_reverse(),
        }
    }

    fn reverse(&mut self) {
        match self {
            Self::Path(path) | Self::Oriented(path) => path.reverse(),
            Self::VariableWidth(entity) => entity.reverse(),
        }
    }
}

impl GapFillEntity {
    pub(in crate::project_slice) fn first_point(&self) -> Point {
        let point = match self {
            Self::Path(path) => path.polyline.points.first().expect("gap path is nonempty"),
            Self::Loop(paths) => paths[0]
                .polyline
                .points
                .first()
                .expect("gap loop is nonempty"),
        };
        Point::new(point.x, point.y)
    }

    pub(in crate::project_slice) fn last_point(&self) -> Point {
        let point = match self {
            Self::Path(path) => path.polyline.points.last().expect("gap path is nonempty"),
            Self::Loop(paths) => paths
                .last()
                .expect("gap loop is nonempty")
                .polyline
                .points
                .last()
                .expect("gap loop is nonempty"),
        };
        Point::new(point.x, point.y)
    }

    pub(in crate::project_slice) fn reverse(&mut self) {
        if let Self::Path(path) = self {
            path.reverse();
        }
    }
}

impl ChainEntity for GapFillEntity {
    fn first_point(&self) -> Point {
        GapFillEntity::first_point(self)
    }

    fn last_point(&self) -> Point {
        GapFillEntity::last_point(self)
    }

    fn can_reverse(&self) -> bool {
        matches!(self, Self::Path(path) if path.can_reverse)
    }

    fn reverse(&mut self) {
        GapFillEntity::reverse(self);
    }
}
impl ChainEntity for ThickPolyline {
    fn first_point(&self) -> Point {
        self.points[0]
    }

    fn last_point(&self) -> Point {
        *self.points.last().unwrap()
    }

    fn can_reverse(&self) -> bool {
        true
    }

    fn reverse(&mut self) {
        ThickPolyline::reverse(self);
    }
}

pub(in crate::project_slice) fn reorder_thick_polylines(polylines: &mut Vec<ThickPolyline>) {
    let points = polylines
        .iter()
        .map(|line| line.points[0])
        .collect::<Vec<_>>();
    let order = chain_points(&points);
    let mut source = std::mem::take(polylines)
        .into_iter()
        .map(Some)
        .collect::<Vec<_>>();
    polylines.reserve(order.len());
    for index in order {
        polylines.push(source[index].take().expect("chain indices are unique"));
    }
}

const fn coordinates(point: Point) -> [Coord; 2] {
    [point.x(), point.y()]
}
