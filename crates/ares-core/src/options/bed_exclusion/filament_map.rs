//! `bed_exclusion_extruder_for_filament` (`PrintConfig.cpp:14187-14209` at v2.5.0-beta
//! `345efec64`). See `docs/architecture/web-plater-arrange-imex.md`, "A2b3".

use crate::ProjectFilamentMapMode;

/// The map a filament resolves through: the configured `filament_map` (1-based extruders), its
/// mode, whether the printer is a Bambu one, and whether an automatic map has been resolved.
#[derive(Clone, Copy, Debug)]
pub struct ExclusionFilamentMap<'a> {
    pub configured: &'a [i32],
    pub mode: ProjectFilamentMapMode,
    pub is_bambu: bool,
    pub automatic_map_resolved: bool,
}

/// `is_auto_filament_map_mode` (`PrintConfig.hpp:649-651`): every mode before `fmmManual`.
pub(crate) fn is_auto_filament_map_mode(mode: ProjectFilamentMapMode) -> bool {
    match mode {
        ProjectFilamentMapMode::AutoForFlush | ProjectFilamentMapMode::AutoForMatch => true,
        ProjectFilamentMapMode::Manual => false,
    }
}

/// The 0-based physical extruder of 0-based `filament_id`, or -1 when it may be any.
pub fn bed_exclusion_extruder_for_filament(
    filament_id: usize,
    map: &ExclusionFilamentMap<'_>,
    extruder_count: usize,
) -> i32 {
    if extruder_count == 0 {
        return -1;
    }
    let automatic = is_auto_filament_map_mode(map.mode);
    if automatic && map.is_bambu && !map.automatic_map_resolved {
        return -1;
    }
    // Generic multi-tool printers use filament ids as tool ids; their global map still carries the
    // Bambu grouping seed [1, 1, ...].
    if automatic && !map.is_bambu && filament_id < extruder_count {
        return filament_id as i32;
    }
    if let Some(&extruder) = map.configured.get(filament_id)
        && extruder > 0
        && extruder as usize <= extruder_count
    {
        return extruder - 1;
    }
    if filament_id < extruder_count {
        filament_id as i32
    } else {
        -1
    }
}
