//! `CoolingBuffer::calculate_layer_slowdown` (637–690).
use super::{Adjustment, range};
use crate::geometry::fixed_gcc_sort_by;

pub(super) fn apply(adjustments: &mut [Adjustment]) -> f32 {
    let mut elapsed = 0.0_f32;
    let mut order = Vec::new();
    for (index, adjustment) in adjustments.iter_mut().enumerate() {
        adjustment.total = adjustment.elapsed();
        adjustment.maximum = adjustment.maximum_time();
        if adjustment.config.enabled && !adjustment.lines.is_empty() {
            order.push(index);
            adjustment.sort();
        } else {
            elapsed += adjustment.elapsed();
        }
    }
    fixed_gcc_sort_by(&mut order, |&left, &right| {
        adjustments[left].config.target_time < adjustments[right].config.target_time
    });
    for start in 0..order.len() {
        let remaining = &order[start..];
        let total = remaining
            .iter()
            .fold(elapsed, |total, &index| total + adjustments[index].total);
        let target = adjustments[order[start]].config.target_time * 1.001_f32;
        if total > target {
            elapsed += adjustments[order[start]].elapsed();
            continue;
        }
        let maximum = remaining
            .iter()
            .fold(elapsed, |total, &index| total + adjustments[index].maximum);
        if maximum > target {
            range::slow_down(adjustments, remaining, target - total);
        } else {
            for &index in remaining {
                adjustments[index].slow_to_minimum();
            }
        }
        elapsed += adjustments[order[start]].elapsed();
    }
    elapsed
}
