use super::value::{Config, Value};
use crate::project::effective_config::types::ManualNozzleMaps;

// GCode.cpp:3421,6733,10900–10919: initialize and refresh the template maps
// from the published grouping, independently of the serialized config block.
pub(super) fn insert_maps(config: &mut Config, maps: Option<&ManualNozzleMaps>) {
    if let Some(maps) = maps {
        for (name, map) in [
            ("filament_map", &maps.extruders),
            ("filament_volume_map", &maps.volumes),
            ("filament_nozzle_map", &maps.nozzles),
        ] {
            config.insert(
                name,
                Value::List(
                    map.0
                        .iter()
                        .map(|id| Value::Integer(i64::from(id.0)))
                        .collect(),
                ),
            );
        }
    }
}
