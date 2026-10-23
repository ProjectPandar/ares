//! Type1 always fits arcs (`WipeTower.cpp:865–907,1102–1153`), unlike Type2.
use super::Writer;
use crate::geometry::{CoordinateScale, Point, Polygon, Polyline};
use crate::project_slice::{gcode_emit::motion, perimeters::classic::materialize::FittedArc};
use std::fmt::Write;

struct Segment {
    start: [f32; 2],
    end: [f32; 2],
    arc: Option<FittedArc>,
}
fn append_segments(segments: &mut Vec<Segment>, path: &mut Polyline, scale: CoordinateScale) {
    if path.points().len() < 2 {
        return;
    }
    let mut points = path
        .points()
        .iter()
        .map(|p| (scale.unscale(p.x()), scale.unscale(p.y())))
        .collect::<Vec<_>>();
    let fitting = motion::simplify_points(&mut points, 0.1, 1.0 / scale.factor());
    *path = Polyline::new(
        points
            .into_iter()
            .map(|(x, y)| {
                Point::new(
                    (x / scale.factor()).round() as i64,
                    (y / scale.factor()).round() as i64,
                )
            })
            .collect(),
    );
    for fit in fitting {
        // WipeTower.cpp:1124–1128 / Point.hpp:700–705: unscaled<float>(Point).
        let convert = |i: usize| {
            let point = path.points()[i];
            [
                point.x() as f32 * scale.factor() as f32,
                point.y() as f32 * scale.factor() as f32,
            ]
        };
        if let Some(arc) = fit.arc {
            segments.push(Segment {
                start: convert(fit.start),
                end: convert(fit.end),
                arc: Some(arc),
            });
        } else {
            for i in fit.start..fit.end {
                segments.push(Segment {
                    start: convert(i),
                    end: convert(i + 1),
                    arc: None,
                });
            }
        }
    }
}

fn normalize_angle(mut angle: f64) -> f64 {
    if angle > std::f64::consts::TAU {
        angle -= std::f64::consts::TAU;
    } else if angle < 0.0 {
        angle += std::f64::consts::TAU;
    }
    angle
}

impl Writer {
    pub(in super::super) fn paths(
        &mut self,
        paths: &mut [Polyline],
        speed: f32,
        retract: (f32, f32),
        scale: CoordinateScale,
    ) {
        let mut segments = Vec::new();
        for path in paths {
            append_segments(&mut segments, path, scale);
        }
        let mut nearest = 0;
        let mut minimum = f32::MAX;
        for (index, segment) in segments.iter().enumerate() {
            let dx = segment.start[0] - self.position[0];
            let dy = segment.start[1] - self.position[1];
            let distance = dx * dx + dy * dy;
            if distance < minimum {
                minimum = distance;
                nearest = index;
            }
        }
        self.travel(segments[nearest].start, 0.0);
        for offset in 0..segments.len() {
            let segment = &segments[(nearest + offset) % segments.len()];
            let dx = segment.start[0] - self.position[0];
            let dy = segment.start[1] - self.position[1];
            if offset != 0 && f64::from((dx * dx + dy * dy).sqrt()) > 1e-4 {
                self.load(-retract.0, retract.1);
                self.travel(segment.start, 600.0);
                self.load(retract.0, retract.1);
            }
            if let Some(arc) = &segment.arc {
                self.arc(segment.end, arc, speed, scale);
            } else {
                self.extrude(segment.end, speed);
            }
        }
    }

    pub(in super::super) fn polygon(
        &mut self,
        polygon: &Polygon,
        speed: f32,
        scale: CoordinateScale,
    ) {
        let mut points = polygon
            .points()
            .iter()
            .map(|p| (scale.unscale(p.x()), scale.unscale(p.y())))
            .collect::<Vec<_>>();
        points.push(points[0]);
        motion::simplify_linear_points(&mut points, 0.001, 1.0 / scale.factor());
        let points = points
            .into_iter()
            .map(|(x, y)| {
                Point::new(
                    (x / scale.factor()).round() as i64,
                    (y / scale.factor()).round() as i64,
                )
            })
            .collect();
        self.paths(&mut [Polyline::new(points)], speed, (0.0, 0.0), scale);
    }

