use super::{MotionBlock, MotionKind};
mod centripetal;
mod dispatch;
mod klipper;
pub(in crate::project_slice::gcode_emit::processor) use dispatch::Planner;
mod junction;
mod trapezoid;
use junction::{junction_speed, max_allowable_speed};
use trapezoid::block_time;

#[derive(Clone, Copy)]
struct PlannedBlock {
    distance: f32,
    acceleration: f32,
    cruise: f32,
    entry: f32,
    exit: f32,
    max_entry: f32,
    safe: f32,
    time: f32,
    trapezoid: Trapezoid,
    nominal_length: bool,
    recalculate: bool,
    direction: [f32; 4],
    jd_direction: [f32; 4],
    axis_feedrate: [f64; 4],
}

/// `TimeBlock::trapezoid` distances (`GCodeProcessor.hpp:432-446`) that
/// place the actual-speed preview vertices (`GCodeProcessor.cpp:499-547`).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Trapezoid {
    pub(crate) distance: f32,
    pub(crate) accelerate_until: f32,
    pub(crate) decelerate_after: f32,
    pub(crate) cruise_feedrate: f32,
    /// `TimeBlock::feedrate_profile.{entry, exit}`.
    pub(crate) entry: f32,
    pub(crate) exit: f32,
}

struct JunctionInput {
    direction: [f32; 4],
    jerk: [f32; 4],
    cruise: f32,
    axis_feedrate: [f64; 4],
    safe: f32,
}

pub(crate) struct RollingPlanner {
    blocks: Vec<PlannedBlock>,
    previous: Option<PlannedBlock>,
}

impl RollingPlanner {
    pub(crate) fn new() -> Self {
        Self {
            blocks: Vec::new(),
            previous: None,
        }
    }

    pub(crate) fn push(&mut self, block: &MotionBlock) -> Vec<(f64, Trapezoid)> {
        let planned = prepare(block, self.previous, !self.blocks.is_empty());
        // ToolChange inserts a time block without updating machine.prev.
        if block.kind != MotionKind::ToolChange {
            self.previous = Some(planned);
        }
        self.blocks.push(planned);
        if self.blocks.len() > 256 {
            self.process(64)
        } else {
            Vec::new()
        }
    }

    pub(crate) fn flush(&mut self) -> Vec<(f64, Trapezoid)> {
        if self.blocks.len() < 2 {
            Vec::new()
        } else {
            self.process(0)
        }
    }

    /// Final `calculate_time` pass (`GCodeProcessor.cpp:419-431` at
    /// `finalize`, :2699 keep=0, `is_final`): shares the `blocks.size() < 2`
    /// guard — a lone trailing block (e.g. an end-gcode `G1 E-1` after the
    /// last tool-change flush) never reaches `machine.time` — unless extra
    /// time is still buffered (`drain_final`), which drains the lone block.
    pub(crate) fn finish(&mut self, delay_buffered: bool) -> Vec<(f64, Trapezoid)> {
        if self.blocks.is_empty() || self.blocks.len() < 2 && !delay_buffered {
            Vec::new()
        } else {
            self.process(0)
        }
    }

    fn process(&mut self, keep: usize) -> Vec<(f64, Trapezoid)> {
        plan(&mut self.blocks);
        let process_len = self.blocks.len() - keep;
        let times = self.blocks[..process_len]
            .iter()
            .map(|block| (f64::from(block.time), block.trapezoid))
            .collect();
        self.blocks.drain(..process_len);
        if let Some(first) = self.blocks.first_mut() {
            first.max_entry = first.entry;
        }
        times
    }
}

