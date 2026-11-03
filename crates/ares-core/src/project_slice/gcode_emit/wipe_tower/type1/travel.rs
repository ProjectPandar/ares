//! `GCode.cpp:594–710`: shorter squared-length route around the tower bbox.
//! The polyline points are `Point`s (grid ints) that `GCode.cpp:1034-1046`
//! feeds straight into `travel_to(p)` — never rescaled through `f32`.
use super::super::super::{motion, timelapse};
use super::super::postprocess;
use crate::geometry::CoordinateScale;
type Point = [i64; 2];

pub(super) fn path(
    source: &str,
    end: [f32; 2],
    state: &motion::EmitState,
    context: timelapse::Context<'_>,
    scale: CoordinateScale,
) -> Vec<Point> {
    let mut start = [0.0_f32; 2];
    // GCodeProcessor.cpp:3021: complete motion lines and first X/Y word.
    for line in source
        .split_inclusive('\n')
        .filter_map(|line| line.strip_suffix('\n'))
    {
        let line = line
            .trim_matches(' ')
            .trim_end_matches(';')
            .trim_end_matches(' ');
        if line.len() <= 5
            || !["G0 ", "G1 ", "G2 ", "G3 "]
                .iter()
                .any(|prefix| line.starts_with(prefix))
        {
            continue;
        }
        for (axis, key) in [" X", " Y"].iter().enumerate() {
            let Some((_, tail)) = line.split_once(key) else {
                continue;
            };
            let word = tail.split([' ', '\t', '\r', '\n', ';']).next().unwrap();
            if word.len() <= 8
                && let Ok(value) = word.parse::<f32>()
            {
                start[axis] = value;
            }
        }
    }
    let tower = state.wipe_tower.as_ref().unwrap();
    let bounds = tower.data.first_layer_bounds.unwrap();
    let translation = [
        tower.position[0] + tower.data.rib_offset[0],
        tower.position[1] + tower.data.rib_offset[1],
    ];
    // Tower and bed points take `plate_origin_2d` before `wipe_tower_point_to_object_point`
    // (`GCode.cpp:1015-1018, 1513-1522`).
    let plate_origin = state.xy_offset_f32();
    let local = |p: [f32; 2]| {
        let p = [p[0] + plate_origin[0], p[1] + plate_origin[1]];
        [
            ((f64::from(p[0]) - state.origin.0) / scale.factor()).round() as i64,
            ((f64::from(p[1]) - state.origin.1) / scale.factor()).round() as i64,
        ]
    };
    let min = bounds.min();
    let max = bounds.max();
    let corners = [
        [min.x(), min.y()],
        [max.x(), min.y()],
        [max.x(), max.y()],
        [min.x(), max.y()],
    ]
    .map(|p| {
        local(postprocess::transform(
            [scale.unscale(p[0]) as f32, scale.unscale(p[1]) as f32],
            translation,
            tower.angle,
        ))
    });
    let mut low = [i64::MAX; 2];
    let mut high = [i64::MIN; 2];
    for p in corners {
        for a in 0..2 {
            low[a] = low[a].min(p[a]);
            high[a] = high[a].max(p[a]);
        }
    }
    let offset = (2.0 / scale.factor()) as i64;
    for a in 0..2 {
        low[a] -= offset;
        high[a] += offset;
    }
    let points = [low, [high[0], low[1]], high, [low[0], high[1]]];
    let mut bed_min = [i64::MAX; 2];
    let mut bed_max = [i64::MIN; 2];
    for p in &context
        .traversal
        .resolved
        .views
        .full
        .printer
        .remaining
        .printable_area
        .0
    {
        let p = local([p.x as f32, p.y as f32]);
        for a in 0..2 {
            bed_min[a] = bed_min[a].min(p[a]);
            bed_max[a] = bed_max[a].max(p[a]);
        }
    }
    let end_local = local(end);
    if points
        .iter()
        .any(|p| (0..2).any(|a| p[a] <= bed_min[a] || p[a] >= bed_max[a]))
    {
        return vec![end_local];
    }
    // GCode.cpp:1513,8138: parsed G-code is in nozzle coordinates, unlike
    // tower points. It takes the plate origin, then gcode_to_point adds the
    // active nozzle offset and truncates.
    let start = [
        ((f64::from(start[0]) + f64::from(plate_origin[0]) - state.origin.0
            + state.extruder_offset.0)
            / scale.factor()) as i64,
        ((f64::from(start[1]) + f64::from(plate_origin[1]) - state.origin.1
            + state.extruder_offset.1)
            / scale.factor()) as i64,
    ];
    let direction = if (end_local[0] - low[0]).abs() < (high[0] - low[0]) / 2 {
        -1.0
    } else {
        1.0
    };
    let mm = |p: Point| [scale.unscale(p[0]), scale.unscale(p[1])];
    let intersect = |a: [f64; 2], v: [f64; 2], b: [f64; 2], c: [f64; 2], ray: bool| {
        let w = [c[0] - b[0], c[1] - b[1]];
        let d = [a[0] - b[0], a[1] - b[1]];
        let cross = |a: [f64; 2], b: [f64; 2]| a[0] * b[1] - a[1] * b[0];
        let denominator = cross(v, w);
        if denominator.abs() < 1e-4 {
            return None;
        }
        let t = cross(w, d) / denominator;
        let u = cross(v, d) / denominator;
        (t >= 0.0 && (ray || t <= 1.0) && (0.0..=1.0).contains(&u)).then(|| {
            [
                ((a[0] + t * v[0]) / scale.factor()) as i64,
                ((a[1] + t * v[1]) / scale.factor()) as i64,
            ]
        })
    };
    let Some((end_edge, hit)) = (0..4).find_map(|i| {
        intersect(
            mm(end_local),
            [direction, 0.0],
            mm(points[i]),
            mm(points[(i + 1) % 4]),
            true,
        )
        .map(|p| (i, p))
    }) else {
        return vec![end_local];
    };
    let a = mm(start);
    let b = mm(hit);
    let v = [b[0] - a[0], b[1] - a[1]];
    let entry = (0..4).filter(|&i| i != end_edge).find_map(|i| {
        intersect(a, v, mm(points[i]), mm(points[(i + 1) % 4]), false).map(|p| (i, p))
    });
    let mut path = if let Some((entry_edge, entry)) = entry {
        let around = |forward: bool| {
            let mut i = if forward {
                (entry_edge + 1) % 4
            } else {
                entry_edge
            };
            let end = if forward {
                end_edge
            } else {
                (end_edge + 1) % 4
            };
            let squared = |a: Point, b: Point| {
                let a = mm(a);
                let b = mm(b);
                (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)
            };
            let mut length = squared(entry, points[i]);
            let mut path = vec![entry];
            let step = if forward { 1 } else { 3 };
            while i != end {
                let next = (i + step) % 4;
                length += squared(points[i], points[next]);
                path.push(points[i]);
                i = next;
            }
            path.push(points[end]);
            path.push(hit);
            length += squared(hit, points[end]);
            (path, length)
        };
        let (forward, a) = around(true);
        let (backward, b) = around(false);
        if a < b { forward } else { backward }
    } else {
        vec![hit]
    };
    path.push(end_local);
    path
}
