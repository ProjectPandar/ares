//! Nest closed Arachne walls and place the open walls at each center end.
use super::chain::{self, Ring, contains, distance_squared};
use crate::{
    ProcessSurfaceFillOrder,
    geometry::{ClipperError, ExPolygon, Point, Polygon, ThickPolyline},
};

impl Ring for ThickPolyline {
    type Path = Self;
    fn open(&self, start: usize) -> Self {
        let mut path = self.clone();
        if path.points.first() != path.points.last() {
            path.points.push(path.points[0]);
            path.width.push(*path.width.last().unwrap());
            path.width.push(path.width[0]);
        }
        path.start_at_index(start);
        path
    }
    fn points(path: &Self) -> &[Point] {
        &path.points
    }
    fn trim(path: &mut Self, mut distance: f64) {
        while distance > 0.0 && path.points.len() >= 2 {
            let last = path.points.pop().unwrap();
            let end = path.width.pop().unwrap();
            let start = path.width.pop().unwrap();
            let previous = *path.points.last().unwrap();
            let len = distance_squared(last, previous).sqrt();
            if len > distance {
                let t = distance / len;
                path.points.push(Point::new(
                    (last.x() as f64 + (previous.x() - last.x()) as f64 * t) as i64,
                    (last.y() as f64 + (previous.y() - last.y()) as f64 * t) as i64,
                ));
                path.width
                    .extend([start, start + (end - start) * (1.0 - t)]);
                return;
            }
            distance -= len;
        }
        path.clear();
    }
    fn append(path: &mut Self, next: Self) {
        if path.points.last() == next.points.first() {
            path.points.extend(next.points.into_iter().skip(1));
            path.width.extend(next.width);
        } else {
            path.width
                .extend([*path.width.last().unwrap(), next.width[0]]);
            path.points.extend(next.points);
            path.width.extend(next.width);
        }
    }
    fn reverse(path: &mut Self) {
        path.reverse();
    }
    fn center_stub(_: &mut Self, _: &Polygon, _: f64) -> Result<(), ClipperError> {
        Ok(())
    }
}

pub(crate) fn spiral_walls(
    walls: Vec<(ThickPolyline, bool)>,
    spacing: i64,
    order: ProcessSurfaceFillOrder,
    domain: &ExPolygon,
) -> Result<Vec<ThickPolyline>, ClipperError> {
    let mut closed = Vec::new();
    let mut outlines = Vec::new();
    let mut open = Vec::new();
    for (wall, is_closed) in walls {
        if is_closed {
            outlines.push(Polygon::new(wall.points.clone()));
            closed.push(wall);
        } else {
            open.push(wall);
        }
    }
    let inside = closed
        .iter()
        .enumerate()
        .map(|(i, wall)| {
            outlines
                .iter()
                .enumerate()
                .filter_map(|(j, ring)| (i != j && contains(ring, wall.points[0])).then_some(j))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut children = vec![Vec::new(); closed.len() + 1];
    let root = closed.len();
    for (i, containers) in inside.iter().enumerate() {
        // Strictly greater depth wins; ties retain the first parent.
        let parent = containers
            .iter()
            .copied()
            .rev()
            .max_by_key(|&j| inside[j].len())
            .unwrap_or(root);
        children[parent].push(i);
    }
    let mut stack = children[root].iter().rev().copied().collect::<Vec<_>>();
    let mut ordered = Vec::new();
    let mut ordered_outlines = Vec::new();
    while let Some(i) = stack.pop() {
        ordered.push(std::mem::take(&mut closed[i]));
        ordered_outlines.push(std::mem::replace(
            &mut outlines[i],
            Polygon::new(Vec::new()),
        ));
        stack.extend(children[i].iter().rev().copied());
    }
    let spirals = chain::generate(&ordered, &ordered_outlines, spacing, order, Some(domain))?;
    if spirals.is_empty() {
        return Ok(open);
    }
    let outward = order != ProcessSurfaceFillOrder::Inward;
    let center = |path: &ThickPolyline| {
        if outward {
            path.points[0]
        } else {
            *path.points.last().unwrap()
        }
    };
    let mut beside = vec![Vec::new(); spirals.len()];
    for wall in open {
        let near = |spiral: &ThickPolyline| {
            let p = center(spiral);
            distance_squared(p, wall.points[0])
                .min(distance_squared(p, *wall.points.last().unwrap()))
        };
        let i = spirals
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| near(a).total_cmp(&near(b)))
            .unwrap()
            .0;
        beside[i].push(wall);
    }
    let mut output = Vec::new();
    for (mut walls, spiral) in beside.into_iter().zip(spirals) {
        chain_open_walls(&mut walls, center(&spiral));
        if outward {
            walls.reverse();
            for wall in &mut walls {
                wall.reverse();
            }
            output.extend(walls);
            output.push(spiral);
        } else {
            output.push(spiral);
            output.extend(walls);
        }
    }
    Ok(output)
}

fn chain_open_walls(walls: &mut [ThickPolyline], mut pos: Point) {
    for i in 0..walls.len() {
        let endpoints = walls.iter().enumerate().skip(i).flat_map(|(j, wall)| {
            [
                (j, wall.points[0], false),
                (j, *wall.points.last().unwrap(), true),
            ]
        });
        let (next, _, flip) = endpoints
            .min_by(|(_, a, _), (_, b, _)| {
                distance_squared(pos, *a).total_cmp(&distance_squared(pos, *b))
            })
            .unwrap();
        walls.swap(i, next);
        if flip {
            walls[i].reverse();
        }
        pos = *walls[i].points.last().unwrap();
    }
}
