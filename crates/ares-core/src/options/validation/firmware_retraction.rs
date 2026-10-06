use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{GCodeFlavor, ProjectSettings, SliceError};

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

        if let Some(message) = rejection_message(
            use_firmware_retraction,
            GCodeFlavor::deserialize(
                serde::de::value::StrDeserializer::<serde::de::value::Error>::new(&gcode_flavor),
            )
            .is_ok_and(is_known_unsupported_firmware_retraction_flavor),
            wipe.into_iter(),
        ) {
            errors.insert(FIRMWARE_RETRACTION_KEY.to_owned(), message.to_owned());
        }

        Ok(errors)
    }
}

// DynamicPrintConfig::validate / Slic3r::validate (PrintConfig.cpp:8866-8878,10391-10394).
impl ProjectSettings {
    /// The O2 rule's contribution to the shared `Slic3r::validate`
    /// `std::map` (keyed like the other loaded-config diagnostics).
    pub(crate) fn validate_loaded_retraction_map(
        &self,
    ) -> Result<BTreeMap<String, String>, SliceError> {
        let mut errors = BTreeMap::new();
        let unsupported =
            is_known_unsupported_firmware_retraction_flavor(self.printer.gcode.gcode_flavor);
        if let Some(message) = rejection_message(
            self.printer.gcode.use_firmware_retraction.0,
            unsupported,
            self.project.print.wipe.0.iter().map(|value| value.0),
        ) {
            errors.insert(FIRMWARE_RETRACTION_KEY.to_owned(), message.to_owned());
        }
        Ok(errors)
    }
}

fn rejection_message(
    firmware: bool,
    unsupported_flavor: bool,
    mut wipe: impl Iterator<Item = bool>,
) -> Option<&'static str> {
    if !firmware {
        return None;
    }
    if unsupported_flavor {
        return Some(FIRMWARE_RETRACTION_SUPPORT_MESSAGE);
    }
    wipe.any(|value| value)
        .then_some(FIRMWARE_RETRACTION_WIPE_MESSAGE)
}

fn is_known_unsupported_firmware_retraction_flavor(gcode_flavor: GCodeFlavor) -> bool {
    matches!(
        gcode_flavor,
        GCodeFlavor::Teacup
            | GCodeFlavor::MakerWare
            | GCodeFlavor::Sailfish
            | GCodeFlavor::Mach3
            | GCodeFlavor::NoExtrusion
    )
}
