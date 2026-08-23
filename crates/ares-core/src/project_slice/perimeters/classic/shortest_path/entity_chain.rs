use super::chain::{chain_points, chain_segments_constrained};
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
}

pub(in crate::project_slice) fn chain_and_reorder_entities<T: ChainEntity>(
    entities: &mut Vec<T>,
    start_near: Point,
) {
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
        .map(ChainEntity::can_reverse)
        .collect::<Vec<_>>();
    let chain = chain_segments_constrained(&endpoints, coordinates(start_near), &can_reverse);
    let mut source = std::mem::take(entities)
        .into_iter()
        .map(Some)
        .collect::<Vec<_>>();
    entities.reserve(chain.len());
    for (index, reverse) in chain {
        let mut entity = source[index].take().expect("chain indices are unique");
        if reverse {
            entity.reverse();
        }
        entities.push(entity);
    }
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
        let last_index = self.polyline.points().len().saturating_sub(1);
        for fitted in &mut self.fitting {
            let start = fitted.start;
            fitted.start = last_index - fitted.end;
            fitted.end = last_index - start;
            if let Some(arc) = &mut fitted.arc {
                arc.clockwise = !arc.clockwise;
            }
        }
        self.fitting.reverse();
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
            Self::Path(path) => path.first_point(),
            Self::VariableWidth(entity) => entity.first_point(),
        }
    }

    fn last_point(&self) -> Point {
        match self {
            Self::Path(path) => path.last_point(),
            Self::VariableWidth(entity) => entity.last_point(),
        }
    }

    fn can_reverse(&self) -> bool {
        match self {
            Self::Path(_) => true,
            Self::VariableWidth(entity) => entity.can_reverse(),
        }
    }

    fn reverse(&mut self) {
        match self {
            Self::Path(path) => path.reverse(),
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
