use crate::geometry::{CoordinateScale, Point, Polygon, ThickPolyline};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ExtrusionJunction {
    pub(crate) point: Point,
    pub(crate) width: i64,
    pub(crate) perimeter_index: usize,
}

impl ExtrusionJunction {
    pub(crate) const fn new(point: Point, width: i64, perimeter_index: usize) -> Self {
        Self {
            point,
            width,
            perimeter_index,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExtrusionLine {
    pub(crate) inset_index: usize,
    pub(crate) is_odd: bool,
    pub(crate) is_closed: bool,
    pub(crate) junctions: Vec<ExtrusionJunction>,
}

impl ExtrusionLine {
    pub(crate) const fn new(inset_index: usize, is_odd: bool) -> Self {
        Self {
            inset_index,
            is_odd,
            is_closed: false,
            junctions: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, junction: ExtrusionJunction) {
        self.junctions.push(junction);
    }

    pub(crate) fn remove(&mut self, index: usize) -> ExtrusionJunction {
        self.junctions.remove(index)
    }

    pub(crate) fn insert(&mut self, index: usize, junction: ExtrusionJunction) {
        self.junctions.insert(index, junction);
    }

    pub(crate) fn clear(&mut self) {
        self.junctions.clear();
    }

    pub(crate) fn reverse(&mut self) {
        self.junctions.reverse();
    }

    pub(crate) fn length(&self) -> i64 {
        let mut length = self
            .junctions
            .windows(2)
            .map(|pair| distance(pair[0].point, pair[1].point) as i64)
            .sum();
        if self.is_closed && !self.junctions.is_empty() {
            length += distance(
                self.junctions[0].point,
                self.junctions[self.junctions.len() - 1].point,
            ) as i64;
        }
        length
    }

    pub(crate) fn to_polygon(&self) -> Polygon {
        Polygon::new(
            self.junctions
                .iter()
                .map(|junction| junction.point)
                .collect(),
        )
    }

    pub(crate) fn to_thick_polyline(&self) -> ThickPolyline {
        assert!(self.junctions.len() >= 2);
        let mut points = Vec::with_capacity(self.junctions.len());
        let mut width = Vec::with_capacity(2 * (self.junctions.len() - 1));
        points.push(self.junctions[0].point);
        width.push(self.junctions[0].width as f64);
        points.push(self.junctions[1].point);
        width.push(self.junctions[1].width as f64);
        for pair in self.junctions[1..].windows(2) {
            points.push(pair[1].point);
            width.push(pair[0].width as f64);
            width.push(pair[1].width as f64);
        }
        ThickPolyline {
            points,
            width,
            endpoints: (false, false),
        }
    }

    pub(crate) fn is_contour(&self) -> bool {
        self.is_closed && self.area() < 0.0
    }

    pub(crate) fn area(&self) -> f64 {
        assert!(self.is_closed);
        let Some(mut previous) = self.junctions.last().map(|junction| junction.point) else {
            return 0.0;
        };
        let mut area = 0.0;
        for junction in &self.junctions {
            area += previous.x() as f64 * junction.point.y() as f64
                - previous.y() as f64 * junction.point.x() as f64;
            previous = junction.point;
        }
        0.5 * area
    }

    pub(crate) fn extrusion_area_deviation(
        a: ExtrusionJunction,
        b: ExtrusionJunction,
        c: ExtrusionJunction,
    ) -> i64 {
        let ab_length = distance(a.point, b.point) as i64;
        let bc_length = distance(b.point, c.point) as i64;
        let width_difference = (b.width - a.width).abs().max((c.width - b.width).abs());
        if width_difference > 1 {
            let ab_weight = (a.width + b.width) / 2;
            let bc_weight = (b.width + c.width) / 2;
            let weighted_width =
                (ab_length * ab_weight + bc_length * bc_weight) / (ab_length + bc_length);
            let ac_length = distance(a.point, c.point) as i64;
            ((ab_weight * ab_length + bc_weight * bc_length) - weighted_width * ac_length).abs()
        } else if ab_length > bc_length {
            width_difference * bc_length
        } else {
            width_difference * ab_length
        }
    }

    pub(crate) fn simplify(
        &mut self,
        smallest_segment_squared: i64,
        allowed_error_squared: i64,
        maximum_area_deviation: i64,
        scale: CoordinateScale,
    ) {
        let minimum_size = if self.is_closed { 3 } else { 2 };
        if self.junctions.len() <= minimum_size {
            return;
        }
        let (five_microns, floating_five_microns) = five_micron_tolerances(scale);
        let five_microns_squared = five_microns * five_microns;
        let mut output = vec![self.junctions[0]];
        let mut previous = self.junctions[0];
        let mut previous_previous = if self.is_closed {
            self.junctions[self.junctions.len() - 2]
        } else {
            previous
        };
        let initial = self.junctions[1];
        let mut removed_area = cross(previous.point, initial.point);
        let end = self.junctions.len() - usize::from(!self.is_closed);

        for index in 1..end {
            let is_last = index + 1 == self.junctions.len();
            let current = if is_last {
                output[0]
            } else {
                self.junctions[index]
            };
            if self.is_closed && output.len() + (self.junctions.len() - index) <= 3 {
                output.push(current);
                continue;
            }
            let spill = self.is_closed
                && index + 2 >= self.junctions.len()
                && index + 2 - self.junctions.len() < output.len();
            let next = if spill {
                output[index + 2 - self.junctions.len()]
            } else {
                self.junctions[index + 1]
            };
            let removed_area_next = cross(current.point, next.point);
            let closing_area = cross(next.point, previous.point);
            removed_area += removed_area_next;
            let length_squared = distance_squared(current.point, previous.point);
            if length_squared < five_microns_squared {
                continue;
            }
            let area = removed_area + closing_area;
            let base_squared = distance_squared(next.point, previous.point);
            if base_squared == 0 {
                continue;
            }
            let height_squared = (area as f64 * area as f64 / base_squared as f64) as i64;
            let area_error = Self::extrusion_area_deviation(previous, current, next);
            if height_squared <= five_microns_squared
                && distance_to_infinite(current.point, previous.point, next.point)
                    <= floating_five_microns
                && area_error <= maximum_area_deviation
            {
                continue;
            }
            let action = if length_squared < smallest_segment_squared
                && height_squared <= allowed_error_squared
            {
                short_segment_action(
                    [previous_previous, previous, current, next],
                    smallest_segment_squared,
                    allowed_error_squared,
                )
            } else {
                ShortSegmentAction::Keep
            };
            match action {
                ShortSegmentAction::Remove => continue,
                ShortSegmentAction::Replace(replacement) => {
                    output.pop();
                    previous = previous_previous;
                    removed_area = removed_area_next;
                    previous_previous = previous;
                    previous = replacement;
                    output.push(replacement);
                    continue;
                }
                ShortSegmentAction::Keep => {}
            }
            removed_area = removed_area_next;
            previous_previous = previous;
            previous = current;
            output.push(current);
        }
        if self.is_closed {
            output[0].point = output.last().unwrap().point;
        } else {
            output.push(*self.junctions.last().unwrap());
        }
        self.junctions = output;
    }
}

enum ShortSegmentAction {
    Keep,
    Remove,
    Replace(ExtrusionJunction),
}

fn short_segment_action(
    [previous_previous, previous, current, next]: [ExtrusionJunction; 4],
    smallest_segment_squared: i64,
    allowed_error_squared: i64,
) -> ShortSegmentAction {
    if distance_squared(current.point, next.point) <= 4 * smallest_segment_squared {
        return ShortSegmentAction::Remove;
    }
    let Some(intersection) = infinite_intersection(
        previous_previous.point,
        previous.point,
        current.point,
        next.point,
    ) else {
        return ShortSegmentAction::Keep;
    };
    if distance_to_infinite(intersection, previous.point, current.point).powi(2)
        > allowed_error_squared as f64
        || distance_greater(intersection, previous.point, smallest_segment_squared)
        || distance_greater(intersection, current.point, smallest_segment_squared)
    {
        return ShortSegmentAction::Keep;
    }
    ShortSegmentAction::Replace(ExtrusionJunction::new(
        intersection,
        current.width,
        current.perimeter_index,
    ))
}

fn distance(left: Point, right: Point) -> f64 {
    (distance_squared(left, right) as f64).sqrt()
}

pub(super) fn five_micron_tolerances(scale: CoordinateScale) -> (i64, f64) {
    (scale.checked_scale(0.005).unwrap(), 0.005 / scale.factor())
}

fn distance_squared(left: Point, right: Point) -> i64 {
    let x = left.x() - right.x();
    let y = left.y() - right.y();
    x * x + y * y
}

fn cross(left: Point, right: Point) -> i64 {
    left.x() * right.y() - left.y() * right.x()
}

fn distance_to_infinite(point: Point, start: Point, end: Point) -> f64 {
    let dx = (end.x() - start.x()) as f64;
    let dy = (end.y() - start.y()) as f64;
    let numerator =
        (dy * (point.x() - start.x()) as f64 - dx * (point.y() - start.y()) as f64).abs();
    numerator / (dx * dx + dy * dy).sqrt()
}

pub(super) fn infinite_intersection(a: Point, b: Point, c: Point, d: Point) -> Option<Point> {
    let ab = ((b.x() - a.x()) as f64, (b.y() - a.y()) as f64);
    let cd = ((d.x() - c.x()) as f64, (d.y() - c.y()) as f64);
    let denominator = ab.0 * cd.1 - ab.1 * cd.0;
    if denominator.abs() < 1.0e-4 {
        return None;
    }
    let ac = ((c.x() - a.x()) as f64, (c.y() - a.y()) as f64);
    let t = (ac.0 * cd.1 - ac.1 * cd.0) / denominator;
    let x = a.x() as f64 + t * ab.0;
    let y = a.y() as f64 + t * ab.1;
    let maximum_exclusive = -(i64::MIN as f64);
    if !x.is_finite()
        || !y.is_finite()
        || x < i64::MIN as f64
        || x >= maximum_exclusive
        || y < i64::MIN as f64
        || y >= maximum_exclusive
    {
        return None;
    }
    Some(Point::new(x as i64, y as i64))
}

fn distance_greater(left: Point, right: Point, threshold_squared: i64) -> bool {
    let x = (left.x() - right.x()).unsigned_abs();
    let y = (left.y() - right.y()).unsigned_abs();
    x > threshold_squared as u64
        || y > threshold_squared as u64
        || x * x + y * y > threshold_squared as u64
}
