use super::*;

pub(super) fn assert_fields(fixture: &Fixture) -> BTreeSet<&'static str> {
    assert_source_fields! {
        fixture, project;
        "deretraction_speed" => deretraction_speed,
        "filament_ids" => filament_ids,
        "filament_map_mode" => filament_map_mode,
        "filament_map" => filament_map,
        "retract_before_wipe" => retract_before_wipe,
        "retraction_length" => retraction_length,
        "retract_length_toolchange" => retract_length_toolchange,
        "z_hop" => z_hop,
        "retract_lift_above" => retract_lift_above,
        "retract_lift_below" => retract_lift_below,
        "retract_restart_extra" => retract_restart_extra,
        "retract_restart_extra_toolchange" => retract_restart_extra_toolchange,
        "retraction_speed" => retraction_speed,
        "nozzle_volume_type" => nozzle_volume_type,
        "extruder_ams_count" => extruder_ams_count,
        "bbl_calib_mark_logo" => bbl_calib_mark_logo,
        "has_scarf_joint_seam" => has_scarf_joint_seam,
    }
}