    pub(in super::super) fn polygon_wipe(
        &mut self,
        polygon: &Polygon,
        distance: f32,
        scale: CoordinateScale,
    ) {
        let mut distance = f64::from(distance);
        let position = super::super::super::geometry::scaled(self.position, scale);
        let start = polygon
            .points()
            .iter()
            .enumerate()
            .min_by_key(|(_, p)| {
                let dx = i128::from(p.x()) - i128::from(position.x());
                let dy = i128::from(p.y()) - i128::from(position.y());
                dx * dx + dy * dy
            })
            .unwrap()
            .0;
        let n = polygon.points().len();
        for offset in 0..=n {
            if distance < 1e-4 {
                break;
            }
            let p = polygon.points()[(start + n - offset % n) % n];
            let factor = scale.factor() as f32;
            self.add_wipe([p.x() as f32 * factor, p.y() as f32 * factor]);
            if offset > 0 {
                let previous = polygon.points()[(start + n - (offset - 1) % n) % n];
                let dx = scale.unscale(p.x() - previous.x());
                let dy = scale.unscale(p.y() - previous.y());
                distance -= (dx * dx + dy * dy).sqrt();
            }
        }
    }

    fn arc(&mut self, end: [f32; 2], arc: &FittedArc, mut speed: f32, scale: CoordinateScale) {
        // ArcSegment stores scaled length and an integer Point center;
        // WipeTowerWriter uses unscaled<float>, not unscale(...).cast<float>().
        let length = (arc.length / scale.factor()) as f32 * scale.factor() as f32;
        let amount = length * self.flow;
        if length < 1e-4_f32 && amount == 0.0 && (speed == 0.0 || speed == self.feedrate) {
            return;
        }
        self.used += amount;
        let start = self.rotated(self.position);
        let target = self.rotated(end);
        let center = [arc.center.0, arc.center.1]
            .map(|value| (value / scale.factor()).round() as f32 * scale.factor() as f32);
        if !self.suppress_preview && amount > 0.0 && length > 0.0 {
            let width = amount * self.area / (length * self.height)
                + self.height * (1.0 - std::f64::consts::FRAC_PI_4) as f32;
            if self.extrusions.last().is_none_or(|e| e.position != start) {
                self.extrusions.push(super::Extrusion {
                    position: start,
                    width: 0.0,
                });
            }
            let theta = (f64::from(self.position[1]) - arc.center.1)
                .atan2(f64::from(self.position[0]) - arc.center.0);
            let angle = arc.length / arc.radius * if arc.clockwise { -1.0 } else { 1.0 };
            for i in 0..20 {
                let t = normalize_angle(theta + f64::from(i as f32 / 20.0) * angle);
                let x = ((arc.center.0 / scale.factor()) as i64
                    + (arc.radius / scale.factor() * t.cos()) as i64)
                    as f32
                    * scale.factor() as f32;
                let y = ((arc.center.1 / scale.factor()) as i64
                    + (arc.radius / scale.factor() * t.sin()) as i64)
                    as f32
                    * scale.factor() as f32;
                self.extrusions.push(super::Extrusion {
                    position: self.rotated([x, y]),
                    width,
                });
            }
            self.extrusions.push(super::Extrusion {
                position: target,
                width,
            });
        }
        let center = self.rotated(center);
        write!(
            self.gcode,
            "G{} X{:.3} Y{:.3} I{:.3} J{:.3}",
            if arc.clockwise { 2 } else { 3 },
            target[0],
            target[1],
            center[0] - start[0],
            center[1] - start[1]
        )
        .unwrap();
        if amount != 0.0 {
            write!(self.gcode, " E{amount:.4}").unwrap();
        }
        if speed != 0.0 && speed != self.feedrate {
            let e_speed =
                amount / (if length == 0.0 { amount.abs() } else { length } / speed * 60.0);
            speed /= (e_speed / self.max_e_speed).max(1.0);
            self.format_speed(speed);
        }
        self.position = end;
        self.gcode.push('\n');
    }
}
