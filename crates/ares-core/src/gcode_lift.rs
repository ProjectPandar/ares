use crate::{Point2, gcode_writer::GCodeWriter};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TravelLiftMove {
    SlopeTop {
        point: Point2,
        z: f64,
    },
    Spiral {
        start: Point2,
        z_start: f64,
        z: f64,
        slope_radians: f64,
        resolution: f64,
        target: Point2,
    },
    Target {
        z: f64,
    },
}

pub(crate) fn distance(start: Point2, end: Point2) -> f64 {
    (end.x() - start.x()).hypot(end.y() - start.y())
}

pub(crate) fn slope_lift_move(
    writer: &GCodeWriter,
    target: Point2,
    z_hop: f64,
    slope_radians: f64,
    raised_z: f64,
) -> TravelLiftMove {
    let (x, y, _) = writer.current_position();
    let start = Point2::new(x, y);
    let travel_distance = distance(start, target);
    let slope_distance = z_hop / slope_radians.tan();
    if travel_distance <= f64::EPSILON || slope_distance >= travel_distance {
        return TravelLiftMove::Target { z: raised_z };
    }
    let ratio = slope_distance / travel_distance;
    TravelLiftMove::SlopeTop {
        point: Point2::new(
            start.x() + (target.x() - start.x()) * ratio,
            start.y() + (target.y() - start.y()) * ratio,
        ),
        z: raised_z,
    }
}

pub(crate) fn spiral_lift_move(
    writer: &GCodeWriter,
    target: Point2,
    slope_radians: f64,
    resolution: f64,
    raised_z: f64,
) -> TravelLiftMove {
    let (x, y, z_start) = writer.current_position();
    let start = Point2::new(x, y);
    if distance(start, target) <= f64::EPSILON {
        return TravelLiftMove::Target { z: raised_z };
    }
    TravelLiftMove::Spiral {
        start,
        z_start,
        z: raised_z,
        slope_radians,
        resolution,
        target,
    }
}
