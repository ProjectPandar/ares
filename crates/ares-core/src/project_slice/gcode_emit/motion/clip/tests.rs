use super::clip_end;
use crate::{
    FloatOrPercent,
    geometry::{CoordinateScale, JoinType, Point, Polygon, offset_paths, simplify_closed_points},
    project_slice::perimeters::flow::build_nonbridging_flow,
};

#[test]
fn clips_a_closed_loop_from_its_end() {
    let mut points = vec![(0, 0), (4, 0), (4, 4), (0, 4), (0, 0)];

    clip_end(&mut points, 1.0);

    assert_eq!(points, vec![(0, 0), (4, 0), (4, 4), (0, 4), (0, 1)]);
}

#[test]
fn clips_across_short_terminal_segments() {
    let mut points = vec![(0, 0), (2, 0), (2, 2), (0, 2), (0, 0)];

    clip_end(&mut points, 3.0);

    assert_eq!(points, vec![(0, 0), (2, 0), (2, 2), (1, 2)]);
}

/// Sweep 9 `skirt_loops=10` fixture
/// `a2a00f959b47b4e4-981ed3251527fc53.3mf`: the eighth emitted loop is the
/// first source-visible rounding discriminator for `Polyline3::clip_end`.
#[test]
fn skirt_max_loop_clip_matches_source_squared_norm_rounding() {
    let scale = CoordinateScale::Normal;
    let flow = build_nonbridging_flow(FloatOrPercent::Float(0.42), 0.2, 0.4).unwrap();
    let scaled_spacing = (f64::from(flow.spacing) / scale.factor()) as f32;
    let mut distance = ((2.0 - f64::from(flow.spacing) * 0.5) / scale.factor()) as f32;
    let hull = Polygon::new(vec![
        scaled_point(scale, 105.0, 105.0),
        scaled_point(scale, 115.0, 105.0),
        scaled_point(scale, 115.0, 115.0),
        scaled_point(scale, 105.0, 115.0),
    ]);
    let mut loops = Vec::new();
    for _ in 0..10 {
        distance += scaled_spacing;
        let offset = offset_paths(
            std::slice::from_ref(&hull),
            distance,
            JoinType::Round,
            scale.checked_scale(0.1).unwrap() as f64,
        )
        .unwrap();
        loops.push(simplify_closed_points(
            offset[0].points().to_vec(),
            scale.checked_scale(0.05).unwrap() as f64,
        ));
    }
    loops.reverse();

    let mut seam_target = find_start_point(&loops[0], -135.0);
    let mut eighth_endpoint = None;
    for (index, loop_points) in loops.iter().enumerate() {
        let mut split = split_at_nearest(loop_points, seam_target)
            .into_iter()
            .map(|point| (point.x(), point.y()))
            .collect::<Vec<_>>();
        clip_end(&mut split, 0.04 / scale.factor());
        let endpoint = *split.last().unwrap();
        seam_target = Point::new(endpoint.0, endpoint.1);
        if index == 7 {
            eighth_endpoint = Some(endpoint);
        }
    }

    let endpoint = eighth_endpoint.unwrap();
    assert_eq!(
        (
            super::super::format::axis(scale.unscale(endpoint.0)),
            super::super::format::axis(scale.unscale(endpoint.1)),
        ),
        ("102.873".to_owned(), "103.107".to_owned())
    );
}

fn scaled_point(scale: CoordinateScale, x: f64, y: f64) -> Point {
    Point::new(
        scale.checked_scale(x).unwrap(),
        scale.checked_scale(y).unwrap(),
    )
}

fn find_start_point(points: &[Point], start_angle_deg: f64) -> Point {
    let (mut min_x, mut max_x) = (i64::MAX, i64::MIN);
    let (mut min_y, mut max_y) = (i64::MAX, i64::MIN);
    for point in points {
        min_x = min_x.min(point.x());
        max_x = max_x.max(point.x());
        min_y = min_y.min(point.y());
        max_y = max_y.max(point.y());
    }
    let center_x = (min_x + max_x) as f64 / 2.0;
    let center_y = (min_y + max_y) as f64 / 2.0;
    let radius = ((center_x - min_x as f64).powi(2) + (center_y - min_y as f64).powi(2)).sqrt();
    let radians = start_angle_deg.to_radians();
    Point::new(
        (center_x + radius * radians.cos()) as i64,
        (center_y + radius * radians.sin()) as i64,
    )
}

fn split_at_nearest(points: &[Point], target: Point) -> Vec<Point> {
    let mut best_distance = f64::MAX;
    let mut seam = points[0];
    let mut seam_index = 0;
    for (index, pair) in points.windows(2).enumerate() {
        let foot = projection_onto(pair[0], pair[1], target);
        let dx = (foot.x() - target.x()) as f64;
        let dy = (foot.y() - target.y()) as f64;
        let distance = dx * dx + dy * dy;
        if distance < best_distance {
            best_distance = distance;
            seam = foot;
            seam_index = index;
        }
    }
    let mut output = Vec::with_capacity(points.len() + 1);
    if seam == points[seam_index + 1] {
        output.extend_from_slice(&points[seam_index + 1..]);
        output.extend_from_slice(&points[..=seam_index]);
        output.push(seam);
    } else {
        output.push(seam);
        output.extend_from_slice(&points[seam_index + 1..]);
        output.extend_from_slice(&points[..=seam_index]);
        output.push(seam);
    }
    output
}

fn projection_onto(a: Point, b: Point, point: Point) -> Point {
    let lx = (b.x() - a.x()) as f64;
    let ly = (b.y() - a.y()) as f64;
    let theta =
        ((b.x() - point.x()) as f64 * lx + (b.y() - point.y()) as f64 * ly) / (lx * lx + ly * ly);
    if (0.0..=1.0).contains(&theta) {
        return Point::new(
            (theta * a.x() as f64 + (1.0 - theta) * b.x() as f64) as i64,
            (theta * a.y() as f64 + (1.0 - theta) * b.y() as f64) as i64,
        );
    }
    let da = (a.x() - point.x()).pow(2) + (a.y() - point.y()).pow(2);
    let db = (b.x() - point.x()).pow(2) + (b.y() - point.y()).pow(2);
    if da < db { a } else { b }
}
