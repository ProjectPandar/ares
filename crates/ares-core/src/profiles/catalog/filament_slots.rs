use super::{Catalog, PresetCatalogHandle, ProfileKind, invalid};
use crate::{SliceError, project::raw_settings::RawConfigValue};

impl Catalog {
    /// `filaments.first_visible().name`.
    pub(in crate::profiles) fn first_visible_filament(&self) -> &str {
        self.selectable_positions(ProfileKind::Filament)
            .into_iter()
            .find(|position| self.missing_parent(*position).is_none())
            .map(|position| self.document(position).name.as_str())
            .unwrap_or_default()
    }

    pub(in crate::profiles) fn filament_slot_counts(
        &self,
        printer: &str,
    ) -> Result<(usize, usize), SliceError> {
        let position = self
            .selectable_position(ProfileKind::Machine, printer)
            .filter(|position| self.missing_parent(*position).is_none())
            .ok_or_else(|| invalid(format!("printer preset '{printer}' was not found")))?;
        let values = self.resolved_values(
            position,
            &[
                "nozzle_diameter",
                "single_extruder_multi_material",
                "printer_technology",
            ],
        )?;
        let nozzles = match &values["nozzle_diameter"] {
            RawConfigValue::List(values) => values.len(),
            value => value.token().map_or(0, |value| {
                if value.is_empty() {
                    0
                } else {
                    value.split([',', ';']).count()
                }
            }),
        }
        .max(1);
        let semm = values["single_extruder_multi_material"]
            .token()
            .is_some_and(|value| value == "1" || value == "true");
        let fff = values["printer_technology"].token().as_deref() != Some("SLA");
        Ok((
            nozzles,
            if !fff {
                0
            } else if semm {
                1
            } else {
                nozzles
            },
        ))
    }
}

impl PresetCatalogHandle {
    /// `PresetBundle::update_multi_material_filament_presets`: pad the selection
    /// and resize project flush data for the selected printer.
    pub fn normalize_filament_slots(
        &self,
        printer: &str,
        mut state: crate::FilamentSlotState,
    ) -> Result<crate::FilamentSlotState, SliceError> {
        let (nozzles, minimum) = self.catalog.filament_slot_counts(printer)?;
        state.normalize(minimum, nozzles, self.catalog.first_visible_filament())?;
        Ok(state)
    }

    /// `PresetBundle::min_filament_count` (40ecc66f7, PresetBundle.cpp:5419-5436).
    /// SLA selections are unchanged by `update_filament_count`.
    pub fn min_filament_count(&self, printer: &str) -> Result<usize, SliceError> {
        self.catalog
            .filament_slot_counts(printer)
            .map(|(_, minimum)| minimum)
    }
}
