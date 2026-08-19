use super::MotionBlock;

#[derive(Clone, Copy)]
struct PlannedBlock {
    distance: f64,
    acceleration: f64,
    cruise: f64,
    entry: f64,
    exit: f64,
    max_entry: f64,
    safe: f64,
    nominal_length: bool,
    recalculate: bool,
    direction: [f64; 4],
    axis_feedrate: [f64; 4],
}

pub(super) fn planned_times(blocks: &[MotionBlock]) -> Vec<f64> {
    let mut planned = Vec::with_capacity(blocks.len());
    for block in blocks {
        planned.push(prepare(block, planned.last().copied()));
    }
    if planned.is_empty() {
        return Vec::new();
    }

    for index in (0..planned.len().saturating_sub(1)).rev() {
        let next = planned[index + 1];
        let current = &mut planned[index];
        if current.entry != current.max_entry || next.recalculate {
            let entry = if current.nominal_length {
                current.max_entry
            } else {
                current.max_entry.min(max_allowable_speed(
                    current.acceleration,
                    next.entry,
                    current.distance,
                ))
            };
            if current.entry != entry {
                current.entry = entry;
                current.recalculate = true;
            }
        }
    }
    for index in 1..planned.len() {
        let previous = planned[index - 1];
        let current = &mut planned[index];
        if !previous.nominal_length && previous.entry < current.entry {
            let entry =
                max_allowable_speed(previous.acceleration, previous.entry, previous.distance);
            if entry < current.entry {
                current.entry = entry;
                current.recalculate = true;
            }
        }
    }
    for index in 0..planned.len().saturating_sub(1) {
        planned[index].exit = planned[index + 1].entry;
    }
    let last = planned.len() - 1;
    planned[last].exit = planned[last].safe;

    planned.into_iter().map(block_time).collect()
}

fn prepare(block: &MotionBlock, previous: Option<PlannedBlock>) -> PlannedBlock {
    let mut cruise = block.speed;
    if let Some(previous) = previous {
        let previous_xy = xy_unit(previous.direction);
        let current_xy = xy_unit(block.direction);
        if let (Some(previous_xy), Some(current_xy)) = (previous_xy, current_xy) {
            let difference = (current_xy[0] - previous_xy[0]).hypot(current_xy[1] - previous_xy[1]);
            if difference < 0.5 && difference > 0.000_01 {
                let dot = previous_xy[0] * current_xy[0] + previous_xy[1] * current_xy[1];
                let cross = previous_xy[0] * current_xy[1] - previous_xy[1] * current_xy[0];
                let angle = cross.atan2(dot);
                let sin_half = ((1.0 - angle.cos()) * 0.5).sqrt();
                let xy_distance = block.distance * block.direction[0].hypot(block.direction[1]);
                let radius = xy_distance * 0.5 / sin_half;
                cruise = cruise.min((block.acceleration * radius).sqrt());
            }
        }
    }

    let axis_feedrate = block.direction.map(|direction| cruise * direction);
    let safe = axis_feedrate
        .iter()
        .zip(block.jerk)
        .filter(|(feedrate, jerk)| feedrate.abs() > *jerk)
        .fold(cruise, |safe, (_, jerk)| safe.min(jerk));
    let max_entry = previous.map_or(safe, |previous| {
        junction_speed(previous, block, cruise, axis_feedrate, safe)
    });
    let allowable = max_allowable_speed(block.acceleration, safe, block.distance);
    let entry = max_entry.min(allowable);

    PlannedBlock {
        distance: block.distance,
        acceleration: block.acceleration,
        cruise,
        entry,
        exit: safe,
        max_entry,
        safe,
        nominal_length: cruise <= allowable,
        recalculate: true,
        direction: block.direction,
        axis_feedrate,
    }
}

fn junction_speed(
    previous: PlannedBlock,
    block: &MotionBlock,
    cruise: f64,
    axis_feedrate: [f64; 4],
    safe: f64,
) -> f64 {
    let direction = block.direction;
    let jerk = block.jerk;
    if previous.cruise <= 0.000_1 {
        return safe;
    }
    let previous_larger = previous.cruise > cruise;
    let smaller_factor = if previous_larger {
        cruise / previous.cruise
    } else {
        previous.cruise / cruise
    };
    let mut maximum = previous.cruise.min(cruise);
    let mut factor = 1.0;
    let mut limited = false;

    let mut exit = previous
        .direction
        .map(|direction| previous.cruise * direction);
    if previous_larger {
        exit.iter_mut().for_each(|value| *value *= smaller_factor);
    }
    let mut difference = [
        (cruise * direction[0] - exit[0]).abs(),
        (cruise * direction[1] - exit[1]).abs(),
        (cruise * direction[2] - exit[2]).abs(),
    ];
    for axis in 0..3 {
        if difference[axis] > jerk[axis] {
            factor *= jerk[axis] / difference[axis];
            difference.iter_mut().for_each(|value| *value *= factor);
            limited = true;
        }
    }

    let mut e_exit = previous.axis_feedrate[3];
    let mut e_entry = axis_feedrate[3];
    if previous_larger {
        e_exit *= smaller_factor;
    }
    if limited {
        e_exit *= factor;
        e_entry *= factor;
    }
    let e_jerk = axis_jerk(e_exit, e_entry);
    if e_jerk > jerk[3] {
        factor *= jerk[3] / e_jerk;
        limited = true;
    }
    if limited {
        maximum *= factor;
    }
    if previous.safe > maximum * 0.99 && safe > maximum * 0.99 {
        safe
    } else {
        maximum
    }
}

fn axis_jerk(exit: f64, entry: f64) -> f64 {
    if exit > entry {
        if entry > 0.0 || exit < 0.0 {
            exit - entry
        } else {
            exit.max(-entry)
        }
    } else if entry < 0.0 || exit > 0.0 {
        entry - exit
    } else {
        (-exit).max(entry)
    }
}

fn xy_unit(direction: [f64; 4]) -> Option<[f64; 2]> {
    let norm = direction[0].hypot(direction[1]);
    (norm > 0.0).then(|| [direction[0] / norm, direction[1] / norm])
}

fn max_allowable_speed(acceleration: f64, target: f64, distance: f64) -> f64 {
    (target * target + 2.0 * acceleration * distance).sqrt()
}

fn block_time(block: PlannedBlock) -> f64 {
    let accelerate = ((block.cruise * block.cruise - block.entry * block.entry)
        / (2.0 * block.acceleration))
        .max(0.0);
    let decelerate = ((block.cruise * block.cruise - block.exit * block.exit)
        / (2.0 * block.acceleration))
        .max(0.0);
    let cruise_distance = block.distance - accelerate - decelerate;
    if cruise_distance >= 0.0 {
        (block.cruise - block.entry) / block.acceleration
            + cruise_distance / block.cruise.max(f64::MIN_POSITIVE)
            + (block.cruise - block.exit) / block.acceleration
    } else {
        let accelerate = ((2.0 * block.acceleration * block.distance - block.entry * block.entry
            + block.exit * block.exit)
            / (4.0 * block.acceleration))
            .clamp(0.0, block.distance);
        let peak = (block.entry * block.entry + 2.0 * block.acceleration * accelerate).sqrt();
        (peak - block.entry) / block.acceleration + (peak - block.exit) / block.acceleration
    }
}
