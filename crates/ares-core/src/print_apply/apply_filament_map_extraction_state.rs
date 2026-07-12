const SOURCE_CONFIG: &str = "new_full_config";
const OPTION_KEY: &str = "filament_map";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedFilamentMapExtraction {
    pub(super) source_config: &'static str,
    pub(super) option_key: &'static str,
    pub(super) filament_maps: Vec<i32>,
}

pub(super) fn staged_apply_filament_map_extraction(
    values: Option<&[i32]>,
) -> StagedFilamentMapExtraction {
    StagedFilamentMapExtraction {
        source_config: SOURCE_CONFIG,
        option_key: OPTION_KEY,
        filament_maps: values.unwrap_or_default().to_vec(),
    }
}
