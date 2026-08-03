use crate::geometry::{Point, Polygon};

use super::{ClipperOffset, EndType, JoinType, OffsetPath};
use crate::geometry::clipper::predicates::area;

impl ClipperOffset {
    pub(crate) fn add_closed_path(&mut self, path: &Polygon, join_type: JoinType) {
        let Some(mut high) = path.points().len().checked_sub(1) else {
            return;
        };
        while high > 0 && self.points_are_near(path.points()[high], path.points()[0]) {
            high -= 1;
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
        if contour.len() < 3 {
            return;
        }

        let candidate = contour[lowest];
        let replace_lowest = self.lowest.is_none_or(|(path_index, point_index)| {
            is_lower(
                candidate,
                self.paths[path_index].contour.points()[point_index],
            )
        });
        let path_index = self.paths.len();
        self.paths.push(OffsetPath {
            contour: Polygon::new(contour),
            join_type,
            end_type: EndType::ClosedPolygon,
        });
        if replace_lowest {
            self.lowest = Some((path_index, lowest));
        }
    }

    pub(crate) fn add_open_path(&mut self, path: &Polygon, join_type: JoinType) {
        let Some((&first, rest)) = path.points().split_first() else {
            return;
        };
        let mut contour = Vec::with_capacity(path.points().len());
        contour.push(first);
        for &point in rest {
            if !self.points_are_near(point, *contour.last().unwrap()) {
                contour.push(point);
            }
        }
        self.paths.push(OffsetPath {
            contour: Polygon::new(contour),
            join_type,
            end_type: EndType::OpenButt,
        });
    }

    pub(crate) fn add_closed_paths(&mut self, paths: &[Polygon], join_type: JoinType) {
        for path in paths {
            self.add_closed_path(path, join_type);
        }
    }

    pub(super) fn fix_orientations(&mut self) {
        let Some((path_index, _)) = self.lowest else {
            return;
        };
        if area(self.paths[path_index].contour.points()) < 0.0 {
            self.reverse_closed_paths();
        }
    }

    fn reverse_closed_paths(&mut self) {
        for path in &mut self.paths {
            if path.end_type == EndType::ClosedPolygon {
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
