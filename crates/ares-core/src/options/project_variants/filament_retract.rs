//! Filament variant selection of the retract overrides (`PrintConfig.cpp:97-122`).

use crate::options::ProjectSettings;

use super::filament_select::select_filament;

pub(super) fn materialize(
    settings: &mut ProjectSettings,
    source: &ProjectSettings,
    indices: &[usize],
) {
    select_filament(
        &mut settings
            .filament
            .retract_overrides
            .filament_deretraction_speed,
        &source
            .filament
            .retract_overrides
            .filament_deretraction_speed,
        indices,
    );
    select_filament(
        &mut settings
            .filament
            .retract_overrides
            .filament_long_retractions_when_cut,
        &source
            .filament
            .retract_overrides
            .filament_long_retractions_when_cut,
        indices,
    );
    select_filament(
        &mut settings
            .filament
            .retract_overrides
            .filament_retract_after_wipe,
        &source
            .filament
            .retract_overrides
            .filament_retract_after_wipe,
        indices,
    );
    select_filament(
        &mut settings
            .filament
            .retract_overrides
            .filament_retract_before_wipe,
        &source
            .filament
            .retract_overrides
            .filament_retract_before_wipe,
        indices,
    );
    select_filament(
        &mut settings
            .filament
            .retract_overrides
            .filament_retract_lift_above,
        &source
            .filament
            .retract_overrides
            .filament_retract_lift_above,
        indices,
    );
    select_filament(
        &mut settings
            .filament
            .retract_overrides
            .filament_retract_lift_below,
        &source
            .filament
            .retract_overrides
            .filament_retract_lift_below,
        indices,
    );
    select_filament(
        &mut settings
            .filament
            .retract_overrides
            .filament_retract_lift_enforce,
        &source
            .filament
            .retract_overrides
            .filament_retract_lift_enforce,
        indices,
    );
    select_filament(
        &mut settings
            .filament
            .retract_overrides
            .filament_retract_restart_extra,
        &source
            .filament
            .retract_overrides
            .filament_retract_restart_extra,
        indices,
    );
    select_filament(
        &mut settings
            .filament
            .retract_overrides
            .filament_retract_when_changing_layer,
        &source
            .filament
            .retract_overrides
            .filament_retract_when_changing_layer,
        indices,
    );
    select_filament(
        &mut settings
            .filament
            .retract_overrides
            .filament_retraction_distances_when_cut,
        &source
            .filament
            .retract_overrides
            .filament_retraction_distances_when_cut,
        indices,
    );
    select_filament(
        &mut settings
            .filament
            .retract_overrides
            .filament_retraction_length,
        &source.filament.retract_overrides.filament_retraction_length,
        indices,
    );
    select_filament(
        &mut settings
            .filament
            .retract_overrides
            .filament_retraction_minimum_travel,
        &source
            .filament
            .retract_overrides
            .filament_retraction_minimum_travel,
        indices,
    );
    select_filament(
        &mut settings
            .filament
            .retract_overrides
            .filament_retraction_speed,
        &source.filament.retract_overrides.filament_retraction_speed,
        indices,
    );
    select_filament(
        &mut settings.filament.retract_overrides.filament_wipe,
        &source.filament.retract_overrides.filament_wipe,
        indices,
    );
    select_filament(
        &mut settings.filament.retract_overrides.filament_wipe_distance,
        &source.filament.retract_overrides.filament_wipe_distance,
        indices,
    );
    select_filament(
        &mut settings.filament.retract_overrides.filament_z_hop,
        &source.filament.retract_overrides.filament_z_hop,
        indices,
    );
    select_filament(
        &mut settings.filament.retract_overrides.filament_z_hop_types,
        &source.filament.retract_overrides.filament_z_hop_types,
        indices,
    );
    select_filament(
        &mut settings
            .filament
            .retract_overrides
            .filament_retract_length_toolchange,
        &source
            .filament
            .retract_overrides
            .filament_retract_length_toolchange,
        indices,
    );
    select_filament(
        &mut settings
            .filament
            .retract_overrides
            .filament_retract_restart_extra_toolchange,
        &source
            .filament
            .retract_overrides
            .filament_retract_restart_extra_toolchange,
        indices,
    );
}
