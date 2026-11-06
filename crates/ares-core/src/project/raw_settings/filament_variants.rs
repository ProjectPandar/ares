//! `normalize_filament_values_to_variants` (`PrintConfig.cpp:11431-11457`).

use super::{ProjectSettingsRaw, RawConfigValue};
use crate::SliceError;

impl ProjectSettingsRaw {
    /// A per-variant filament option stored once per filament, or once for all of them, is laid
    /// out one value per filament variant of `filament_self_index`.
    pub(crate) fn normalize_filament_values_to_variants(&mut self) {
        let Some(RawConfigValue::List(self_index)) = self.0.get("filament_self_index") else {
            return;
        };
        let Some(ids) = self_index
            .iter()
            .map(|id| id.token()?.parse::<i32>().ok())
            .collect::<Option<Vec<_>>>()
        else {
            return;
        };
        let Some(&filament_count) = ids.iter().max() else {
            return;
        };
        if filament_count <= 0 || filament_count as usize >= ids.len() {
            return;
        }
        let filament_count = filament_count as usize;
        for key in crate::filament_options_with_variant() {
            let Some(RawConfigValue::List(values)) = self.0.get_mut(*key) else {
                continue;
            };
            if values.len() != filament_count && values.len() != 1 {
                continue;
            }
            // `map_variant_indices` (`PrintConfig.cpp:817-831`): a variant takes its filament's
            // value, a single source its only value; an id outside the filaments reads the
            // first value through `get_at`.
            let single = values.len() == 1;
            *values = ids
                .iter()
                .map(|&id| {
                    let index = usize::try_from(id - 1)
                        .ok()
                        .filter(|&index| !single && index < filament_count)
                        .unwrap_or(0);
                    values[index].clone()
                })
                .collect();
        }
    }

    /// `OrcaSlicer.cpp:4250-4259`: the CLI completes the config with the full print config
    /// defaults, then lays the per-variant filament options out again.
    pub(crate) fn complete_filament_variant_defaults(&mut self) -> Result<(), SliceError> {
        for key in crate::filament_options_with_variant() {
            if self.0.contains_key(*key) {
                continue;
            }
            let Some(definition) =
                crate::option_definition(key).filter(|definition| definition.full_config_default)
            else {
                continue;
            };
            let value = crate::options::json_deserialize::decode(
                definition.kind,
                definition.default_wire_value,
            )
            .map_err(SliceError::InvalidInput)?;
            self.0.insert((*key).to_owned(), value);
        }
        self.normalize_filament_values_to_variants();
        Ok(())
    }
}
