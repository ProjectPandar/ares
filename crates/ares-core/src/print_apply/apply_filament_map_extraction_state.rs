#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedFilamentMapExtraction {
    pub(super) filament_maps: Vec<i32>,
}

pub(super) fn staged_apply_filament_map_extraction(
    values: Option<&[i32]>,
) -> StagedFilamentMapExtraction {
    StagedFilamentMapExtraction {
        filament_maps: values.unwrap_or_default().to_vec(),
    }
}
