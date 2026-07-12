const PREVIOUS_COUNT_NAME: &str = "num_extruders";
const CURRENT_COUNT_SOURCE: &str = "m_config.filament_diameter.size()";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StagedExtruderCountChange {
    pub(super) previous_count_name: &'static str,
    pub(super) current_count_source: &'static str,
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
        previous_count_name: PREVIOUS_COUNT_NAME,
        current_count_source: CURRENT_COUNT_SOURCE,
        branch_taken,
        assigned_num_extruders: branch_taken.then_some(current_filament_diameter_count),
        num_extruders_changed: branch_taken,
    }
}
