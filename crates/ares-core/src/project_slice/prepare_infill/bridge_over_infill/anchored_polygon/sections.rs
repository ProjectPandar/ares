use crate::geometry::{Line, LineDistanceTree, Point, Polygon, fixed_msvc_sort_by};

pub(super) fn build_sections(
    bridged_area: &[Polygon],
    anchors: &[Line],
    spacing: i64,
    width: i64,
) -> Vec<Vec<Line>> {
    let area_bounds = polygon_bounds(bridged_area);
    let anchor_bounds = line_bounds(anchors);
    let vertical_lines =
        vertical_lines(area_bounds.0.x(), area_bounds.1.x(), anchor_bounds, spacing);
    let area_lines = bridged_area
        .iter()
        .flat_map(Polygon::lines)
        .collect::<Vec<_>>();
    let area_tree = LineDistanceTree::new(&area_lines);
    let anchor_tree = LineDistanceTree::new(anchors);

    vertical_lines
        .into_iter()
        .map(|vertical| prepare_sections(&area_tree, &anchor_tree, vertical, width))
        .collect()
}

fn prepare_sections(
    area_tree: &LineDistanceTree<'_>,
    anchor_tree: &LineDistanceTree<'_>,
    vertical: Line,
    width: i64,
) -> Vec<Line> {
    let area_hits = area_tree.intersections_sorted(vertical);
    let mut sections = adjacent_sections(&area_hits, |point| area_tree.outside(point) < 0);
    let anchor_hits = anchor_tree.intersections_sorted(vertical);
    for section in &mut sections {
        extend_to_anchors(section, &anchor_hits, width);
    }
    order_sections(&mut sections);
    sections
}

fn adjacent_sections(
    hits: &[(Point, usize)],
    mut is_inside: impl FnMut(Point) -> bool,
) -> Vec<Line> {
    hits.windows(2)
        .filter(|pair| is_inside(midpoint(pair[0].0, pair[1].0)))
        .map(|pair| Line::new(pair[0].0, pair[1].0))
        .collect()
}

fn order_sections(sections: &mut Vec<Line>) {
    merge_overlapping(sections);
    sections.retain(|section| section.a != section.b);
    sort_sections(sections);
}

fn sort_sections(sections: &mut [Line]) {
    fixed_msvc_sort_by(sections, |left, right| {
        left != right && left.a.y() < right.b.y()
    });
}

fn midpoint(left: Point, right: Point) -> Point {
    Point::new((left.x() + right.x()) / 2, (left.y() + right.y()) / 2)
}

fn extend_to_anchors(section: &mut Line, anchors: &[(Point, usize)], width: i64) {
    if let Some((anchor, _)) = anchors
        .iter()
        .rev()
        .find(|(anchor, _)| section.a.y() > anchor.y())
    {
        section.a = Point::new(
            anchor.x(),
            (anchor.y() as f64 - width as f64 * (0.5 + 0.5)) as i64,
        );
    }
    if let Some((anchor, _)) = anchors
        .iter()
        .find(|(anchor, _)| section.b.y() < anchor.y())
    {
        section.b = Point::new(
            anchor.x(),
            (anchor.y() as f64 + width as f64 * (0.5 + 0.5)) as i64,
        );
    }
}

fn merge_overlapping(sections: &mut [Line]) {
    for index in 0..sections.len().saturating_sub(1) {
        let (before, after) = sections.split_at_mut(index + 1);
        let left = &mut before[index];
        let right = &mut after[0];
        if segments_overlap(left.a.y(), left.b.y(), right.a.y(), right.b.y()) {
            right.a = if left.a.y() < right.a.y() {
                left.a
            } else {
                right.a
            };
            right.b = if left.b.y() < right.b.y() {
                right.b
            } else {
                left.b
            };
            left.a = left.b;
        }
    }
}

fn segments_overlap(a_low: i64, a_high: i64, b_low: i64, b_high: i64) -> bool {
    (a_low >= b_low && a_low <= b_high)
        || (a_high >= b_low && a_high <= b_high)
        || (b_low >= a_low && b_low <= a_high)
        || (b_high >= a_low && b_high <= a_high)
}

fn vertical_lines(
    min_x: i64,
    max_x: i64,
    anchor_bounds: (Point, Point),
    spacing: i64,
) -> Vec<Line> {
    let count = ((max_x - min_x + spacing - 1) / spacing) as usize;
    (0..count)
        .map(|index| {
            let x = (min_x as f64 + (index as f64 + 0.5) * spacing as f64) as i64;
            Line::new(
                Point::new(x, anchor_bounds.0.y() - spacing),
                Point::new(x, anchor_bounds.1.y() + spacing),
            )
        })
        .collect()
}

fn polygon_bounds(polygons: &[Polygon]) -> (Point, Point) {
    point_bounds(
        polygons
            .iter()
            .flat_map(|polygon| polygon.points().iter().copied()),
    )
}

fn line_bounds(lines: &[Line]) -> (Point, Point) {
    point_bounds(lines.iter().flat_map(|line| [line.a, line.b]))
}

fn point_bounds(mut points: impl Iterator<Item = Point>) -> (Point, Point) {
    let first = points.next().expect("trusted geometry is nonempty");
    points.fold((first, first), |(minimum, maximum), point| {
        (
            Point::new(minimum.x().min(point.x()), minimum.y().min(point.y())),
            Point::new(maximum.x().max(point.x()), maximum.y().max(point.y())),
        )
    })
}

#[cfg(test)]
pub(super) fn all_adjacent_sections_for_test(points: &[Point]) -> Vec<Line> {
    let hits = points
        .iter()
        .copied()
        .enumerate()
        .map(|(index, point)| (point, index))
        .collect::<Vec<_>>();
    adjacent_sections(&hits, |_| true)
}

#[cfg(test)]
pub(super) fn sort_sections_for_test(mut sections: Vec<Line>) -> Vec<Line> {
    sort_sections(&mut sections);
    sections
}

#[cfg(test)]
pub(super) fn midpoint_for_test(left: Point, right: Point) -> Point {
    midpoint(left, right)
}

#[cfg(test)]
pub(super) fn vertical_lines_for_test(
    min_x: i64,
    max_x: i64,
    anchor_min: Point,
    anchor_max: Point,
    spacing: i64,
) -> Vec<Line> {
    vertical_lines(min_x, max_x, (anchor_min, anchor_max), spacing)
}

#[cfg(test)]
pub(super) fn extend_to_anchors_for_test(
    section: &mut Line,
    anchors: &[(Point, usize)],
    width: i64,
) {
    extend_to_anchors(section, anchors, width);
}

#[cfg(test)]
pub(super) fn prepare_order_for_test(mut sections: Vec<Line>) -> Vec<Line> {
    order_sections(&mut sections);
    sections
}
