//! ToolOrdering.cpp:1778–1796 and MultiNozzleUtils.cpp:183–232.

use std::collections::{BTreeMap, btree_map::Entry};

use super::super::nozzle_group::{NozzleGroup, canonical_diameter};
use crate::options::{
    NozzleInventory, project_config_views::ProjectConfigViews, refresh_filament_variants,
};
use crate::project::{effective_config::types::ManualNozzleMaps, raw_settings::ProjectSettingsRaw};

use crate::{OrcaInt, OrcaInts, ProjectSettings, SliceError};

#[derive(Default)]
pub(super) struct Grouping {
    pub(super) groups: Vec<NozzleGroup>,
    pub(super) maps: ManualNozzleMaps,
    inventory: NozzleInventory,
}

impl Grouping {
    pub(super) fn resolve(
        settings: &ProjectSettings,
        raw: &ProjectSettingsRaw,
        used: &[usize],
    ) -> Self {
        let gcode = &settings.project.gcode;
        let count = settings.filament.gcode.filament_colour.0.len();
        if gcode.filament_volume_map.0.len() != count || gcode.filament_nozzle_map.0.len() != count
        {
            return Self::default();
        }
        let diameter = settings
            .project
            .print
            .nozzle_diameter
            .0
            .first()
            .map_or(0.4, |diameter| f64::from(diameter.0 as f32));
        let inventory = NozzleInventory::read(raw);
        let diameter = canonical_diameter(diameter);
        let groups = inventory
            .counts()
            .iter()
            .enumerate()
            .flat_map(|(extruder, stats)| {
                stats
                    .iter()
                    .flat_map(move |(&volume, &count)| (0..count).map(move |_| (extruder, volume)))
            })
            .enumerate()
            .map(|(group_id, (extruder, volume))| NozzleGroup {
                extruder,
                group_id,
                diameter,
                volume,
            })
            .collect();
        Self::assign(settings, used, groups, inventory).unwrap_or_default()
    }

    fn assign(
        settings: &ProjectSettings,
        used: &[usize],
        groups: Vec<NozzleGroup>,
        inventory: NozzleInventory,
    ) -> Option<Self> {
        let gcode = &settings.project.gcode;
        let count = gcode.filament_nozzle_map.0.len();
        if groups.is_empty() || count == 0 {
            return None;
        }
        let mut resolved = vec![0; count];
        let mut output_claimed = vec![false; groups.len()];
        let mut labels = BTreeMap::new();
        for &filament in used {
            let label = gcode.filament_nozzle_map.0.get(filament)?.0;
            let entry = match labels.entry(label) {
                Entry::Occupied(entry) => {
                    resolved[filament] = *entry.get();
                    continue;
                }
                Entry::Vacant(entry) => entry,
            };
            let extruder = (gcode.filament_map.0.get(filament)?.0 - 1) as usize;
            let volume = gcode.filament_volume_map.0[filament].0;
            let group = groups.iter().position(|nozzle| {
                !output_claimed[nozzle.group_id]
                    && nozzle.extruder == extruder
                    && nozzle.volume == volume
            })?;
            output_claimed[group] = true;
            entry.insert(group);
            resolved[filament] = group;
        }
        let maps = ManualNozzleMaps {
            extruders: OrcaInts(
                resolved
                    .iter()
                    .map(|&group| OrcaInt(groups[group].extruder as i32 + 1))
                    .collect(),
            ),
            volumes: OrcaInts(
                resolved
                    .iter()
                    .map(|&group| OrcaInt(groups[group].volume))
                    .collect(),
            ),
            nozzles: OrcaInts(
                resolved
                    .into_iter()
                    .map(|group| OrcaInt(group as i32))
                    .collect(),
            ),
        };
        Some(Self {
            groups,
            maps,
            inventory,
        })
    }

    // ToolOrdering.cpp:3064–3084 retains unused extruder/volume entries, but
    // publishes the complete derived nozzle map.
    pub(super) fn write_back(
        &self,
        checkpoint: &mut ProjectSettings,
        current: &ProjectSettings,
        used: &[usize],
    ) -> Result<Option<ProjectConfigViews>, SliceError> {
        if self.groups.is_empty() {
            return Ok(None);
        }
        let mut refreshed = current.clone();
        let config = &mut refreshed.project.gcode;
        let count = self.maps.nozzles.0.len();
        if config.filament_map.0.len() != count {
            config.filament_map = OrcaInts(vec![OrcaInt(1); count]);
        }
        for &filament in used {
            config.filament_map.0[filament] = self.maps.extruders.0[filament];
            config.filament_volume_map.0[filament] = self.maps.volumes.0[filament];
        }
        config.filament_nozzle_map.clone_from(&self.maps.nozzles);
        checkpoint
            .project
            .gcode
            .filament_map
            .clone_from(&config.filament_map);
        checkpoint
            .project
            .gcode
            .filament_volume_map
            .clone_from(&config.filament_volume_map);
        checkpoint
            .project
            .gcode
            .filament_nozzle_map
            .clone_from(&config.filament_nozzle_map);
        refresh_filament_variants(
            checkpoint,
            &refreshed.project.gcode.filament_map,
            &refreshed,
            Some(&self.inventory),
        )
        .map(Some)
    }
}
