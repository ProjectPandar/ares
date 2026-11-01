//! The filaments of an object and their physical extruders on a plate, for the arrange job's
//! bed-exclusion nozzle ids (`PartPlate.cpp` at v2.5.0-beta `345efec64`): `get_real_filament_maps`
//! / `get_real_filament_map_mode` (`:384-419`), `get_effective_exclusion_filament_maps`
//! (`:3540-3560`) and `get_object_filament_extruders` (`:3562-3654`). See
//! `docs/architecture/web-plater-arrange-imex.md`, "A2b3".

use std::collections::{BTreeMap, BTreeSet};

use super::PlateList;
use super::extruders::volume_extruders;
use crate::bed_exclusion::{
    ExclusionFilamentMap, bed_exclusion_extruder_for_filament, is_auto_filament_map_mode,
};
use crate::options::{OrcaInt, ProcessBrimType, ProjectFilamentMapMode, RegionOptionOverrides};
use crate::project_slice::gcode_emit::wipe_tower_estimate::config::DynamicPrintConfig;
use crate::{Project, ProjectObject};

/// The six feature filament keys and whether each feature prints (`:3590-3597`).
type FeatureKeys = [(
    fn(&RegionOptionOverrides) -> Option<OrcaInt>,
    &'static str,
    bool,
); 6];

/// `collect_config_filaments` (`:3598-3607`): the active feature ids and `extruder` one config sets.
fn collect_config_filaments(
    filament_ids: &mut BTreeSet<i32>,
    keys: &FeatureKeys,
    config: &RegionOptionOverrides,
) {
    for (get, _, active) in keys {
        if let Some(id) = get(config).filter(|id| *active && id.0 > 0) {
            filament_ids.insert(id.0);
        }
    }
    if let Some(id) = config.extruder.filter(|id| id.0 > 0) {
        filament_ids.insert(id.0);
    }
}

impl PlateList {
    /// `get_real_filament_maps` and `get_real_filament_map_mode`: the plate's own map and mode, else
    /// the global `filament_map` / `filament_map_mode`; plus whether the plate has its own map.
    fn real_filament_map(
        &self,
        index: usize,
        project: &Project,
    ) -> (ProjectFilamentMapMode, Vec<i32>, bool) {
        let (own_mode, own_map) = self.plates[index]
            .source
            .map_or((None, Vec::new()), |source| {
                project.plate_filament_map(source)
            });
        let global = &project.settings().project.gcode;
        let has_own_map = !own_map.is_empty();
        let map = if has_own_map {
            own_map
        } else {
            global.filament_map.0.iter().map(|id| id.0).collect()
        };
        (
            own_mode.unwrap_or(global.filament_map_mode),
            map,
            has_own_map,
        )
    }

    /// `get_real_filament_map_mode`.
    pub(in crate::plater) fn real_filament_map_mode(
        &self,
        index: usize,
        project: &Project,
    ) -> ProjectFilamentMapMode {
        self.real_filament_map(index, project).0
    }

    /// `get_effective_exclusion_filament_maps`: each filament's 1-based extruder, 0 when unresolved.
    fn effective_exclusion_filament_maps(
        &self,
        (index, project): (usize, &Project),
        filament_count: usize,
        (extruder_count, is_bambu): (usize, bool),
    ) -> Vec<i32> {
        let mut result = vec![0; filament_count];
        if extruder_count == 0 {
            return result;
        }
        let (mode, configured, has_own_map) = self.real_filament_map(index, project);
        let map = ExclusionFilamentMap {
            configured: &configured,
            mode,
            is_bambu,
            automatic_map_resolved: !is_auto_filament_map_mode(mode) || has_own_map,
        };
        for (filament_id, slot) in result.iter_mut().enumerate() {
            let extruder_id =
                bed_exclusion_extruder_for_filament(filament_id, &map, extruder_count);
            if extruder_id >= 0 {
                *slot = extruder_id + 1;
            }
        }
        result
    }

