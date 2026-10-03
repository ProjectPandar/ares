//! Joint non-proportional range solver (`CoolingBuffer.cpp:258–306,559–634`).
use super::Adjustment;
use crate::geometry::fixed_gcc_sort_by;

const EPSILON: f64 = 1e-4;

pub(super) fn slow_down(adjustments: &mut [Adjustment], range: &[usize], mut stretch: f32) {
    let mut order = range.to_vec();
    let mut feedrate = 0.0_f32;
    for &index in &order {
        let adjustment = &mut adjustments[index];
        adjustment.begin = 0;
        adjustment.end = 0;
        if adjustment.adjustable > 0 {
            feedrate = feedrate.max(adjustment.lines[0].feedrate);
        }
    }
    fixed_gcc_sort_by(&mut order, |&left, &right| {
        adjustments[left].config.minimum_speed > adjustments[right].config.minimum_speed
    });
    loop {
        let mut next_feedrate = 0.0_f32;
        for &index in &order {
            let adjustment = &mut adjustments[index];
            adjustment.end = adjustment.begin;
            while adjustment.end < adjustment.adjustable
                && f64::from(adjustment.lines[adjustment.end].feedrate)
                    > f64::from(feedrate) - EPSILON
            {
                adjustment.end += 1;
            }
            if adjustment.end < adjustment.adjustable {
                next_feedrate = next_feedrate.max(adjustment.lines[adjustment.end].feedrate);
            }
        }
        let mut start = 0;
        while start < order.len() {
            let remaining = &order[start..];
            let minimum = adjustments[order[start]].config.minimum_speed;
            if minimum == 0.0 {
                slow_down_proportional(adjustments, remaining, stretch);
                return;
            }
            let mut limit = next_feedrate.max(minimum);
            let maximum_stretch = remaining.iter().fold(0.0_f32, |total, &index| {
                total + adjustments[index].stretch_to(limit)
            });
            let done = maximum_stretch >= stretch;
            if done {
                limit = new_feedrate(adjustments, remaining, limit, stretch);
            } else {
                stretch -= maximum_stretch;
            }
            for &index in remaining {
                adjustments[index].slow_to_feedrate(limit);
            }
            if done {
                return;
            }
            start += 1;
            while start < order.len()
                && f64::from(adjustments[order[start]].config.minimum_speed)
                    > f64::from(minimum) - EPSILON
            {
                start += 1;
            }
        }
        if next_feedrate == 0.0 {
            return;
        }
        for &index in &order {
            adjustments[index].begin = adjustments[index].end;
        }
        feedrate = next_feedrate;
    }
}

fn slow_down_proportional(adjustments: &mut [Adjustment], order: &[usize], stretch: f32) {
    let adjustable_time = order.iter().fold(0.0_f32, |total, &index| {
        total + adjustments[index].adjustable_time()
    });
    let factor = (adjustable_time + stretch) / adjustable_time;
    for &index in order {
        adjustments[index].proportional(factor);
    }
}

fn new_feedrate(
    adjustments: &[Adjustment],
    order: &[usize],
    mut minimum: f32,
    stretch: f32,
) -> f32 {
    let mut feedrate = minimum;
    for _ in 0..20 {
        let mut numerator = 0.0_f64;
        let mut denominator = f64::from(stretch);
        for &index in order {
            let adjustment = &adjustments[index];
            for line in adjustment.lines[..adjustment.adjustable]
                .iter()
                .filter(|line| line.feedrate > minimum)
            {
                numerator += f64::from(line.time) * f64::from(line.feedrate);
                denominator += f64::from(line.time);
            }
        }
        if denominator < 0.0 {
            return minimum;
        }
        feedrate = (numerator / denominator) as f32;
        if f64::from(feedrate) < f64::from(minimum) + EPSILON {
            break;
        }
        let unfinished = order.iter().any(|&index| {
            let adjustment = &adjustments[index];
            adjustment.lines[..adjustment.adjustable]
                .iter()
                .any(|line| line.feedrate > minimum && line.feedrate < feedrate)
        });
        if !unfinished {
            break;
        }
        minimum = feedrate;
    }
    feedrate
}
