//! A plate's filaments and the instances it holds whole (`PartPlate.cpp` at v2.5.0-beta
//! `345efec64`): `get_extruders(true, config, config)` (`:3075-3250`) with
//! `ModelVolume::get_extruders` (`Model.cpp:2690-2740`), and `contain_instance_totally` /
//! `contain_any_instance_totally` (`PartPlate.cpp:4494-4521`) over the outside test of
//! `check_outside` (`:4545-4625`).

use super::{InstanceKey, PlateList};
use crate::gcode_processor::CustomGCodeType;
use crate::plater::selection::instance_bounding_box;
use crate::project_slice::gcode_emit::wipe_tower_estimate::config::DynamicConfig;
use crate::project_slice::gcode_emit::wipe_tower_estimate::{
    expand_mixed_filaments, has_any_mixed_filament,
};
use crate::{Project, ProjectObject, ProjectVolume, ProjectVolumeType, SliceError};

/// `SINKING_Z_THRESHOLD` (`Model.hpp:1967`, a `float`).
const SINKING_Z_THRESHOLD: f64 = -0.001_f32 as f64;

/// `ModelVolume::get_extruders`: the painted filaments, then the volume's own one.
pub(in crate::plater) fn volume_extruders(
    object: &ProjectObject,
    volume: &ProjectVolume,
) -> Vec<i32> {
    if matches!(
        volume.volume_type(),
        ProjectVolumeType::NegativeVolume
            | ProjectVolumeType::SupportBlocker
            | ProjectVolumeType::SupportEnforcer
    ) {
        return Vec::new();
    }
    let used = volume.mesh().mmu_segmentation_facets.used_states();
    let mut extruders: Vec<i32> = (1..used.len())
        .filter(|&state| used[state])
        .map(|state| state as i32)
        .collect();
    // `extruder_id`: the volume's extruder, else the object's, else 1.
    let extruder = volume
        .region_overrides()
        .extruder
        .filter(|extruder| extruder.0 != 0)
        .or(object.region_overrides().extruder)
        .map_or(1, |extruder| extruder.0);
    if extruder > 0 {
        extruders.push(extruder);
    }
    extruders
}

/// An object override, else the global value, where 0 means unset.
fn role_filament(object: Option<i32>, global: i32) -> Option<i32> {
    match object.unwrap_or(0) {
        0 if global != 0 => Some(global),
        0 => None,
        id => Some(id),
    }
}

impl PlateList {
    /// `check_outside` without the bed exclusion and IMEX branches, which the tower path rejects
    /// before reaching here.
    fn outside(
        &self,
        index: usize,
        key: InstanceKey,
        project: &Project,
    ) -> Result<bool, SliceError> {
        let instance = instance_bounding_box(project, key);
        let (mut min, max) = self.plate_box(self.plates[index].origin);
        if instance.1[2] > min[2] {
            min[2] += instance.0[2];
        }
        if instance.0[2] < SINKING_Z_THRESHOLD {
            let intersects =
                (0..3).all(|axis| min[axis] < instance.1[axis] && max[axis] > instance.0[axis]);
            if !intersects {
                return Ok(true);
            }
            let origin = self.plates[index].origin;
            return crate::project_slice::instance_partly_outside(
                project,
                (origin[0], origin[1]),
                key,
            );
        }
        Ok(!(0..3).all(|axis| min[axis] <= instance.0[axis] && instance.1[axis] <= max[axis]))
    }

    /// `contain_instance_totally`.
    pub(in crate::plater) fn contains_totally(
        &self,
        index: usize,
        key: InstanceKey,
        project: &Project,
    ) -> Result<bool, SliceError> {
        Ok(self.plates[index].instances.contains(&key) && !self.outside(index, key, project)?)
    }

