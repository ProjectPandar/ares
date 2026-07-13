use serde::{Serialize, Serializer, ser::SerializeMap};

use super::ProjectPresetSourceOptions;

impl Serialize for ProjectPresetSourceOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(8))?;
        map.serialize_entry("default_filament_profile", &self.default_filament_profile)?;
        map.serialize_entry("filament_colour_type", &self.filament_colour_type)?;
        map.serialize_entry("filament_multi_colour", &self.filament_multi_colour)?;
        map.serialize_entry("filament_self_index", &self.filament_self_index)?;
        map.serialize_entry("filament_settings_id", &self.filament_settings_id)?;
        map.serialize_entry("print_compatible_printers", &self.print_compatible_printers)?;
        map.serialize_entry("print_settings_id", &self.print_settings_id)?;
        map.serialize_entry("printer_settings_id", &self.printer_settings_id)?;
        map.end()
    }
}
