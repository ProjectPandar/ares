use crate::geometry::{
    ClipperError, ExPolygon, JoinType, Point, Polygon, offset_expolygon, offset_expolygons,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IntersectionKind {
    OuterLow,
    OuterHigh,
    InnerLow,
    InnerHigh,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SegmentIntersection {
    pub(crate) point: Point,
    pub(crate) contour_index: usize,
    pub(crate) segment_index: usize,
    pub(crate) kind: IntersectionKind,
    pub(crate) previous: Option<(usize, LinkType, LinkQuality)>,
    pub(crate) next: Option<(usize, LinkType, LinkQuality)>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LinkType {
    Horizontal,
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LinkQuality {
    Valid,
    Invalid,
    TooLong,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SegmentedLine {
    pub(crate) x: i64,
    pub(crate) intersections: Vec<SegmentIntersection>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OffsetContour {
    pub(crate) polygon: Polygon,
    pub(crate) inner: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RectilinearSlice {
    pub(crate) source: ExPolygon,
    pub(crate) contours: Vec<OffsetContour>,
    pub(crate) lines: Vec<SegmentedLine>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "the source seam carries exact offset and scanline parameters"
)]
pub(crate) fn prepare_rectilinear_slice(
    expolygon: &ExPolygon,
    angle: f64,
    outer_offset: f32,
    inner_offset: f32,
    count: usize,
    x0: i64,
    spacing: i64,
) -> Result<RectilinearSlice, ClipperError> {
    let mut slice = prepare_rectilinear_contours(expolygon, angle, outer_offset, inner_offset)?;
    populate_vertical_lines(&mut slice, count, x0, spacing)?;
    Ok(slice)
}

pub(super) fn prepare_rectilinear_contours(
    expolygon: &ExPolygon,
    angle: f64,
    outer_offset: f32,
    inner_offset: f32,
) -> Result<RectilinearSlice, ClipperError> {
    let (source, contours) = prepare_contours(expolygon, angle, outer_offset, inner_offset)?;
    Ok(RectilinearSlice {
        source,
        contours,
        lines: Vec::new(),
    })
}

#[expect(
    clippy::excessive_nesting,
    reason = "source slicing walks retained contours, segments, then vertical lines"
)]
pub(super) fn populate_vertical_lines(
    slice: &mut RectilinearSlice,
    count: usize,
    x0: i64,
    spacing: i64,
) -> Result<(), ClipperError> {
    slice.lines = (0..count)
        .map(|index| {
            let delta = i64::try_from(index)
                .ok()
                .and_then(|index| index.checked_mul(spacing))
                .and_then(|delta| x0.checked_add(delta))
                .ok_or(ClipperError::CoordinateOutOfRange)?;
            Ok(SegmentedLine {
                x: delta,
                intersections: Vec::new(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    for (contour_index, contour) in slice.contours.iter().enumerate() {
        let points = contour.polygon.points();
        for segment_index in 0..points.len() {
            let first = points[(segment_index + points.len() - 1) % points.len()];
            let second = points[segment_index];
            for line in &mut slice.lines {
                if let Some(y) = intersection_y(points, segment_index, first, second, line.x) {
                    line.intersections.push(SegmentIntersection {
                        point: Point::new(line.x, y),
                        contour_index,
                        segment_index,
                        kind: kind(contour.inner, second.x() > first.x()),
                        previous: None,
                        next: None,
                    });
                }
            }
        }
    }
    for line in &mut slice.lines {
        line.intersections
            .sort_by_key(|intersection| intersection.point.y());
        remove_overlapping_vertices(line, &slice.contours);
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "temporary O77 test shell forwards the exact source slice parameters"
)]
pub(crate) fn slice_vertical_lines(
    expolygon: &ExPolygon,
    angle: f64,
    outer_offset: f32,
    inner_offset: f32,
    count: usize,
    x0: i64,
    spacing: i64,
) -> Result<Vec<SegmentedLine>, ClipperError> {
    prepare_rectilinear_slice(
        expolygon,
        angle,
        outer_offset,
        inner_offset,
        count,
        x0,
        spacing,
    )
    .map(|slice| slice.lines)
}

fn prepare_contours(
    expolygon: &ExPolygon,
    angle: f64,
    outer_offset: f32,
    inner_offset: f32,
) -> Result<(ExPolygon, Vec<OffsetContour>), ClipperError> {
    let rotated = rotate_expolygon(expolygon, angle)?;
    let outer = if outer_offset == 0.0 {
        vec![rotated.clone()]
    } else {
        offset_expolygon(&rotated, outer_offset, JoinType::Miter, 3.0)?
    };
    let inner = if inner_offset < 0.0 {
        offset_expolygons(&outer, inner_offset - outer_offset, JoinType::Miter, 3.0)?
    } else {
        Vec::new()
    };
    let mut contours = flatten(outer, false);
    contours.extend(flatten(inner, true));
    Ok((rotated, contours))
}

fn flatten(expolygons: Vec<ExPolygon>, inner: bool) -> Vec<OffsetContour> {
    let mut output = Vec::new();
    for expolygon in expolygons {
        let (contour, holes) = expolygon.into_parts();
        output.push(OffsetContour {
            polygon: contour,
            inner,
        });
        output.extend(
            holes
                .into_iter()
                .map(|polygon| OffsetContour { polygon, inner }),
        );
    }
    output
}

fn rotate_expolygon(expolygon: &ExPolygon, angle: f64) -> Result<ExPolygon, ClipperError> {
    let rotate = |polygon: &Polygon| {
        let cosine = angle.cos();
        let sine = angle.sin();
        polygon
            .points()
            .iter()
            .map(|point| {
                checked_point(
                    cosine * point.x() as f64 - sine * point.y() as f64,
                    sine * point.x() as f64 + cosine * point.y() as f64,
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Polygon::new)
    };
    Ok(ExPolygon::new(
        rotate(expolygon.contour())?,
        expolygon
            .holes()
            .iter()
            .map(rotate)
            .collect::<Result<_, _>>()?,
    ))
}

fn checked_point(x: f64, y: f64) -> Result<Point, ClipperError> {
    let x = x.round();
    let y = y.round();
    if !x.is_finite()
        || !y.is_finite()
        || !(i64::MIN as f64..-(i64::MIN as f64)).contains(&x)
        || !(i64::MIN as f64..-(i64::MIN as f64)).contains(&y)
    {
        return Err(ClipperError::CoordinateOutOfRange);
    }
    Ok(Point::new(x as i64, y as i64))
}

const fn kind(inner: bool, low: bool) -> IntersectionKind {
    match (inner, low) {
        (false, true) => IntersectionKind::OuterLow,
        (false, false) => IntersectionKind::OuterHigh,
        (true, true) => IntersectionKind::InnerLow,
        (true, false) => IntersectionKind::InnerHigh,
    }
}

fn remove_overlapping_vertices(line: &mut SegmentedLine, contours: &[OffsetContour]) {
    let mut output: Vec<SegmentIntersection> = Vec::with_capacity(line.intersections.len());
    for intersection in line.intersections.iter().copied() {
        let Some(previous) = output.last_mut() else {
            output.push(intersection);
            continue;
        };
        if previous.contour_index != intersection.contour_index
            || !at_segment_vertex(*previous, line.x, contours)
            || !at_segment_vertex(intersection, line.x, contours)
        {
            output.push(intersection);
            continue;
        }
        if previous.point.y() == intersection.point.y() {
            continue;
        }
        if previous.kind == intersection.kind {
            if !matches!(
                intersection.kind,
                IntersectionKind::OuterLow | IntersectionKind::InnerLow
            ) {
                *previous = intersection;
            }
            continue;
        }
        output.push(intersection);
    }
    line.intersections = output;
}

fn at_segment_vertex(
    intersection: SegmentIntersection,
    x: i64,
    contours: &[OffsetContour],
) -> bool {
    let points = contours[intersection.contour_index].polygon.points();
    let previous = (intersection.segment_index + points.len() - 1) % points.len();
    points[previous].x() == x || points[intersection.segment_index].x() == x
}

fn intersection_y(
    points: &[Point],
    segment_index: usize,
    first: Point,
    second: Point,
    x: i64,
) -> Option<i64> {
    let (left, right) = if first.x() <= second.x() {
        (first.x(), second.x())
    } else {
        (second.x(), first.x())
    };
    if x < left || x > right || first.x() == second.x() {
        return None;
    }
    if first.x() == x {
        let previous = points[(segment_index + points.len() - 2) % points.len()];
        return ((previous.x() - first.x()) as i128 * (second.x() - first.x()) as i128 <= 0)
            .then_some(first.y());
    }
    if second.x() == x {
        let next = points[(segment_index + 1) % points.len()];
        return ((next.x() - second.x()) as i128 * (first.x() - second.x()) as i128 <= 0)
            .then_some(second.y());
    }
    let denominator = (second.x() - first.x()).abs();
    let numerator = if second.x() > first.x() {
        x - first.x()
    } else {
        first.x() - x
    };
    let value = numerator as i128 * (second.y() - first.y()) as i128
        + first.y() as i128 * denominator as i128;
    Some(round_ratio(value, denominator))
}

fn round_ratio(mut numerator: i128, denominator: i64) -> i64 {
    numerator += if numerator < 0 {
        -i128::from(denominator >> 1)
    } else {
        i128::from(denominator >> 1)
    };
    (numerator / i128::from(denominator)) as i64
}
