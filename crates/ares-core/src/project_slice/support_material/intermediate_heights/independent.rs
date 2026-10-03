use super::{EPSILON, Extreme, HeightPlanning, IntermediateLayer, Interval, SupportLayerType};

impl HeightPlanning<'_> {
    pub(super) fn independent(&mut self, mut interval: Interval) {
        let parameters = self.placement.parameters;
        let maximum = parameters.max_support_layer_height;
        let mut distance = interval.upper_z - interval.lower_z;
        let mut count = ((distance - EPSILON) / maximum).ceil() as usize;
        let mut step = distance / count as f64;
        if let Some(Extreme::Top(index)) = interval.lower
            && self.tops[index].print_z + self.placement.support_layer_height_min
                > self.tops[index].bottom_z + step
        {
            let top = &self.tops[index];
            self.layers.push(IntermediateLayer {
                layer_type: SupportLayerType::Intermediate,
                print_z: top.print_z,
                bottom_z: top.bottom_z,
                height: top.height,
                polygons: Vec::new(),
            });
            interval.lower_z = top.print_z;
            distance = interval.upper_z - interval.lower_z;
            count = ((distance - EPSILON) / maximum).ceil() as usize;
            if count == 0 {
                return;
            }
            step = distance / count as f64;
        }
        if !parameters.zero_gap_interface_top
            && let Extreme::Top(index) = interval.upper
        {
            let top = &mut self.tops[index];
            top.height = step;
            top.bottom_z = top.print_z - step;
            interval.upper_z = top.bottom_z;
            count -= 1;
            if count == 0 {
                return;
            }
        }
        for index in 0..count {
            let (bottom_z, print_z, height) = if index + 1 == count {
                let bottom_z = match index {
                    0 => interval.lower_z,
                    _ => self.layers.last().unwrap().print_z,
                };
                (bottom_z, interval.upper_z, interval.upper_z - bottom_z)
            } else {
                let bottom_z = interval.lower_z + index as f64 * step;
                (bottom_z, bottom_z + step, step)
            };
            self.layers.push(IntermediateLayer {
                layer_type: SupportLayerType::Intermediate,
                print_z,
                bottom_z,
                height,
                polygons: Vec::new(),
            });
        }
    }
}