    /// `contain_any_instance_totally`.
    pub(in crate::plater) fn contains_any_totally(
        &self,
        index: usize,
        object: usize,
        project: &Project,
    ) -> Result<bool, SliceError> {
        for instance in 0..project.objects()[object].instances().len() {
            if self.contains_totally(index, (object, instance), project)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// `get_extruders(true, config, config)`: the 1-based filaments the plate's objects, their
    /// supports and its tool changes use, mixed slots expanded.
    pub(in crate::plater) fn plate_extruders(
        &self,
        index: usize,
        project: &Project,
        config: &DynamicConfig,
    ) -> Result<Vec<i32>, SliceError> {
        let mut extruders = Vec::new();
        if project.objects().is_empty() {
            return Ok(extruders);
        }
        let global = |key: &str| config.option(key).int() as i32;
        let (mut outer_wall, mut inner_wall) = (
            global("outer_wall_filament_id"),
            global("inner_wall_filament_id"),
        );
        if outer_wall == 0 {
            outer_wall = inner_wall;
        }
        if inner_wall == 0 {
            inner_wall = outer_wall;
        }
        let internal_solid = global("internal_solid_filament_id");
        let top_surface = match global("top_surface_filament_id") {
            0 => internal_solid,
            id => id,
        };
        let bottom_surface = match global("bottom_surface_filament_id") {
            0 => internal_solid,
            id => id,
        };
        let support = config.option("enable_support").bool() || global("raft_layers") > 0;
        for (object_index, object) in project.objects().iter().enumerate() {
            if !self.contains_any_totally(index, object_index, project)? {
                continue;
            }
            for volume in object.volumes() {
                extruders.extend(volume_extruders(object, volume));
            }
            extruders.extend(object.layer_config_ranges().iter().filter_map(|range| {
                range
                    .region_overrides()
                    .extruder
                    .filter(|id| id.0 > 0)
                    .map(|id| id.0)
            }));
            let options = object.object_overrides();
            let object_support =
                if options.enable_support.is_some() || options.raft_layers.is_some() {
                    options.enable_support.is_some_and(|enable| enable.0)
                        || options.raft_layers.is_some_and(|layers| layers.0 > 0)
                } else {
                    support
                };
            if object_support {
                extruders.extend(role_filament(
                    options.support_interface_filament.map(|id| id.0),
                    global("support_interface_filament"),
                ));
                extruders.extend(role_filament(
                    options.support_filament.map(|id| id.0),
                    global("support_filament"),
                ));
            }
            let region = object.region_overrides();
            let own = |value: Option<crate::options::OrcaInt>| value.map(|id| id.0);
            let outer = own(region.outer_wall_filament_id)
                .filter(|&id| id != 0)
                .or(own(region.inner_wall_filament_id));
            let inner = own(region.inner_wall_filament_id)
                .filter(|&id| id != 0)
                .or(own(region.outer_wall_filament_id));
            let solid = own(region.internal_solid_filament_id).unwrap_or(0);
            let top = own(region.top_surface_filament_id)
                .filter(|&id| id != 0)
                .unwrap_or(solid);
            let bottom = own(region.bottom_surface_filament_id)
                .filter(|&id| id != 0)
                .unwrap_or(solid);
            extruders.extend(role_filament(outer, outer_wall));
            extruders.extend(role_filament(inner, inner_wall));
            extruders.extend(role_filament(
                own(region.sparse_infill_filament_id),
                global("sparse_infill_filament_id"),
            ));
            extruders.extend(role_filament(Some(solid), internal_solid));
            extruders.extend(role_filament(Some(top), top_surface));
            extruders.extend(role_filament(Some(bottom), bottom_surface));
        }
        let filaments = config.option("filament_colour").strings().len() as i32;
        let gcodes = self.plates[index]
            .source
            .and_then(|source| project.plates_custom_gcodes().get(&(source as i32)));
        for item in gcodes.into_iter().flat_map(|info| &info.gcodes) {
            if item.kind == CustomGCodeType::ToolChange && item.extruder as i32 <= filaments {
                extruders.push(item.extruder as i32);
            }
        }
        extruders.sort_unstable();
        extruders.dedup();
        let is_mixed = config.option("filament_is_mixed");
        if has_any_mixed_filament(is_mixed.bools()) {
            let slots: Vec<u32> = extruders
                .iter()
                .filter(|&&id| id >= 1)
                .map(|&id| (id - 1) as u32)
                .collect();
            let components = config.option("filament_mixed_components");
            extruders = expand_mixed_filaments(&slots, is_mixed.bools(), components.strings())
                .into_iter()
                .map(|id| id as i32 + 1)
                .collect();
        }
        Ok(extruders)
    }
}
