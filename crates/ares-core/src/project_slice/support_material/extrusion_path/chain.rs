//! `ExtrusionEntity.hpp` path, multi-path and no-sort collection chaining.

use super::{SupportExtrusionEntity, SupportExtrusionPath};
use crate::{
    geometry::Point,
    project_slice::perimeters::classic::{materialize::Polyline3, shortest_path::ChainEntity},
};

impl ChainEntity for SupportExtrusionPath {
    fn first_point(&self) -> Point {
        let point = self.polyline.points[0];
        Point::new(point.x, point.y)
    }

    fn last_point(&self) -> Point {
        let point = self
            .polyline
            .points
            .last()
            .expect("support path is nonempty");
        Point::new(point.x, point.y)
    }

    fn can_reverse(&self) -> bool {
        self.can_reverse
    }

    fn reverse(&mut self) {
        reverse_polyline(&mut self.polyline);
    }
}

pub(super) fn reverse_polyline(polyline: &mut Polyline3) {
    let last = polyline.points.len() - 1;
    for fitted in &mut polyline.fitting {
        let start = fitted.start;
        fitted.start = last - fitted.end;
        fitted.end = last - start;
        if let Some(arc) = &mut fitted.arc {
            arc.clockwise = !arc.clockwise;
        }
    }
    polyline.fitting.reverse();
    polyline.points.reverse();
}

impl ChainEntity for SupportExtrusionEntity {
    fn is_loop(&self) -> bool {
        matches!(self, Self::Loop(_))
    }

    fn first_point(&self) -> Point {
        match self {
            Self::Path(path) | Self::Loop(path) => path.first_point(),
            Self::NoSort(entities) => entities[0].first_point(),
            Self::MultiPath(paths) => paths[0].first_point(),
        }
    }

    fn last_point(&self) -> Point {
        match self {
            Self::Path(path) => path.last_point(),
            Self::Loop(path) => path.first_point(),
            Self::NoSort(entities) => entities
                .last()
                .expect("support group is nonempty")
                .last_point(),
            Self::MultiPath(paths) => paths
                .last()
                .expect("support group is nonempty")
                .last_point(),
        }
    }

    fn can_reverse(&self) -> bool {
        match self {
            Self::Path(path) => path.can_reverse(),
            Self::NoSort(_) | Self::Loop(_) => false,
            Self::MultiPath(_) => true,
        }
    }

    fn reverse(&mut self) {
        match self {
            Self::Path(path) | Self::Loop(path) => path.reverse(),
            Self::NoSort(entities) => {
                for entity in &mut *entities {
                    if !matches!(entity, Self::Loop(_)) {
                        entity.reverse();
                    }
                }
                entities.reverse();
            }
            Self::MultiPath(paths) => {
                for path in &mut *paths {
                    path.reverse();
                }
                paths.reverse();
            }
        }
    }
}
