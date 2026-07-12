use crate::{ExtrusionMove, Point2, PrintPathRole, SpeedOptions, ToolpathMoveKind};

const ORCA_LAYER_TIME_EPSILON: f64 = 1.001;
const SOLVE_ITERATIONS: usize = 20;

pub(super) struct LayerTimeSlowdown {
    options: SpeedOptions,
    last_point: Option<Point2>,
}

#[derive(Clone, Copy)]
struct LineTime {
    move_index: usize,
    length: f64,
    speed: f64,
    time: f64,
    adjustable: bool,
}

impl LayerTimeSlowdown {
    pub(super) const fn new(options: SpeedOptions) -> Self {
        Self {
            options,
            last_point: None,
        }
    }

    pub(super) fn apply(&mut self, moves: &[ExtrusionMove], mut speeds: Vec<f64>) -> Vec<f64> {
        let records = self.line_times(moves, &speeds);
        if !self.options.slow_down_for_layer_cooling()
            || self.options.slow_down_layer_time_s() <= 0.0
        {
            return speeds;
        }
        apply_slowdown(&self.options, &records, &mut speeds);
        speeds
    }

    fn line_times(&mut self, moves: &[ExtrusionMove], speeds: &[f64]) -> Vec<LineTime> {
        let mut records = Vec::new();
        let mut layer_had_extrusion = false;
        for (move_index, (move_, speed)) in moves.iter().zip(speeds.iter()).enumerate() {
            let start = self.last_point.unwrap_or(move_.point());
            let length = distance(start, move_.point());
            if move_.kind() == ToolpathMoveKind::Print {
                layer_had_extrusion = true;
            }
            if layer_had_extrusion && length > 0.0 && *speed > 0.0 {
                records.push(LineTime {
                    move_index,
                    length,
                    speed: *speed,
                    time: length / speed,
                    adjustable: is_adjustable(&self.options, move_),
                });
            }
            self.last_point = Some(move_.point());
        }
        records
    }
}

fn apply_slowdown(options: &SpeedOptions, records: &[LineTime], speeds: &mut [f64]) {
    let target_time = options.slow_down_layer_time_s() * ORCA_LAYER_TIME_EPSILON;
    let total_time = records.iter().map(|record| record.time).sum::<f64>();
    let adjustable = records
        .iter()
        .filter(|record| record.adjustable)
        .copied()
        .collect::<Vec<_>>();
    if adjustable.is_empty() || total_time >= target_time {
        return;
    }
    if options.slow_down_min_speed_mm_s() == 0.0 {
        slow_down_proportionally(&adjustable, target_time - total_time, speeds);
        return;
    }
    if maximum_time(records, options.slow_down_min_speed_mm_s()) <= target_time {
        slow_to_minimum(&adjustable, options.slow_down_min_speed_mm_s(), speeds);
        return;
    }
    let final_speed = equalized_feedrate(
        &adjustable,
        options.slow_down_min_speed_mm_s(),
        target_time - total_time,
    );
    for record in adjustable {
        if record.speed > final_speed {
            speeds[record.move_index] = final_speed;
        }
    }
}

fn slow_down_proportionally(adjustable: &[LineTime], stretch: f64, speeds: &mut [f64]) {
    let adjustable_time = adjustable.iter().map(|record| record.time).sum::<f64>();
    let factor = (adjustable_time + stretch) / adjustable_time;
    for record in adjustable {
        speeds[record.move_index] = record.speed / factor;
    }
}

fn maximum_time(records: &[LineTime], min_speed: f64) -> f64 {
    records
        .iter()
        .map(|record| {
            if record.adjustable && record.speed > min_speed {
                record.length / min_speed
            } else {
                record.time
            }
        })
        .sum()
}

fn slow_to_minimum(adjustable: &[LineTime], min_speed: f64, speeds: &mut [f64]) {
    for record in adjustable {
        if record.speed > min_speed {
            speeds[record.move_index] = min_speed;
        }
    }
}

fn equalized_feedrate(adjustable: &[LineTime], min_speed: f64, stretch: f64) -> f64 {
    let mut floor = min_speed;
    for _ in 0..SOLVE_ITERATIONS {
        let mut distance = 0.0;
        let mut time = stretch;
        for record in adjustable {
            if record.speed > floor {
                distance += record.length;
                time += record.time;
            }
        }
        let feedrate = distance / time;
        if feedrate <= floor {
            return floor;
        }
        if !adjustable
            .iter()
            .any(|record| record.speed > floor && record.speed < feedrate)
        {
            return feedrate;
        }
        floor = feedrate;
    }
    floor
}

fn is_adjustable(options: &SpeedOptions, move_: &ExtrusionMove) -> bool {
    move_.kind() == ToolpathMoveKind::Print
        && (!options.dont_slow_down_outer_wall()
            || move_.role() != PrintPathRole::ExternalPerimeter)
}

fn distance(start: Point2, end: Point2) -> f64 {
    ((end.x() - start.x()).powi(2) + (end.y() - start.y()).powi(2)).sqrt()
}

#[cfg(test)]
mod tests;
