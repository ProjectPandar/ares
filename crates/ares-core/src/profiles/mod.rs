mod catalog;
mod composition;
mod entry;

pub use entry::{ProfileSliceRequest, slice_with_profiles};
mod fragment;
mod inheritance;

#[cfg(test)]
mod tests;

pub use composition::{
    ComposedProfile, ProfileGroupMetadata, ProfileSelection, compose_profile_fragments,
};
pub use fragment::{ProfileFragment, ProfileKind, merge_profile_fragments};
pub use inheritance::{MergedProfile, MergedProfileMetadata};
