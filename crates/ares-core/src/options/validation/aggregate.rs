use std::collections::BTreeMap;

use crate::SliceError;

use super::super::SliceOptions;

impl SliceOptions {
    pub fn validate_fff_options(
        &self,
        under_cli: bool,
    ) -> Result<BTreeMap<String, String>, SliceError> {
        let mut errors = BTreeMap::new();

        merge_first(&mut errors, self.validate_basic_fdm_options()?);
        merge_first(&mut errors, self.validate_firmware_retraction_options()?);
        merge_first(&mut errors, self.validate_gcode_flavor_option()?);
        merge_first(&mut errors, self.validate_infill_pattern_options()?);
        merge_first(&mut errors, self.validate_skirt_and_bridge_flow_options()?);
        merge_first(&mut errors, self.validate_extruder_clearance_options()?);
        merge_first(&mut errors, self.validate_filament_flow_ratio_options()?);
        if under_cli {
            merge_first(&mut errors, self.validate_spiral_vase_cli_options()?);
        }
        merge_first(&mut errors, self.validate_extrusion_width_options()?);
        merge_first(&mut errors, self.validate_line_width_range_options()?);

        Ok(errors)
    }
}

fn merge_first(target: &mut BTreeMap<String, String>, source: BTreeMap<String, String>) {
    for (key, value) in source {
        target.entry(key).or_insert(value);
    }
}
