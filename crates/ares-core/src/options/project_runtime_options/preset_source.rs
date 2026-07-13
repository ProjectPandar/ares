mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

use super::super::{
    OrcaInt, OrcaInts, OrcaString, OrcaStrings, option_group::declare_option_group,
};

declare_option_group! {
    pub struct ProjectPresetSourceOptions, ProjectPresetSourceOptionsBuilder {
        print_compatible_printers => "print_compatible_printers": OrcaStrings = OrcaStrings::default(),
        default_filament_profile => "default_filament_profile": OrcaStrings = OrcaStrings::default(),
        filament_multi_colour => "filament_multi_colour": OrcaStrings = strings(&[""]),
        filament_colour_type => "filament_colour_type": OrcaStrings = strings(&["1"]),
        filament_settings_id => "filament_settings_id": OrcaStrings = strings(&[""]),
        print_settings_id => "print_settings_id": OrcaString = OrcaString::default(),
        printer_settings_id => "printer_settings_id": OrcaString = OrcaString::default(),
        filament_self_index => "filament_self_index": OrcaInts = ints(&[1]),
    }
}

impl ProjectPresetSourceOptions {
    pub const DECLARATION_ORDER: [&'static str; 8] = [
        "print_compatible_printers",
        "default_filament_profile",
        "filament_multi_colour",
        "filament_colour_type",
        "filament_settings_id",
        "print_settings_id",
        "printer_settings_id",
        "filament_self_index",
    ];
}

impl Default for ProjectPresetSourceOptions {
    fn default() -> Self {
        ProjectPresetSourceOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for ProjectPresetSourceOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(PresetSourceVisitor)
    }
}

struct PresetSourceVisitor;

impl<'de> Visitor<'de> for PresetSourceVisitor {
    type Value = ProjectPresetSourceOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca preset project options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = ProjectPresetSourceOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::unknown_field(
                    &key,
                    &ProjectPresetSourceOptions::DECLARATION_ORDER,
                ));
            }
        }
        Ok(builder.resolve())
    }
}

fn ints(values: &[i32]) -> OrcaInts {
    OrcaInts(values.iter().copied().map(OrcaInt).collect())
}

fn strings(values: &[&str]) -> OrcaStrings {
    OrcaStrings(values.iter().map(|value| (*value).to_owned()).collect())
}
