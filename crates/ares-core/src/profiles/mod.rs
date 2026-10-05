mod catalog;
pub(crate) mod compatible;
mod composition;
mod entry;
mod import;

mod fragment;
mod inheritance;

#[cfg(test)]
mod tests;

pub use compatible::filter_compatible_presets;
pub use composition::{
    ComposedProfile, ProfileGroupMetadata, ProfileSelection, compose_profile_fragments,
};
pub use entry::{ProfileSliceRequest, slice_with_profiles};
pub use fragment::{ProfileFragment, ProfileKind, merge_profile_fragments};
pub use import::parse_preset_bundle;
pub use inheritance::{MergedProfile, MergedProfileMetadata};
