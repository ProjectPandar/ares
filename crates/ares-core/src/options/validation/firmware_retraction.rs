use std::collections::BTreeMap;

use crate::SliceError;

use super::super::SliceOptions;

const FIRMWARE_RETRACTION_KEY: &str = "use_firmware_retraction";
const FIRMWARE_RETRACTION_SUPPORT_MESSAGE: &str = "--use-firmware-retraction is only supported by Klipper, Marlin, Smoothie, RepRapFirmware, Repetier and Machinekit firmware";
const FIRMWARE_RETRACTION_WIPE_MESSAGE: &str =
    "--use-firmware-retraction is not compatible with --wipe";

impl SliceOptions {
    pub fn validate_firmware_retraction_options(
        &self,
    ) -> Result<BTreeMap<String, String>, SliceError> {
        let mut errors = BTreeMap::new();
        let use_firmware_retraction = self.bool_or_default(FIRMWARE_RETRACTION_KEY)?;
        let gcode_flavor = self.string_or_default("gcode_flavor")?;
        let wipe = self.bool_vector_or_default("wipe")?;

        if use_firmware_retraction {
            if is_known_unsupported_firmware_retraction_flavor(&gcode_flavor) {
                errors.insert(
                    FIRMWARE_RETRACTION_KEY.to_owned(),
                    FIRMWARE_RETRACTION_SUPPORT_MESSAGE.to_owned(),
                );
            }

            if wipe.iter().any(|value| *value) {
                errors
                    .entry(FIRMWARE_RETRACTION_KEY.to_owned())
                    .or_insert_with(|| FIRMWARE_RETRACTION_WIPE_MESSAGE.to_owned());
            }
        }

        Ok(errors)
    }
}

fn is_known_unsupported_firmware_retraction_flavor(gcode_flavor: &str) -> bool {
    matches!(
        gcode_flavor,
        "teacup" | "makerware" | "sailfish" | "mach3" | "no-extrusion"
    )
}
