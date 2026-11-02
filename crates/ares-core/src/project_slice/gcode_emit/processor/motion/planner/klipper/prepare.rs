//! Klipper branches of `process_G1:5453-5663` and junction helpers
//! `GCodeProcessor.cpp:7787-7857` at the pinned upstream revision.
use super::{Block, MotionBlock};

pub(super) fn block(
    motion: &MotionBlock,
    previous: Option<Block>,
    queue_previous: Option<Block>,
) -> Block {
    let distance = motion.distance as f32;
    let delta = motion.delta;
    let virtual_motion = motion.virtual_motion;
    let mut direction = [delta[0] as f32, delta[1] as f32, delta[2] as f32, 0.0];
    let norm = (direction[0] * direction[0]
        + (direction[1] * direction[1] + direction[2] * direction[2]))
        .sqrt();
    if !motion.e_only {
        for component in &mut direction[..3] {
            *component /= norm;
        }
    }
    let mut cruise = motion.speed as f32;
    if virtual_motion && let Some(previous) = previous {
        cruise = super::super::centripetal::limit(
            delta,
            previous.direction,
            direction,
            motion.centripetal_acceleration,
            cruise,
        );
    }
    if !virtual_motion && !motion.e_only && motion.max_feedrate[0] > 0.0 {
        cruise = cruise.min(motion.max_feedrate[0] as f32);
    }
    let limit_extruder = virtual_motion || delta[3] < 0.0 || (delta[0] == 0.0 && delta[1] == 0.0);
    let inverse = 1.0 / distance;
    let mut axis_feedrate = delta.map(|axis| f64::from(cruise) * axis * f64::from(inverse));
    axis_feedrate[3] *= f64::from(motion.extrude_factor as f32);
    let mut factor = 1.0_f32;
    for (axis, feedrate) in axis_feedrate.iter().enumerate() {
        if !virtual_motion && axis < 2 {
            continue;
        }
        let speed = feedrate.abs();
        let limit = motion.max_feedrate[axis] as f32;
        if speed != 0.0 && (axis != 3 || limit_extruder) && limit != 0.0 {
            factor = factor.min((f64::from(limit) / speed) as f32);
        }
    }
    cruise *= factor;
    if factor < 1.0 {
        for speed in &mut axis_feedrate {
            *speed *= f64::from(factor);
        }
    }
    let toolhead_accel = motion.centripetal_acceleration as f32;
    let mut acceleration = if virtual_motion {
        motion.acceleration as f32
    } else if motion.e_only {
        let extrude_limit = motion.max_acceleration[3] as f32;
        if extrude_limit > 0.0 {
            extrude_limit
        } else {
            motion.acceleration as f32
        }
    } else {
        toolhead_accel
    };
    for (axis, delta) in delta.iter().enumerate() {
        let maximum = motion.max_acceleration[axis] as f32;
        if !virtual_motion && (axis < 2 || (axis == 3 && !limit_extruder) || maximum <= 0.0) {
            continue;
        }
        let component = delta.abs() * f64::from(inverse);
        if f64::from(acceleration) * delta.abs() * f64::from(inverse) > f64::from(maximum) {
            acceleration = (f64::from(maximum) / component) as f32;
        }
    }
    let scv = motion.jerk[0] as f32;
    let junction_deviation = if virtual_motion || scv <= 0.0 || toolhead_accel <= 0.0 {
        0.0
    } else {
        scv * scv * (2.0_f32.sqrt() - 1.0) / toolhead_accel
    };
    let cruise_accel = if motion.klipper.legacy() {
        toolhead_accel.min(motion.klipper.requested_accel_to_decel)
    } else {
        toolhead_accel * (1.0 - motion.klipper.minimum_cruise_ratio)
    };
    let mut block = Block {
        distance,
        acceleration,
        cruise,
        junction_deviation,
        axis_feedrate,
        direction,
        mcr_delta_v2: if virtual_motion {
            0.0
        } else {
            2.0 * distance * acceleration.min(cruise_accel)
        },
        ..Block::default()
    };
    if let (Some(previous), Some(queued)) = (previous, queue_previous)
        && previous.cruise > 0.0001
    {
        block.max_entry = junction(&block, &previous, &queued) as f32;
        if !virtual_motion {
            block.max_entry = block.max_entry.min(
                (queued.max_entry * queued.max_entry + 2.0 * queued.distance * queued.acceleration)
                    .sqrt(),
            );
            block.max_mcr_entry_sqr = (block.max_entry * block.max_entry)
                .min(queued.max_mcr_entry_sqr + queued.mcr_delta_v2);
        }
    }
    block
}

fn junction(block: &Block, previous: &Block, queued: &Block) -> f64 {
    if block.axis_feedrate[..3].iter().all(|axis| *axis == 0.0)
        || previous.axis_feedrate[..3].iter().all(|axis| *axis == 0.0)
    {
        return 0.0;
    }
    let cruise = f64::from(block.cruise);
    let prev_cruise = f64::from(previous.cruise);
    let mut limit = (cruise * cruise).min(prev_cruise * prev_cruise);
    let extrude_ratio_change =
        (block.axis_feedrate[3] / cruise - previous.axis_feedrate[3] / prev_cruise).abs();
    if extrude_ratio_change > 0.0 {
        let speed = 1.0 / extrude_ratio_change;
        limit = limit.min(speed * speed);
    }
    let direction = unit(block.axis_feedrate);
    let prev_direction = unit(previous.axis_feedrate);
    let cosine = (-(prev_direction[0] * direction[0]
        + (prev_direction[1] * direction[1] + prev_direction[2] * direction[2])))
        .clamp(-1.0, 1.0);
    let sin_half = (0.5 * (1.0 - cosine)).sqrt();
    let cos_half = (0.5 * (1.0 + cosine)).sqrt();
    if sin_half < 1.0 && cos_half > 0.0 {
        let radius = sin_half / (1.0 - sin_half);
        let half_tan = 0.5 * sin_half / cos_half;
        limit = limit
            .min(radius * f64::from(block.junction_deviation) * f64::from(block.acceleration))
            .min(radius * f64::from(queued.junction_deviation) * f64::from(queued.acceleration))
            .min(half_tan * f64::from(block.distance) * f64::from(block.acceleration))
            .min(half_tan * f64::from(queued.distance) * f64::from(queued.acceleration));
    }
    limit.sqrt()
}

fn unit(axis: [f64; 4]) -> [f64; 3] {
    let norm = (axis[0] * axis[0] + (axis[1] * axis[1] + axis[2] * axis[2])).sqrt();
    [axis[0] / norm, axis[1] / norm, axis[2] / norm]
}
