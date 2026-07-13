use serde::{Serialize, Serializer, ser::SerializeMap};

use super::FilamentRegionSourceOptions;

impl Serialize for FilamentRegionSourceOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(4))?;
        serialize_entries(&mut map, self)?;
        map.end()
    }
}

pub(crate) fn serialize_entries<M>(
    map: &mut M,
    value: &FilamentRegionSourceOptions,
) -> Result<(), M::Error>
where
    M: SerializeMap,
{
    map.serialize_entry("filament_ironing_flow", &value.filament_ironing_flow)?;
    map.serialize_entry("filament_ironing_inset", &value.filament_ironing_inset)?;
    map.serialize_entry("filament_ironing_spacing", &value.filament_ironing_spacing)?;
    map.serialize_entry("filament_ironing_speed", &value.filament_ironing_speed)?;
    Ok(())
}
