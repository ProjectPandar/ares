#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StagedExtruderCountChange {
    pub(super) branch_taken: bool,
    pub(super) assigned_num_extruders: Option<usize>,
    pub(super) num_extruders_changed: bool,
}

pub(super) fn staged_apply_extruder_count_change(
    previous_count: usize,
    current_filament_diameter_count: usize,
) -> StagedExtruderCountChange {
    let branch_taken = previous_count != current_filament_diameter_count;

    StagedExtruderCountChange {
        branch_taken,
        assigned_num_extruders: branch_taken.then_some(current_filament_diameter_count),
        num_extruders_changed: branch_taken,
    }
}
