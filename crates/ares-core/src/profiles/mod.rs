mod catalog;
pub(crate) mod compatible;
mod composition;
mod entry;
mod import;
mod load_settings;
mod project_refresh;
pub(crate) use project_refresh::ProjectPresetRefresh;

mod fragment;
mod inheritance;

pub(crate) use catalog::VendorType;
pub use catalog::{
    CurrentPrinter, PresetCatalogHandle, PresetValues, PrinterModelInfo, SelectablePrinter,
};
pub use compatible::{FilteredPresets, MissingParent, filter_compatible_presets};
pub use composition::{
    ComposedProfile, ProfileGroupMetadata, ProfileSelection, compose_profile_fragments,
};
pub(crate) use entry::with_edited_profile_source;
pub use entry::{
    CliProfileSliceRequest, LoadSettingsSliceRequest, Placement, ProfileSettings,
    ProfileSliceRequest, profile_placed_project, profile_settings,
    slice_cli_with_profiles_validated, slice_with_load_settings, slice_with_profiles,
    slice_with_profiles_processed, slice_with_profiles_processed_validated,
    slice_with_profiles_validated, validate_load_settings_input, validate_with_profiles,
};
pub use fragment::{ProfileFragment, ProfileKind, merge_profile_fragments};
pub use import::{ParsedPresetBundle, parse_preset_bundle};
pub use inheritance::{MergedProfile, MergedProfileMetadata};
pub use load_settings::{LoadSettingsFile, LoadSettingsRequest};
