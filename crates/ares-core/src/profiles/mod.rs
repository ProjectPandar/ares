mod composition;
mod fragment;

pub use composition::{ComposedProfile, ProfileSelection, compose_profile_fragments};
pub use fragment::{ProfileFragment, ProfileKind, merge_profile_fragments};