fn plan(blocks: &mut [PlannedBlock]) {
    if blocks.is_empty() {
        return;
    }

    for index in (0..blocks.len().saturating_sub(1)).rev() {
        let next = blocks[index + 1];
        let current = &mut blocks[index];
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
    for index in 1..blocks.len() {
        let previous = blocks[index - 1];
        let current = &mut blocks[index];
        if !previous.nominal_length && previous.entry < current.entry {
            let entry =
                max_allowable_speed(previous.acceleration, previous.entry, previous.distance);
            if entry < current.entry {
                current.entry = entry;
                current.recalculate = true;
            }
        }
    }
    for index in 0..blocks.len().saturating_sub(1) {
        if blocks[index].recalculate || blocks[index + 1].recalculate {
            blocks[index].exit = blocks[index + 1].entry;
            (blocks[index].time, blocks[index].trapezoid) = block_time(blocks[index]);
            blocks[index].recalculate = false;
        }
    }
    let last = blocks.len() - 1;
    blocks[last].exit = blocks[last].safe;
    (blocks[last].time, blocks[last].trapezoid) = block_time(blocks[last]);
    blocks[last].recalculate = false;
}

fn prepare(
    block: &MotionBlock,
    previous: Option<PlannedBlock>,
    has_junction: bool,
) -> PlannedBlock {
    if block.kind == MotionKind::ToolChange {
        return PlannedBlock {
            distance: 0.0,
            acceleration: 0.0,
            cruise: 0.0,
            entry: 0.0,
            exit: 0.0,
            max_entry: 0.0,
            safe: 0.0,
            time: 0.0,
            trapezoid: Trapezoid::default(),
            nominal_length: false,
            recalculate: false,
            direction: [0.0; 4],
            jd_direction: [0.0; 4],
            axis_feedrate: [0.0; 4],
        };
    }
    let distance = block.distance as f32;
    let delta = block.delta;
    let jd_delta = delta.map(|component| component as f32);
    let jd_norm = ((jd_delta[0] * jd_delta[0] + jd_delta[2] * jd_delta[2])
        + (jd_delta[1] * jd_delta[1] + jd_delta[3] * jd_delta[3]))
        .sqrt();
    let jd_direction = jd_delta.map(|component| component / jd_norm);
    let jerk = block.jerk.map(|component| component as f32);
    let mut feedrate = block.speed as f32;
    // Enter direction (`GCodeProcessor.cpp:4000-4007`): the f32 delta
    // vector normalized by Eigen's f32 `norm()` unless the move is
    // extrusion-only; Eigen unrolls the reduction by halving
    // (`Redux.h` `redux_novec_unroller`), so the sum is `x² + (y² + z²)`.
    let mut direction = [delta[0] as f32, delta[1] as f32, delta[2] as f32, 0.0];
    let direction_norm = (direction[0] * direction[0]
        + (direction[1] * direction[1] + direction[2] * direction[2]))
        .sqrt();
    if !block.e_only {
        direction[0] /= direction_norm;
        direction[1] /= direction_norm;
        direction[2] /= direction_norm;
    }
    // Centripetal cruise limit (`GCodeProcessor.cpp:4016-4041`):
    // XY-projected unit vectors via the naive norm, the junction angle
    // from double `atan2` cast to float, and the radius from the raw XY
    // deltas.
    if let Some(previous) = previous {
        feedrate = centripetal::limit(
            delta,
            previous.direction,
            direction,
            block.centripetal_acceleration,
            feedrate,
        );
    }
    // Axis feedrates (`GCodeProcessor.cpp:4034-4041`): `feedrate * delta
    // * inv_distance` multiplies in double with a single float store; the
    // E axis carries the M221 extrude-factor override (a bare `M221 S`
    // word reads as 0).
    let inv_distance = 1.0 / distance;
    let mut axis_feedrate = [
        f64::from(feedrate) * delta[0] * f64::from(inv_distance),
        f64::from(feedrate) * delta[1] * f64::from(inv_distance),
        f64::from(feedrate) * delta[2] * f64::from(inv_distance),
        f64::from(feedrate) * delta[3] * f64::from(inv_distance),
    ];
    axis_feedrate[3] *= f64::from(block.extrude_factor as f32);
    let mut abs_axis_feedrate = axis_feedrate.map(f64::abs);
    // `min_feedrate_factor` and the axis-limited cruise
    // (`GCodeProcessor.cpp:4043-4053`).
    let mut factor = 1.0_f32;
    for (&abs_feedrate, &max_feedrate) in abs_axis_feedrate.iter().zip(&block.max_feedrate) {
        if abs_feedrate != 0.0 {
            let max = max_feedrate as f32;
            if max != 0.0 {
                factor = factor.min((f64::from(max) / abs_feedrate) as f32);
            }
        }
    }
    feedrate *= factor;
    let cruise = feedrate;
    if factor < 1.0 {
        for (feedrate, abs_feedrate) in axis_feedrate.iter_mut().zip(&mut abs_axis_feedrate) {
            *feedrate *= f64::from(factor);
            *abs_feedrate *= f64::from(factor);
        }
    }
    // Acceleration axis limits (`GCodeProcessor.cpp:4065-4071`).
    let mut acceleration = block.acceleration as f32;
    for (&max_acceleration, &axis_delta) in block.max_acceleration.iter().zip(&delta) {
        let max = max_acceleration as f32;
        let inv_distance = f64::from(inv_distance);
        if f64::from(acceleration) * axis_delta.abs() * inv_distance > f64::from(max) {
            acceleration = (f64::from(max) / (axis_delta.abs() * inv_distance)) as f32;
        }
    }
    // `process_G1:5544-5564`: positive Marlin junction deviation uses a
    // zero safe feedrate and its own junction-speed calculation; non-positive
    // junction deviation keeps the classic jerk path unchanged.
    let junction_deviation = junction::junction_deviation_speed(
        block,
        previous.filter(|previous| has_junction && previous.cruise > 0.0001),
        jd_direction,
        cruise,
        acceleration,
    );
    let safe = if junction_deviation.is_some() {
        0.0
    } else {
        let mut safe = cruise;
        for axis in 0..4 {
            if abs_axis_feedrate[axis] > f64::from(jerk[axis]) {
                safe = safe.min(jerk[axis]);
            }
        }
        safe
    };
    let max_entry = junction_deviation.unwrap_or_else(|| {
        if has_junction && let Some(previous) = previous {
            junction_speed(
                previous,
                JunctionInput {
                    direction,
                    jerk,
                    cruise,
                    axis_feedrate,
                    safe,
                },
            )
        } else {
            safe
        }
    });
    let allowable = max_allowable_speed(acceleration, safe, distance);
    let entry = max_entry.min(allowable);

    let mut planned = PlannedBlock {
        distance,
        acceleration,
        cruise,
        entry,
        exit: safe,
        max_entry,
        safe,
        time: 0.0,
        trapezoid: Trapezoid::default(),
        nominal_length: cruise <= allowable,
        recalculate: true,
        direction,
        jd_direction,
        axis_feedrate,
    };
    (planned.time, planned.trapezoid) = block_time(planned);
    planned
}
