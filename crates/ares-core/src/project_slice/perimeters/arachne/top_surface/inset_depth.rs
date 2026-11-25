//! `PerimeterGenerator.cpp:2357–2470`: depth from the real outer walls.

use crate::{
    arachne::{ExtrusionJunction, ExtrusionLine},
    geometry::{BoundingBox, Point},
};

pub(super) fn renumber_inner_walls_by_depth(
    outer: &[ExtrusionLine],
    inner: &mut Vec<Vec<ExtrusionLine>>,
    (outer_threshold, inner_threshold): (f64, f64),
) {
    let lines = std::mem::take(inner)
        .into_iter()
        .flatten()
        .filter(|line| !line.junctions.is_empty())
        .collect::<Vec<_>>();
    let bounds = lines.iter().map(grown_bounds).collect::<Vec<_>>();
    let touches_inner = |a: usize, b: usize| {
        bounds[a].overlap(bounds[b]) && lines_touch(&lines[a], &lines[b], inner_threshold)
    };
    let mut depth = vec![0; lines.len()];
    let mut frontier = Vec::new();
    for wall in outer {
        let outer_bounds = grown_bounds(wall);
        for (index, line) in lines.iter().enumerate() {
            if depth[index] == 0
                && line.inset_index == 1
                && outer_bounds.overlap(bounds[index])
                && lines_touch(wall, line, outer_threshold)
            {
                depth[index] = 1;
                frontier.push(index);
            }
        }
    }

    // Preserve depths already chained from the outer wall by original inset.
    let mut numbered = frontier.clone();
    let mut level = 2;
    while !frontier.is_empty() {
        let mut next = Vec::new();
        for (index, line) in lines.iter().enumerate() {
            if depth[index] == 0
                && line.inset_index == level
                && frontier.iter().any(|&other| touches_inner(other, index))
            {
                depth[index] = level;
                next.push(index);
            }
        }
        numbered.extend_from_slice(&next);
        frontier = next;
        level += 1;
    }

    // Assign other walls by contact with the previous frontier only.
    frontier = numbered;
    while !frontier.is_empty() {
        let mut next = Vec::new();
        for index in 0..lines.len() {
            if depth[index] != 0 {
                continue;
            }
            let deepest = deepest_contact(&frontier, &depth, |other| touches_inner(other, index));
            if deepest > 0 {
                depth[index] = deepest + 1;
                next.push(index);
            }
        }
        frontier = next;
    }

    for (mut line, depth) in lines.into_iter().zip(depth) {
        if depth > 0 {
            line.inset_index = depth;
        }
        if inner.len() < line.inset_index {
            inner.resize_with(line.inset_index, Vec::new);
        }
        inner[line.inset_index - 1].push(line);
    }
}

fn deepest_contact(frontier: &[usize], depth: &[usize], touches: impl Fn(usize) -> bool) -> usize {
    let mut deepest = 0;
    for &other in frontier {
        if depth[other] > deepest && touches(other) {
            deepest = depth[other];
        }
    }
    deepest
}

fn grown_bounds(line: &ExtrusionLine) -> BoundingBox {
    let widest = line
        .junctions
        .iter()
        .map(|junction| junction.width)
        .max()
        .unwrap();
    BoundingBox::from_points(line.junctions.iter().map(|junction| junction.point))
        .unwrap()
        .inflated(widest)
}

fn lines_touch(a: &ExtrusionLine, b: &ExtrusionLine, threshold: f64) -> bool {
    let one_way = |from: &ExtrusionLine, to: &ExtrusionLine| {
        from.junctions.iter().any(|junction| {
            let point = coords(junction.point);
            to.junctions.windows(2).any(|pair| {
                touches_segment((point, junction.width), (&pair[0], &pair[1]), threshold)
            })
        })
    };
    one_way(a, b) || one_way(b, a)
}

fn touches_segment(
    ([px, py], point_width): ([f64; 2], i64),
    (start, end): (&ExtrusionJunction, &ExtrusionJunction),
    threshold: f64,
) -> bool {
    let [x0, y0] = coords(start.point);
    let [x1, y1] = coords(end.point);
    let dx = x1 - x0;
    let dy = y1 - y0;
    let length_squared = dx * dx + dy * dy;
    let t = if length_squared > 0.0 {
        (((px - x0) * dx + (py - y0) * dy) / length_squared).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let width = start.width as f64 + t * (end.width - start.width) as f64;
    let touch_distance = threshold.max(0.5 * (point_width as f64 + width));
    let x = x0 + t * dx - px;
    let y = y0 + t * dy - py;
    (x * x + y * y).sqrt() <= touch_distance
}

fn coords(point: Point) -> [f64; 2] {
    [point.x() as f64, point.y() as f64]
}
