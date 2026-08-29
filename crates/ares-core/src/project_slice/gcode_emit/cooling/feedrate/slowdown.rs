use crate::geometry::fixed_gcc_sort_by;

use super::{Config, CoolingLine};

const EPSILON: f64 = 1e-4;

pub(super) fn apply(lines: &mut [CoolingLine], config: Config) -> f32 {
    let total_time = elapsed_time(lines);
    let maximum_time = maximum_time(lines);
    if !config.enabled || lines.is_empty() {
        return total_time;
    }

    fixed_gcc_sort_by(lines, |left, right| {
        let left_adjustable = left.adjustable();
        let right_adjustable = right.adjustable();
        if left_adjustable == right_adjustable {
            left.feedrate > right.feedrate
        } else {
            left_adjustable
        }
    });
    let adjustable = lines.iter().take_while(|line| line.adjustable()).count();
    let target_time = config.target_time * 1.001;
    if total_time > target_time {
        return total_time;
    }
    if maximum_time > target_time {
        slow_down_non_proportional(
            lines,
            adjustable,
            config.minimum_speed,
            target_time - total_time,
        );
    } else {
        slow_down_to_minimum(lines);
    }
    elapsed_time(lines)
}

fn elapsed_time(lines: &[CoolingLine]) -> f32 {
    lines.iter().fold(0.0, |total, line| total + line.time)
}

fn maximum_time(lines: &[CoolingLine]) -> f32 {
    let mut total = 0.0;
    for line in lines {
        if line.adjustable() {
            if line.maximum_time == f32::MAX {
                return f32::MAX;
            }
            total += line.maximum_time;
        } else {
            total += line.time;
        }
    }
    total
}

fn slow_down_to_minimum(lines: &mut [CoolingLine]) {
    for line in lines {
        if line.adjustable() {
            line.slowed = true;
            line.time = line.maximum_time;
            if line.time > 0.0 {
                line.feedrate = line.length / line.time;
            }
        }
    }
}

fn slow_down_non_proportional(
    lines: &mut [CoolingLine],
    adjustable: usize,
    minimum_speed: f32,
    mut time_stretch: f32,
) {
    let mut begin = 0;
    let mut feedrate = lines[0].feedrate;
    loop {
        let mut end = begin;
        while end < adjustable && f64::from(lines[end].feedrate) > f64::from(feedrate) - EPSILON {
            end += 1;
        }
        let next_feedrate = lines
            .get(end..adjustable)
            .and_then(|remaining| remaining.first())
            .map_or(0.0, |line| line.feedrate);

        if minimum_speed == 0.0 {
            let adjustable_time = lines[..adjustable]
                .iter()
                .fold(0.0, |total, line| total + line.time);
            let factor = (adjustable_time + time_stretch) / adjustable_time;
            slow_down_proportionally(&mut lines[..adjustable], factor);
            return;
        }

        let mut feedrate_limit = next_feedrate.max(minimum_speed);
        let maximum_stretch = time_stretch_to_feedrate(&lines[..adjustable], feedrate_limit);
        let done = maximum_stretch >= time_stretch;
        if done {
            feedrate_limit = new_feedrate_to_reach_time_stretch(
                &lines[..adjustable],
                feedrate_limit,
                time_stretch,
            );
        } else {
            time_stretch -= maximum_stretch;
        }
        slow_down_to_feedrate(&mut lines[..adjustable], feedrate_limit);
        if done || next_feedrate == 0.0 {
            return;
        }
        begin = end;
        feedrate = next_feedrate;
    }
}

fn slow_down_proportionally(lines: &mut [CoolingLine], factor: f32) {
    for line in lines {
        if line.adjustable() {
            line.slowed = true;
            line.time = line.maximum_time.min(line.time * factor);
            if line.time > 0.0 {
                line.feedrate = line.length / line.time;
            }
        }
    }
}

fn time_stretch_to_feedrate(lines: &[CoolingLine], minimum_feedrate: f32) -> f32 {
    let mut stretch = 0.0;
    for line in lines {
        if line.feedrate > minimum_feedrate {
            stretch += line.time * (line.feedrate / minimum_feedrate - 1.0);
        }
    }
    stretch
}

fn slow_down_to_feedrate(lines: &mut [CoolingLine], minimum_feedrate: f32) {
    for line in lines {
        if line.feedrate > minimum_feedrate {
            line.time *= (line.feedrate / minimum_feedrate).max(1.0);
            line.feedrate = minimum_feedrate;
            line.slowed = true;
        }
    }
}

fn new_feedrate_to_reach_time_stretch(
    lines: &[CoolingLine],
    mut minimum_feedrate: f32,
    time_stretch: f32,
) -> f32 {
    let mut new_feedrate = minimum_feedrate;
    for _ in 0..20 {
        let mut numerator = 0.0_f64;
        let mut denominator = f64::from(time_stretch);
        for line in lines {
            if line.feedrate > minimum_feedrate {
                numerator += f64::from(line.time) * f64::from(line.feedrate);
                denominator += f64::from(line.time);
            }
        }
        new_feedrate = (numerator / denominator) as f32;
        if f64::from(new_feedrate) < f64::from(minimum_feedrate) + EPSILON {
            break;
        }
        if lines
            .iter()
            .any(|line| line.feedrate > minimum_feedrate && line.feedrate < new_feedrate)
        {
            minimum_feedrate = new_feedrate;
        } else {
            break;
        }
    }
    new_feedrate
}
