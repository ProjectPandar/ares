mod catalog;
pub(crate) mod compatible;
mod composition;
mod entry;
mod import;

mod fragment;
mod inheritance;

#[cfg(test)]
mod tests;

pub use catalog::{PresetCatalogHandle, SelectablePrinter};
pub use compatible::{FilteredPresets, MissingParent, filter_compatible_presets};
pub use composition::{
    ComposedProfile, ProfileGroupMetadata, ProfileSelection, compose_profile_fragments,
};
pub use entry::{
    Placement, ProfileSettings, ProfileSliceRequest, profile_settings, slice_with_profiles,
    slice_with_profiles_processed,
};
pub use fragment::{ProfileFragment, ProfileKind, merge_profile_fragments};
pub use import::{ParsedPresetBundle, parse_preset_bundle};
pub use inheritance::{MergedProfile, MergedProfileMetadata};
