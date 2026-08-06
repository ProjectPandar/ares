use crate::geometry::clipper::predicates::area;
use crate::geometry::{Point, Polygon};

use super::{ClipperOffset, EndType, JoinType, OffsetPath};

impl ClipperOffset {
    pub(crate) fn add_closed_path(&mut self, path: &Polygon, join_type: JoinType) {
        self.add_path(path, join_type, EndType::ClosedPolygon);
    }

    pub(crate) fn add_closed_line(&mut self, path: &Polygon, join_type: JoinType) {
        self.add_path(path, join_type, EndType::ClosedLine);
    }

    pub(crate) fn add_open_path(&mut self, path: &Polygon, join_type: JoinType) {
        self.add_path(path, join_type, EndType::OpenButt);
    }

    pub(crate) fn add_open_round_path(&mut self, path: &Polygon, join_type: JoinType) {
        self.add_path(path, join_type, EndType::OpenRound);
    }

    pub(crate) fn add_closed_paths(&mut self, paths: &[Polygon], join_type: JoinType) {
        for path in paths {
            self.add_closed_path(path, join_type);
        }
    }

    fn add_path(&mut self, path: &Polygon, join_type: JoinType, end_type: EndType) {
        let Some(mut high) = path.points().len().checked_sub(1) else {
            return;
        };
        if matches!(end_type, EndType::ClosedPolygon | EndType::ClosedLine) {
            while high > 0 && self.points_are_near(path.points()[high], path.points()[0]) {
                high -= 1;
            }
        }

        let mut contour = Vec::with_capacity(high + 1);
        contour.push(path.points()[0]);
        let mut lowest = 0;
        for &point in &path.points()[1..=high] {
            if self.points_are_near(point, *contour.last().unwrap()) {
                continue;
            }
            contour.push(point);
            let index = contour.len() - 1;
            if is_lower(contour[index], contour[lowest]) {
                lowest = index;
            }
        }
        if end_type == EndType::ClosedPolygon && contour.len() < 3 {
            return;
        }

        let path_index = self.paths.len();
        let replace_lowest = end_type == EndType::ClosedPolygon
            && self.lowest.is_none_or(|(current_path, current_point)| {
                is_lower(
                    contour[lowest],
                    self.paths[current_path].contour.points()[current_point],
                )
            });
        self.paths.push(OffsetPath {
            contour: Polygon::new(contour),
            join_type,
            end_type,
        });
        if replace_lowest {
            self.lowest = Some((path_index, lowest));
        }
    }

    pub(super) fn fix_orientations(&mut self) {
        let reverse_polygons = self
            .lowest
            .is_some_and(|(path_index, _)| area(self.paths[path_index].contour.points()) < 0.0);
        for path in &mut self.paths {
            let positive = area(path.contour.points()) >= 0.0;
            let reverse = if reverse_polygons {
                path.end_type == EndType::ClosedPolygon
                    || (path.end_type == EndType::ClosedLine && positive)
            } else {
                path.end_type == EndType::ClosedLine && !positive
            };
            if reverse {
                path.contour.reverse();
            }
        }
    }

    fn points_are_near(&self, first: Point, second: Point) -> bool {
        if self.shortest_edge_length > 0.0 {
            let dx = (i128::from(first.x()) - i128::from(second.x())) as f64;
            let dy = (i128::from(first.y()) - i128::from(second.y())) as f64;
            dx * dx + dy * dy < self.shortest_edge_length * self.shortest_edge_length
        } else {
            first == second
        }
    }
}

fn is_lower(candidate: Point, current: Point) -> bool {
    candidate.y() > current.y() || (candidate.y() == current.y() && candidate.x() < current.x())
}
