use crate::geometry::clipper::point_in_polygon;
use crate::geometry::{BoundingBox, CoordinateScale, ExPolygon, Point};

#[derive(Clone, Copy)]
struct Item {
    boundary: usize,
    bounds: Bounds,
}

impl Item {
    fn centroid(self, axis: usize) -> i64 {
        let (min, max) = if axis == 0 {
            (self.bounds.min.x(), self.bounds.max.x())
        } else {
            (self.bounds.min.y(), self.bounds.max.y())
        };
        min + max / 2
    }
}

#[derive(Clone, Copy)]
struct Bounds {
    min: Point,
    max: Point,
}

impl Bounds {
    fn from_box(mut bounds: BoundingBox, epsilon: i64) -> Self {
        bounds.offset(epsilon);
        Self {
            min: bounds.min(),
            max: bounds.max(),
        }
    }

    fn union(self, other: Self) -> Self {
        Self {
            min: Point::new(
                self.min.x().min(other.min.x()),
                self.min.y().min(other.min.y()),
            ),
            max: Point::new(
                self.max.x().max(other.max.x()),
                self.max.y().max(other.max.y()),
            ),
        }
    }

    fn contains(self, point: Point) -> bool {
        self.min.x() <= point.x()
            && point.x() <= self.max.x()
            && self.min.y() <= point.y()
            && point.y() <= self.max.y()
    }
}

#[derive(Clone, Copy)]
struct Node {
    bounds: Bounds,
    boundary: Option<usize>,
}

pub(super) struct BoundaryAabb<'a> {
    boundaries: &'a [ExPolygon],
    nodes: Vec<Option<Node>>,
}

fn scaled_epsilon(scale: CoordinateScale) -> i64 {
    match scale {
        CoordinateScale::Normal => 100,
        CoordinateScale::LargeBed => 10,
    }
}

impl<'a> BoundaryAabb<'a> {
    pub(super) fn build(boundaries: &'a [ExPolygon], scale: CoordinateScale) -> Self {
        let epsilon = scaled_epsilon(scale);
        let mut items = boundaries
            .iter()
            .enumerate()
            .map(|(boundary, expolygon)| Item {
                boundary,
                bounds: leaf_bounds(expolygon, epsilon),
            })
            .collect::<Vec<_>>();
        let len = 2 * items.len().next_power_of_two() - 1;
        let mut tree = Self {
            boundaries,
            nodes: vec![None; len],
        };
        let right = items.len() - 1;
        tree.build_recursive(&mut items, 0, 0, right);
        tree
    }

    pub(super) fn sample(&self, point: Point) -> Option<usize> {
        self.sample_node(0, point)
    }

    fn sample_node(&self, node: usize, point: Point) -> Option<usize> {
        let record = self.nodes[node].expect("built AABB nodes are valid");
        if !record.bounds.contains(point) {
            return None;
        }
        if let Some(boundary) = record.boundary {
            return contains(&self.boundaries[boundary], point).then_some(boundary);
        }
        self.sample_node(node * 2 + 1, point)
            .or_else(|| self.sample_node(node * 2 + 2, point))
    }

    fn build_recursive(&mut self, items: &mut [Item], node: usize, left: usize, right: usize) {
        if left == right {
            self.nodes[node] = Some(Node {
                bounds: items[left].bounds,
                boundary: Some(items[left].boundary),
            });
            return;
        }
        let mut bounds = items[left].bounds;
        for item in &items[left + 1..=right] {
            bounds = bounds.union(item.bounds);
        }
        let axis = longest_axis(bounds);
        let center = (left + right) / 2;
        partition(items, axis, left, right, center);
        self.nodes[node] = Some(Node {
            bounds,
            boundary: None,
        });
        self.build_recursive(items, node * 2 + 1, left, center);
        self.build_recursive(items, node * 2 + 2, center + 1, right);
    }
}

fn longest_axis(bounds: Bounds) -> usize {
    let dx = bounds.max.x() - bounds.min.x();
    let dy = bounds.max.y() - bounds.min.y();
    usize::from(dy > dx)
}

fn partition(items: &mut [Item], axis: usize, mut left: usize, mut right: usize, k: usize) {
    while left < right {
        let center = (left + right) / 2;
        let mut left_value = items[left].centroid(axis);
        let mut center_value = items[center].centroid(axis);
        let mut right_value = items[right].centroid(axis);
        if left_value > center_value {
            items.swap(left, center);
            std::mem::swap(&mut left_value, &mut center_value);
        }
        if left_value > right_value {
            items.swap(left, right);
            right_value = left_value;
        }
        if center_value > right_value {
            items.swap(center, right);
            center_value = right_value;
        }
        let pivot = center_value;
        if right <= left + 2 {
            break;
        }
        let mut i = left;
        let mut j = right - 1;
        items.swap(center, j);
        loop {
            i += 1;
            while items[i].centroid(axis) < pivot {
                i += 1;
            }
            j -= 1;
            while items[j].centroid(axis) > pivot && i < j {
                j -= 1;
            }
            if i >= j {
                break;
            }
            items.swap(i, j);
        }
        items.swap(i, right - 1);
        if k < i {
            right = i - 1;
        } else if k == i {
            break;
        } else {
            left = i + 1;
        }
    }
}

fn leaf_bounds(expolygon: &ExPolygon, epsilon: i64) -> Bounds {
    Bounds::from_box(
        BoundingBox::from_polygon(expolygon.contour())
            .expect("a boundary ExPolygon contour must be nonempty"),
        epsilon,
    )
}

pub(in crate::geometry::region_expansion) fn sample_in_expolygons(
    expolygons: &[ExPolygon],
    point: Point,
    scale: CoordinateScale,
) -> Option<usize> {
    BoundaryAabb::build(expolygons, scale).sample(point)
}

fn contains(expolygon: &ExPolygon, point: Point) -> bool {
    point_in_polygon(point, expolygon.contour().points()) != 0
        && !expolygon
            .holes()
            .iter()
            .any(|hole| point_in_polygon(point, hole.points()) > 0)
}

#[cfg(test)]
pub(in crate::geometry) fn bbox_contains_for_test(
    boundary: &ExPolygon,
    point: Point,
    scale: CoordinateScale,
) -> bool {
    let epsilon = scaled_epsilon(scale);
    leaf_bounds(boundary, epsilon).contains(point)
}

#[cfg(test)]
pub(in crate::geometry) fn sample_for_test(
    boundaries: &[ExPolygon],
    point: Point,
    scale: CoordinateScale,
) -> Option<usize> {
    BoundaryAabb::build(boundaries, scale).sample(point)
}

#[cfg(test)]
pub(in crate::geometry) fn centroid_for_test(min: Point, max: Point, axis: usize) -> i64 {
    Item {
        boundary: 0,
        bounds: Bounds { min, max },
    }
    .centroid(axis)
}

#[cfg(test)]
pub(in crate::geometry) fn partition_for_test(
    bounds: &[(Point, Point)],
    axis: usize,
    k: usize,
) -> Vec<usize> {
    let mut items = bounds
        .iter()
        .enumerate()
        .map(|(boundary, &(min, max))| Item {
            boundary,
            bounds: Bounds { min, max },
        })
        .collect::<Vec<_>>();
    let right = items.len() - 1;
    partition(&mut items, axis, 0, right, k);
    items.into_iter().map(|item| item.boundary).collect()
}

#[cfg(test)]
pub(in crate::geometry) fn longest_axis_for_test(min: Point, max: Point) -> usize {
    longest_axis(Bounds { min, max })
}
