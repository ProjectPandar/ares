use serde::{Serialize, Serializer, ser::SerializeMap};

use super::FilamentRetractOverrideOptions;

impl Serialize for FilamentRetractOverrideOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(16))?;
        serialize_entries(&mut map, self)?;
        map.end()
    }
}

pub(crate) fn serialize_entries<M>(
    map: &mut M,
    value: &FilamentRetractOverrideOptions,
) -> Result<(), M::Error>
where
    M: SerializeMap,
{
    map.serialize_entry(
        "filament_deretraction_speed",
        &value.filament_deretraction_speed,
    )?;
    map.serialize_entry(
        "filament_long_retractions_when_cut",
        &value.filament_long_retractions_when_cut,
    )?;
    map.serialize_entry(
        "filament_retract_before_wipe",
        &value.filament_retract_before_wipe,
    )?;
    map.serialize_entry(
        "filament_retract_lift_above",
        &value.filament_retract_lift_above,
    )?;
    map.serialize_entry(
        "filament_retract_lift_below",
        &value.filament_retract_lift_below,
    )?;
    map.serialize_entry(
        "filament_retract_lift_enforce",
        &value.filament_retract_lift_enforce,
    )?;
    map.serialize_entry(
        "filament_retract_restart_extra",
        &value.filament_retract_restart_extra,
    )?;
    map.serialize_entry(
        "filament_retract_when_changing_layer",
        &value.filament_retract_when_changing_layer,
    )?;
    map.serialize_entry(
        "filament_retraction_distances_when_cut",
        &value.filament_retraction_distances_when_cut,
    )?;
    map.serialize_entry(
        "filament_retraction_length",
        &value.filament_retraction_length,
    )?;
    map.serialize_entry(
        "filament_retraction_minimum_travel",
        &value.filament_retraction_minimum_travel,
    )?;
    map.serialize_entry(
        "filament_retraction_speed",
        &value.filament_retraction_speed,
    )?;
    map.serialize_entry("filament_wipe", &value.filament_wipe)?;
    map.serialize_entry("filament_wipe_distance", &value.filament_wipe_distance)?;
    map.serialize_entry("filament_z_hop", &value.filament_z_hop)?;
    map.serialize_entry("filament_z_hop_types", &value.filament_z_hop_types)?;
    Ok(())
}
