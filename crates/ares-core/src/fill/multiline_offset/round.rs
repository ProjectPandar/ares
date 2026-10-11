use crate::geometry::{ClipperError, Point, Polygon, union_positive_polygons_paths};
use std::f64::consts::{PI, TAU};

type Normal = (f64, f64);

// Clipper2 clipper.offset.cpp: DoGroupOffset/DoRound/OffsetOpenPath,
// restricted to the positive constant offset and round ends used by multiline_fill.
pub(super) fn offset(paths: &[Polygon], delta: f64) -> Result<Vec<Polygon>, ClipperError> {
    let steps = (PI / (1.0_f64 - 0.002).acos()).min(delta * PI);
    let arc = Arc {
        delta,
        sin: (TAU / steps).sin(),
        cos: (TAU / steps).cos(),
        steps_per_rad: steps / TAU,
    };
    let mut output = Vec::with_capacity(paths.len());
    for path in paths {
        let mut points = path.points().to_vec();
        points.dedup();
        if points.len() == 1 {
            if delta >= 1.0 {
                output.push(arc.circle(points[0]));
            }
            continue;
        }
        let high = points.len() - 1;
        let mut normals: Vec<_> = points
            .windows(2)
            .map(|p| {
                let dx = (p[1].x() - p[0].x()) as f64;
                let dy = (p[1].y() - p[0].y()) as f64;
                let inverse = 1.0 / (dx * dx + dy * dy).sqrt();
                (dy * inverse, -dx * inverse)
            })
            .collect();
        normals.push((0.0, 0.0));
        let mut ring = Vec::new();
        arc.round(points[0], [negate(normals[0]), normals[0]], PI, &mut ring);
        for j in 1..high {
            arc.corner(points[j], normals[j - 1], normals[j], &mut ring);
        }
        for i in (1..=high).rev() {
            normals[i] = negate(normals[i - 1]);
        }
        normals[0] = normals[high];
        arc.round(
            points[high],
            [negate(normals[high]), normals[high]],
            PI,
            &mut ring,
        );
        for j in (1..high).rev() {
            arc.corner(points[j], normals[j + 1], normals[j], &mut ring);
        }
        output.push(Polygon::new(ring));
    }
    // Clipper2 clipper.offset.cpp:604-630 uses Positive for open round groups.
    union_positive_polygons_paths(&output)
}

fn negate(n: Normal) -> Normal {
    (-n.0, -n.1)
}

struct Arc {
    delta: f64,
    sin: f64,
    cos: f64,
    steps_per_rad: f64,
}

impl Arc {
    fn circle(&self, p: Point) -> Polygon {
        let count = (self.steps_per_rad * TAU).ceil() as usize;
        let sin = (TAU / count as f64).sin();
        let cos = (TAU / count as f64).cos();
        let mut unit = (cos, sin);
        let mut ring = Vec::with_capacity(count);
        ring.push(translated(p, (self.delta, 0.0)));
        for _ in 1..count {
            ring.push(translated(p, (self.delta * unit.0, self.delta * unit.1)));
            unit = (unit.0 * cos - unit.1 * sin, unit.1 * cos + unit.0 * sin);
        }
        Polygon::new(ring)
    }

    fn point(&self, p: Point, n: Normal) -> Point {
        translated(p, (n.0 * self.delta, n.1 * self.delta))
    }

    fn corner(&self, p: Point, previous: Normal, current: Normal, out: &mut Vec<Point>) {
        let sin = (current.1 * previous.0 - previous.1 * current.0).clamp(-1.0, 1.0);
        let cos = current.0 * previous.0 + current.1 * previous.1;
        if cos > -0.999 && sin < 0.0 {
            out.push(self.point(p, previous));
            out.push(p);
            out.push(self.point(p, current));
        } else {
            self.round(p, [previous, current], sin.atan2(cos), out);
        }
    }

    fn round(&self, p: Point, [previous, current]: [Normal; 2], angle: f64, out: &mut Vec<Point>) {
        let mut v = (previous.0 * self.delta, previous.1 * self.delta);
        out.push(translated(p, v));
        let steps = (self.steps_per_rad * angle.abs()).ceil() as usize;
        for _ in 1..steps {
            v = (
                v.0 * self.cos - self.sin * v.1,
                v.0 * self.sin + v.1 * self.cos,
            );
            out.push(translated(p, v));
        }
        out.push(self.point(p, current));
    }
}

fn translated(p: Point, v: Normal) -> Point {
    Point::new(
        (p.x() as f64 + v.0).round() as _,
        (p.y() as f64 + v.1).round() as _,
    )
}