    /// `get_object_filament_extruders`: the object's 1-based filaments mapped to their 0-based
    /// extruders within `extruder_count`.
    pub(in crate::plater) fn object_filament_extruders(
        &self,
        (index, project, object): (usize, &Project, &ProjectObject),
        config: &DynamicPrintConfig,
        (extruder_count, is_bambu): (usize, bool),
    ) -> BTreeMap<i32, usize> {
        let options = object.object_overrides();
        let region = object.region_overrides();
        let brim_type = options.brim_type.map_or_else(
            || config.option("brim_type").key() != "no_brim",
            |brim| brim != ProcessBrimType::NoBrim,
        );
        let brim_width = options
            .brim_width
            .map_or_else(|| config.option("brim_width").float(), |w| w.0);
        let has_brim = brim_type && brim_width > 0.0;
        let int = |own: Option<OrcaInt>, key: &str| {
            own.map_or_else(|| config.option(key).int(), |v| i64::from(v.0))
        };
        let wall_loops = int(region.wall_loops, "wall_loops");
        let sparse_infill_density = region
            .sparse_infill_density
            .map_or_else(|| config.option("sparse_infill_density").float(), |d| d.0);
        let top_shell_layers = int(region.top_shell_layers, "top_shell_layers");
        let bottom_shell_layers = int(region.bottom_shell_layers, "bottom_shell_layers");
        let keys: FeatureKeys = [
            (
                |c| c.outer_wall_filament_id,
                "outer_wall_filament_id",
                wall_loops > 0 || has_brim,
            ),
            (
                |c| c.inner_wall_filament_id,
                "inner_wall_filament_id",
                wall_loops > 1,
            ),
            (
                |c| c.sparse_infill_filament_id,
                "sparse_infill_filament_id",
                sparse_infill_density > 0.0,
            ),
            (
                |c| c.internal_solid_filament_id,
                "internal_solid_filament_id",
                sparse_infill_density > 0.0 || top_shell_layers > 0 || bottom_shell_layers > 0,
            ),
            (
                |c| c.top_surface_filament_id,
                "top_surface_filament_id",
                top_shell_layers > 0,
            ),
            (
                |c| c.bottom_surface_filament_id,
                "bottom_surface_filament_id",
                bottom_shell_layers > 0,
            ),
        ];
        let mut filament_ids = BTreeSet::new();
        for volume in object.volumes() {
            filament_ids.extend(volume_extruders(object, volume));
            collect_config_filaments(&mut filament_ids, &keys, volume.region_overrides());
        }
        collect_config_filaments(&mut filament_ids, &keys, region);
        for range in object.layer_config_ranges() {
            collect_config_filaments(&mut filament_ids, &keys, range.region_overrides());
        }
        // Process-level feature assignments apply where the object does not override them.
        for (get, key, active) in &keys {
            let process = config.option(key).int();
            if *active && get(region).is_none() && process > 0 {
                filament_ids.insert(process as i32);
            }
        }
        let support = options
            .enable_support
            .map_or_else(|| config.option("enable_support").bool(), |e| e.0)
            || int(options.raft_layers, "raft_layers") > 0;
        if support {
            let supports = [
                int(options.support_filament, "support_filament"),
                int(
                    options.support_interface_filament,
                    "support_interface_filament",
                ),
            ];
            filament_ids.extend(
                supports
                    .into_iter()
                    .filter(|&id| id > 0)
                    .map(|id| id as i32),
            );
        }
        if filament_ids.is_empty() {
            filament_ids.insert(1);
        }
        let required = filament_ids.last().map_or(0, |&id| id as usize);
        let filament_map = self.effective_exclusion_filament_maps(
            (index, project),
            required,
            (extruder_count, is_bambu),
        );
        filament_ids
            .into_iter()
            .filter(|&id| id > 0)
            .filter_map(|id| {
                let mapped = filament_map
                    .get(id as usize - 1)
                    .filter(|&&extruder| extruder > 0)
                    .map_or(-1, |&extruder| extruder - 1);
                (mapped >= 0 && (mapped as usize) < extruder_count).then_some((id, mapped as usize))
            })
            .collect()
    }
}
