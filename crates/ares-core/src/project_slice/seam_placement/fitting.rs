use crate::{
    geometry::CoordinateScale,
    project_slice::perimeters::classic::materialize::{FittedArc, FittedMove, Point3, Polyline3},
};

pub(super) fn split_at_index(
    source: &Polyline3,
    index: usize,
    scale: CoordinateScale,
) -> (Polyline3, Polyline3) {
    let mut before_points = source.points[..=index].to_vec();
    let mut before_fitting = source
        .fitting
        .iter()
        .copied()
        .take_while(|fitted| fitted.start < index)
        .collect::<Vec<_>>();
    if let Some(fitted) = before_fitting.last_mut()
        && fitted.end > index
    {
        if let Some(arc) = &mut fitted.arc
            && !clip_arc_end(
                arc,
                source.points[fitted.start],
                &mut before_points[index],
                scale,
            )
        {
            fitted.arc = None;
        }
        fitted.end = index;
    }

    let mut after_points = source.points[index..].to_vec();
    let mut after_fitting = source
        .fitting
        .iter()
        .copied()
        .filter(|fitted| fitted.end > index)
        .collect::<Vec<_>>();
    if let Some(fitted) = after_fitting.first_mut()
        && fitted.start < index
        && let Some(arc) = &mut fitted.arc
        && !clip_arc_start(arc, &mut after_points[0], source.points[fitted.end], scale)
    {
        fitted.arc = None;
    }
    for fitted in &mut after_fitting {
        fitted.start = fitted.start.saturating_sub(index);
        fitted.end -= index;
    }

    (
        Polyline3 {
            points: before_points,
            fitting: before_fitting,
            candidate_points: Vec::new(),
        },
        Polyline3 {
            points: after_points,
            fitting: after_fitting,
            candidate_points: Vec::new(),
        },
    )
}

pub(super) fn append(polyline: &mut Polyline3, point: Point3) {
    if polyline.points.last() == Some(&point) {
        return;
    }
    polyline.points.push(point);
    let end = polyline.points.len() - 1;
    if let Some(last) = polyline.fitting.last_mut() {
        if last.arc.is_none() {
            last.end = end;
        } else {
            polyline.fitting.push(FittedMove {
                start: end - 1,
                end,
                arc: None,
            });
        }
    }
}

pub(super) fn prepend(polyline: &mut Polyline3, point: Point3) {
    if polyline.points.first() == Some(&point) {
        return;
    }
    polyline.points.insert(0, point);
    for fitted in &mut polyline.fitting {
        fitted.start += 1;
        fitted.end += 1;
    }
    if let Some(first) = polyline.fitting.first_mut() {
        if first.arc.is_none() {
            first.start = 0;
        } else {
            polyline.fitting.insert(
                0,
                FittedMove {
                    start: 0,
                    end: 1,
                    arc: None,
                },
            );
        }
    }
}

fn clip_arc_end(
    arc: &mut FittedArc,
    start: Point3,
    target: &mut Point3,
    scale: CoordinateScale,
) -> bool {
    let Some(projected) = project_to_circle(*arc, *target, scale) else {
        return false;
    };
    *target = projected;
    update_arc_length(arc, start, projected, scale)
}

fn clip_arc_start(
    arc: &mut FittedArc,
    target: &mut Point3,
    end: Point3,
    scale: CoordinateScale,
) -> bool {
    let Some(projected) = project_to_circle(*arc, *target, scale) else {
        return false;
    };
    *target = projected;
    update_arc_length(arc, projected, end, scale)
}

fn project_to_circle(arc: FittedArc, point: Point3, scale: CoordinateScale) -> Option<Point3> {
    let x = scale.unscale(point.x);
    let y = scale.unscale(point.y);
    let dx = x - arc.center.0;
    let dy = y - arc.center.1;
    let distance = dx.hypot(dy);
    if distance <= f64::EPSILON {
        return None;
    }
    Some(Point3 {
        x: ((arc.center.0 + dx * arc.radius / distance) / scale.factor()).round() as i64,
        y: ((arc.center.1 + dy * arc.radius / distance) / scale.factor()).round() as i64,
        z: point.z,
    })
}

fn update_arc_length(
    arc: &mut FittedArc,
    start: Point3,
    end: Point3,
    scale: CoordinateScale,
) -> bool {
    let start_angle =
        (scale.unscale(start.y) - arc.center.1).atan2(scale.unscale(start.x) - arc.center.0);
    let end_angle =
        (scale.unscale(end.y) - arc.center.1).atan2(scale.unscale(end.x) - arc.center.0);
    let sweep = if arc.clockwise {
        (start_angle - end_angle).rem_euclid(std::f64::consts::TAU)
    } else {
        (end_angle - start_angle).rem_euclid(std::f64::consts::TAU)
    };
    arc.length = arc.radius * sweep;
    arc.length > f64::EPSILON
}
