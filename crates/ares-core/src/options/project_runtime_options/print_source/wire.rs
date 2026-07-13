use serde::{Serialize, Serializer, ser::SerializeMap};

use super::ProjectPrintSourceOptions;

impl Serialize for ProjectPrintSourceOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(19))?;
        map.serialize_entry("curr_bed_type", &self.curr_bed_type)?;
        map.serialize_entry("extruder_colour", &self.extruder_colour)?;
        map.serialize_entry("extruder_offset", &self.extruder_offset)?;
        map.serialize_entry(
            "first_layer_print_sequence",
            &self.first_layer_print_sequence,
        )?;
        map.serialize_entry("flush_multiplier", &self.flush_multiplier)?;
        map.serialize_entry("flush_volumes_matrix", &self.flush_volumes_matrix)?;
        map.serialize_entry("flush_volumes_vector", &self.flush_volumes_vector)?;
        map.serialize_entry("max_layer_height", &self.max_layer_height)?;
        map.serialize_entry("min_layer_height", &self.min_layer_height)?;
        map.serialize_entry("nozzle_diameter", &self.nozzle_diameter)?;
        map.serialize_entry(
            "other_layers_print_sequence",
            &self.other_layers_print_sequence,
        )?;
        map.serialize_entry(
            "other_layers_print_sequence_nums",
            &self.other_layers_print_sequence_nums,
        )?;
        map.serialize_entry(
            "retract_when_changing_layer",
            &self.retract_when_changing_layer,
        )?;
        map.serialize_entry("retraction_minimum_travel", &self.retraction_minimum_travel)?;
        map.serialize_entry("start_end_points", &self.start_end_points)?;
        map.serialize_entry("wipe", &self.wipe)?;
        map.serialize_entry("wipe_distance", &self.wipe_distance)?;
        map.serialize_entry("wipe_tower_x", &self.wipe_tower_x)?;
        map.serialize_entry("wipe_tower_y", &self.wipe_tower_y)?;
        map.end()
    }
}
