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

/// The 0-based physical extruder of 0-based `filament_id`, or -1 when it may be any.
pub fn bed_exclusion_extruder_for_filament(
    _filament_id: usize,
    _map: &ExclusionFilamentMap<'_>,
    _extruder_count: usize,
) -> i32 {
    todo!("bed_exclusion::bed_exclusion_extruder_for_filament")
}
