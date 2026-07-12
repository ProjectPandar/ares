use std::collections::BTreeMap;

use crate::SliceError;

use super::super::SliceOptions;
use super::helpers::invalid_value_message;

impl SliceOptions {
    pub fn validate_gcode_flavor_option(&self) -> Result<BTreeMap<String, String>, SliceError> {
        let mut errors = BTreeMap::new();
        let gcode_flavor = self.string_or_default("gcode_flavor")?;

        if !is_active_gcode_flavor(&gcode_flavor) {
            errors.insert(
                "gcode_flavor".to_owned(),
                invalid_value_message(gcode_flavor),
            );
        }

        Ok(errors)
    }
}

fn is_active_gcode_flavor(gcode_flavor: &str) -> bool {
    matches!(
        gcode_flavor,
        "marlin" | "klipper" | "reprapfirmware" | "repetier" | "marlin2"
    )
}
