use crate::arachne::extrusion_line::{ExtrusionJunction, ExtrusionLine};

use super::super::SkeletalTrapezoidation;

#[derive(Clone, Copy)]
pub(super) struct SegmentConditions {
    pub(super) is_odd: bool,
    pub(super) force_new_path: bool,
    pub(super) from_is_three_way: bool,
    pub(super) to_is_three_way: bool,
}

impl SkeletalTrapezoidation<'_> {
    pub(super) fn add_toolpath_segment(
        &mut self,
        from: ExtrusionJunction,
        to: ExtrusionJunction,
        conditions: SegmentConditions,
    ) {
        let SegmentConditions {
            is_odd,
            mut force_new_path,
            from_is_three_way,
            to_is_three_way,
        } = conditions;
        if from == to {
            return;
        }
        let inset_index = from.perimeter_index;
        if inset_index >= self.generated_toolpaths.len() {
            self.generated_toolpaths
                .resize_with(inset_index + 1, Vec::new);
        }
        let lines = &mut self.generated_toolpaths[inset_index];
        if lines.last().is_none_or(|line| {
            line.is_odd != is_odd || line.junctions.last().unwrap().perimeter_index != inset_index
        }) {
            force_new_path = true;
        }
        let tolerance = self.config.coordinate_scale.checked_scale(0.010).unwrap();
        if !force_new_path
            && compatible_endpoint(
                lines.last().unwrap().junctions.last().unwrap(),
                &from,
                tolerance,
            )
            && !from_is_three_way
        {
            lines.last_mut().unwrap().push(to);
            return;
        }
        if !force_new_path
            && compatible_endpoint(
                lines.last().unwrap().junctions.last().unwrap(),
                &to,
                tolerance,
            )
            && !to_is_three_way
        {
            lines.last_mut().unwrap().push(from);
            return;
        }
        let mut line = ExtrusionLine::new(inset_index, is_odd);
        line.push(from);
        line.push(to);
        lines.push(line);
    }
}

fn compatible_endpoint(
    endpoint: &ExtrusionJunction,
    junction: &ExtrusionJunction,
    tolerance: i64,
) -> bool {
    let dx = endpoint.point.x() - junction.point.x();
    let dy = endpoint.point.y() - junction.point.y();
    let distance_squared = dx as i128 * dx as i128 + dy as i128 * dy as i128;
    distance_squared < tolerance as i128 * tolerance as i128
        && endpoint.width.abs_diff(junction.width) < tolerance as u64
}

#[cfg(test)]
mod tests;
