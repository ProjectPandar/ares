use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::{LayerSlice, Point2, Segment2, SliceError};

mod overhang_printable;
pub(crate) use overhang_printable::apply as make_overhang_printable_contours;

#[derive(Clone, Debug, PartialEq)]
pub struct Contour {
    points: Vec<Point2>,
}

impl Contour {
    pub fn new(mut points: Vec<Point2>) -> Self {
        rotate_to_lowest_point(&mut points);
        if signed_area(&points) < 0.0 {
            points.reverse();
            rotate_to_lowest_point(&mut points);
        }
        Self { points }
    }

    pub fn points(&self) -> &[Point2] {
        &self.points
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerContours {
    layer_id: usize,
    print_z: f64,
    contours: Vec<Contour>,
}

impl LayerContours {
    pub fn new(layer_id: usize, print_z: f64, contours: Vec<Contour>) -> Self {
        Self {
            layer_id,
            print_z,
            contours,
        }
    }

    pub const fn layer_id(&self) -> usize {
        self.layer_id
    }

    pub const fn print_z(&self) -> f64 {
        self.print_z
    }

    pub fn contours(&self) -> &[Contour] {
        &self.contours
    }
}

pub fn stitch_layer_slices(slices: &[LayerSlice]) -> Result<Vec<LayerContours>, SliceError> {
    let mut layers = slices
        .iter()
        .map(|slice| {
            let mut contours = stitch_segments(slice.segments())?;
            contours.sort_by(compare_contours);
            Ok(LayerContours::new(
                slice.layer_id(),
                slice.print_z(),
                contours,
            ))
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    layers.sort_by_key(LayerContours::layer_id);
    Ok(layers)
}

pub(crate) fn stitch_printable(
    slices: &[LayerSlice],
    options: &crate::SliceOptions,
) -> Result<Vec<LayerContours>, SliceError> {
    Ok(make_overhang_printable_contours(
        stitch_layer_slices(slices)?,
        options.perimeter_options()?,
    ))
}

fn stitch_segments(segments: &[Segment2]) -> Result<Vec<Contour>, SliceError> {
    if segments.is_empty() {
        return Ok(Vec::new());
    }

    let mut seen = BTreeSet::new();
    let mut adjacency: BTreeMap<PointKey, Vec<Point2>> = BTreeMap::new();
    for segment in segments {
        if !seen.insert(SegmentKey::from(*segment)) {
            return Err(SliceError::InvalidInput(
                "slice segments contain duplicate edges".to_owned(),
            ));
        }
        adjacency
            .entry(PointKey::from(segment.start()))
            .or_default()
            .push(segment.end());
        adjacency
            .entry(PointKey::from(segment.end()))
            .or_default()
            .push(segment.start());
    }
    if adjacency.values().any(|neighbors| neighbors.len() != 2) {
        return Err(SliceError::InvalidInput(
            "slice segments do not form simple closed contours".to_owned(),
        ));
    }

    let mut unused: Vec<Segment2> = segments.to_vec();
    unused.sort_by(compare_segments);
    let mut contours = Vec::new();

    while let Some(first) = unused.first().copied() {
        let mut points = vec![first.start()];
        let start = first.start();
        let mut previous = first.start();
        let mut current = first.end();
        remove_segment(&mut unused, first)?;

        while current != start {
            points.push(current);
            let next = adjacency
                .get(&PointKey::from(current))
                .and_then(|neighbors| neighbors.iter().copied().find(|point| *point != previous))
                .ok_or_else(|| SliceError::InvalidInput("slice contour is open".to_owned()))?;
            let segment = Segment2::new(current, next);
            remove_segment(&mut unused, segment)?;
            previous = current;
            current = next;
        }

        if points.len() < 3 {
            return Err(SliceError::InvalidInput(
                "slice contour has fewer than three points".to_owned(),
            ));
        }
        contours.push(Contour::new(points));
    }

    Ok(contours)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct PointKey {
    x: i64,
    y: i64,
}

impl From<Point2> for PointKey {
    fn from(point: Point2) -> Self {
        Self {
            x: scaled(point.x()),
            y: scaled(point.y()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct SegmentKey {
    start: PointKey,
    end: PointKey,
}

impl From<Segment2> for SegmentKey {
    fn from(segment: Segment2) -> Self {
        Self {
            start: PointKey::from(segment.start()),
            end: PointKey::from(segment.end()),
        }
    }
}

fn scaled(value: f64) -> i64 {
    (value * 1_000_000.0).round() as i64
}

fn remove_segment(unused: &mut Vec<Segment2>, segment: Segment2) -> Result<(), SliceError> {
    let Some(index) = unused.iter().position(|candidate| *candidate == segment) else {
        return Err(SliceError::InvalidInput(
            "slice segments contain duplicate or inconsistent edges".to_owned(),
        ));
    };
    unused.remove(index);
    Ok(())
}

fn rotate_to_lowest_point(points: &mut [Point2]) {
    if let Some(index) = points
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| compare_points(**a, **b))
        .map(|(index, _)| index)
    {
        points.rotate_left(index);
    }
}

fn signed_area(points: &[Point2]) -> f64 {
    let mut area = 0.0;
    for (a, b) in points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
    {
        area += a.x() * b.y() - b.x() * a.y();
    }
    area / 2.0
}

fn compare_points(a: Point2, b: Point2) -> Ordering {
    a.x()
        .total_cmp(&b.x())
        .then_with(|| a.y().total_cmp(&b.y()))
}

fn compare_segments(a: &Segment2, b: &Segment2) -> Ordering {
    compare_points(a.start(), b.start()).then_with(|| compare_points(a.end(), b.end()))
}

fn compare_contours(a: &Contour, b: &Contour) -> Ordering {
    a.points()
        .iter()
        .zip(b.points())
        .map(|(a, b)| compare_points(*a, *b))
        .find(|ordering| !ordering.is_eq())
        .unwrap_or_else(|| a.points().len().cmp(&b.points().len()))
}

#[cfg(test)]
mod tests;
