use std::f64::consts::PI;

use boostvoronoi::prelude::{Diagram, EdgeIndex, SourceCategory};

use crate::geometry::{Line, Point};

use super::{MedialAxisError, diagram};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct EdgeData {
    pub(crate) active: bool,
    pub(crate) width_start: f64,
    pub(crate) width_end: f64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ValidationLimits {
    pub(crate) min_width: f64,
    pub(crate) max_width: f64,
    pub(crate) scaled_epsilon: f64,
}

pub(crate) fn validate(
    vd: &Diagram,
    edge: EdgeIndex,
    lines: &[Line],
    limits: ValidationLimits,
) -> Result<Option<EdgeData>, MedialAxisError> {
    let [x0, y0, x1, y1] = vd.edge_as_line(edge).map_err(invariant)?;
    let medial = Line::new(integer_point(x0, y0), integer_point(x1, y1));
    let twin = diagram::twin(vd, edge)?;
    let left = vd.cell(diagram::cell(vd, edge)?).map_err(invariant)?;
    let right = vd.cell(diagram::cell(vd, twin)?).map_err(invariant)?;
    let segment_left = lines[left.source_index().usize()];
    let segment_right = lines[right.source_index().usize()];
    let mut width0 = if right.contains_segment() {
        2.0 * segment_right.distance_to(medial.a)
    } else {
        2.0 * point_distance(endpoint(segment_right, right.source_category())?, medial.a)
    };
    let mut width1 = if left.contains_segment() {
        2.0 * segment_left.distance_to(medial.b)
    } else {
        2.0 * point_distance(endpoint(segment_left, left.source_category())?, medial.b)
    };
    if left.contains_segment() && right.contains_segment() {
        let mut angle = (segment_right.orientation() - segment_left.orientation()).abs();
        if angle > PI {
            angle = 2.0 * PI - angle;
        }
        if PI - angle > PI / 8.0
            && (width0 < limits.scaled_epsilon
                || width1 < limits.scaled_epsilon
                || medial.length() >= limits.min_width)
        {
            return Ok(None);
        }
    } else if width0 < limits.scaled_epsilon || width1 < limits.scaled_epsilon {
        return Ok(None);
    }
    if (width0 >= limits.min_width || width1 >= limits.min_width)
        && (width0 <= limits.max_width || width1 <= limits.max_width)
    {
        if edge.usize() % 2 == 1 {
            std::mem::swap(&mut width0, &mut width1);
        }
        Ok(Some(EdgeData {
            active: true,
            width_start: width0,
            width_end: width1,
        }))
    } else {
        Ok(None)
    }
}

fn endpoint(line: Line, category: SourceCategory) -> Result<Point, MedialAxisError> {
    match category {
        SourceCategory::SegmentStart => Ok(line.a),
        SourceCategory::SegmentEnd => Ok(line.b),
        SourceCategory::Segment | SourceCategory::SinglePoint => {
            Err(MedialAxisError::ConstructionFailed)
        }
    }
}

fn point_distance(left: Point, right: Point) -> f64 {
    let dx = (left.x() - right.x()) as f64;
    let dy = (left.y() - right.y()) as f64;
    (dx * dx + dy * dy).sqrt()
}

pub(crate) fn integer_point(x: f64, y: f64) -> Point {
    Point::new(x as i64, y as i64)
}

fn invariant(_: boostvoronoi::BvError) -> MedialAxisError {
    MedialAxisError::ConstructionFailed
}
