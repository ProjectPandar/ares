mod catalog;
pub(crate) mod compatible;
mod composition;
pub(crate) mod entry;
mod settings;
mod source_identity;
pub use settings::{ProfileSettings, profile_settings};
pub use source_identity::SourceIdentity;
mod import;
mod load_settings;
mod preset_normalization;
mod project_refresh;
mod project_selection;
pub(crate) use project_refresh::{
    ProjectPresetPolicy, ProjectPresetRefresh, apply_retained_preset_defaults, preset_owns_option,
};
pub use project_selection::{
    PresetCollectionState, PresetConfig, PresetOutcome, PresetRecord, PresetSelection,
};

mod filament_slots;
pub use filament_slots::FilamentSlotState;
mod fragment;
mod inheritance;

pub use catalog::LoadedPresetSources;
pub(crate) use catalog::VendorType;
pub use catalog::{
    CurrentPrinter, GuideFilament, GuideModel, InstalledPrinters, PresetCatalogHandle,
    PresetValues, PrinterModelInfo, SelectablePrinter, guide_models,
};
pub use compatible::{FilteredPresets, MissingParent, filter_compatible_presets};
pub use composition::{
    ComposedProfile, ProfileGroupMetadata, ProfileSelection, compose_profile_fragments,
};
pub(crate) use entry::with_edited_profile_source;
pub use entry::{
    CliProfileSliceRequest, LoadSettingsSliceRequest, Placement, ProfileSliceRequest,
    profile_placed_project, slice_cli_with_profiles_validated, slice_with_load_settings,
    slice_with_profiles, slice_with_profiles_processed, slice_with_profiles_processed_validated,
    slice_with_profiles_validated, validate_load_settings_input, validate_with_profiles,
};
pub use fragment::{ProfileFragment, ProfileKind, merge_profile_fragments};
pub use import::{ParsedPresetBundle, parse_preset_bundle};
pub use inheritance::{MergedProfile, MergedProfileMetadata};
pub use load_settings::{LoadSettingsFile, LoadSettingsRequest};

pub(crate) fn without_utf8_bom(input: &[u8]) -> &[u8] {
    input.strip_prefix(b"\xef\xbb\xbf").unwrap_or(input)
}
