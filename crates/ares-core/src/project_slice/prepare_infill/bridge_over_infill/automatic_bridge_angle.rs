use std::f64::consts::PI;

use crate::geometry::{CoordinateScale, Line, LineDistanceTree, Point, Polygon};

#[derive(Clone, Copy)]
struct DirectionBucket {
    angle: f64,
    count: i32,
}

pub(in crate::project_slice) fn determine_automatic_bridge_angle(
    bridged_area: &[Polygon],
    anchors: &[Line],
    scale: CoordinateScale,
) -> f64 {
    let directions = count_directions(bridged_area, anchors, scale);
    let mut angle = reduce_directions(&directions);
    if angle == 0.0 {
        angle = 0.001;
    }
    angle
}

fn count_directions(
    bridged_area: &[Polygon],
    anchors: &[Line],
    scale: CoordinateScale,
) -> Vec<DirectionBucket> {
    let tree = LineDistanceTree::new(anchors);
    let mut directions = Vec::new();
    for_each_sample(bridged_area, scale, |point| {
        let nearest = tree
            .nearest(point)
            .expect("sampled bridge angle requires at least one anchor");
        let mut angle = anchors[nearest.line_index].orientation();
        if angle > PI {
            angle -= PI;
        }
        angle += PI * 0.5;
        insert_direction(&mut directions, angle, 1);
    });
    directions
}

fn for_each_sample(bridged_area: &[Polygon], scale: CoordinateScale, mut visit: impl FnMut(Point)) {
    let scaled_two_mm = (2.0 / scale.factor()) as i64 as f64;
    for polygon in bridged_area {
        let mut accumulated_distance = 0.0;
        for points in polygon.points().windows(2) {
            let segment = SampleSegment::new(points[0], points[1]);
            accumulated_distance += segment.distance;
            if accumulated_distance <= scaled_two_mm {
                continue;
            }
            accumulated_distance = 0.0;
            sample_segment(segment, scaled_two_mm, &mut visit);
        }
    }
}

#[derive(Clone, Copy)]
struct SampleSegment {
    start_x: f64,
    start_y: f64,
    vx: f64,
    vy: f64,
    distance: f64,
}

impl SampleSegment {
    fn new(start: Point, end: Point) -> Self {
        let start_x = start.x() as f64;
        let start_y = start.y() as f64;
        let vx = end.x() as f64 - start_x;
        let vy = end.y() as f64 - start_y;
        Self {
            start_x,
            start_y,
            vx,
            vy,
            distance: (vx * vx + vy * vy).sqrt(),
        }
    }
}

fn sample_segment(segment: SampleSegment, scaled_two_mm: f64, visit: &mut impl FnMut(Point)) {
    let squared_norm = segment.vx * segment.vx + segment.vy * segment.vy;
    let norm = squared_norm.sqrt();
    let (normalized_x, normalized_y) = if squared_norm > 0.0 {
        (segment.vx / norm, segment.vy / norm)
    } else {
        (segment.vx, segment.vy)
    };
    let line_count = (segment.distance / scaled_two_mm).ceil() as i32;
    let step_size = (segment.distance / f64::from(line_count)) as f32;
    for index in 0..line_count {
        let offset = f64::from(index as f32 * step_size);
        visit(Point::new(
            (segment.start_x + normalized_x * offset) as i64,
            (segment.start_y + normalized_y * offset) as i64,
        ));
    }
}

fn insert_direction(directions: &mut Vec<DirectionBucket>, angle: f64, count: i32) {
    let mut index = 0;
    while index < directions.len() && directions[index].angle < angle {
        index += 1;
    }
    if index < directions.len() && angle == directions[index].angle {
        directions[index].count += count;
    } else {
        directions.insert(index, DirectionBucket { angle, count });
    }
}

fn reduce_directions(directions: &[DirectionBucket]) -> f64 {
    let mut best_angle = 0.0;
    let mut best_score = 0;
    for direction in directions {
        let (weighted_angle, score) = accumulate_window(directions, direction.angle);
        if score > best_score {
            best_angle = weighted_angle / f64::from(score);
            best_score = score;
        }
    }
    best_angle
}

fn accumulate_window(directions: &[DirectionBucket], direction: f64) -> (f64, i32) {
    let mut score = 0;
    let mut weighted_angle = 0.0;
    let window_start = direction - PI * 0.1;
    let window_end = direction + PI * 0.1;
    for bucket in directions
        .iter()
        .filter(|bucket| bucket.angle >= window_start && bucket.angle <= window_end)
    {
        weighted_angle += bucket.angle * f64::from(bucket.count);
        score += bucket.count;
    }
    if window_start < 0.5 * PI {
        let wrapped_start = 1.5 * PI - (0.5 * PI - window_start);
        for bucket in directions
            .iter()
            .filter(|bucket| bucket.angle >= wrapped_start)
        {
            weighted_angle += (bucket.angle - PI) * f64::from(bucket.count);
            score += bucket.count;
        }
    }
    if window_start > 1.5 * PI {
        let wrapped_end = window_start - 1.5 * PI;
        for bucket in directions
            .iter()
            .filter(|bucket| bucket.angle <= wrapped_end)
        {
            weighted_angle += (bucket.angle + PI) * f64::from(bucket.count);
            score += bucket.count;
        }
    }
    (weighted_angle, score)
}
