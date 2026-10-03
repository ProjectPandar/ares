use super::{EPSILON, HeightPlanning, IntermediateLayer, Interval, SupportLayerType};

impl HeightPlanning<'_> {
    pub(super) fn synchronized(&mut self, interval: Interval) {
        let object_layers = self.placement.layers;
        let parameters = self.placement.parameters;
        while self.object_index < object_layers.len()
            && object_layers[self.object_index].print_z < interval.lower_z + EPSILON
        {
            self.object_index += 1;
        }
        if self.object_index == 0 && interval.lower_z == parameters.raft_interface_top_z {
            self.layers.push(IntermediateLayer {
                layer_type: SupportLayerType::Intermediate,
                print_z: parameters.object_print_z_min,
                bottom_z: parameters.raft_interface_top_z,
                height: parameters.object_print_z_min - parameters.raft_interface_top_z,
                polygons: Vec::new(),
            });
        }
        while self.object_index < object_layers.len()
            && object_layers[self.object_index].print_z < interval.upper_z + EPSILON
        {
            let object = &object_layers[self.object_index];
            self.layers.push(IntermediateLayer {
                layer_type: SupportLayerType::Intermediate,
                print_z: object.print_z,
                bottom_z: if self.object_index > 0 {
                    object_layers[self.object_index - 1].print_z
                } else {
                    object.print_z - object.height
                },
                height: object.height,
                polygons: Vec::new(),
            });
            self.object_index += 1;
        }
    }
}
