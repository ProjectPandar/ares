mod composition_metadata;
mod composition_multi;
mod composition_single;
mod errors;
mod filament_variant_inheritance;
mod fragment_parsing;
mod inheritance;

use std::fmt::Debug;

use crate::{ProfileFragment, ProfileSelection, SliceError};

pub(super) fn fragment(input: &[u8]) -> ProfileFragment {
    ProfileFragment::from_json_bytes(input).unwrap()
}

pub(super) fn fragments<const N: usize>(inputs: [&[u8]; N]) -> Vec<ProfileFragment> {
    inputs.into_iter().map(fragment).collect()
}

pub(super) fn selection<const N: usize>(
    process: &str,
    machine: &str,
    filaments: [&str; N],
) -> ProfileSelection {
    ProfileSelection::new(process, machine, filaments).unwrap()
}

pub(super) fn assert_invalid<T: Debug>(result: Result<T, SliceError>, category: &str) {
    match result {
        Err(SliceError::InvalidInput(message)) => assert!(
            message.contains(category),
            "invalid-input message {message:?} did not identify {category:?}"
        ),
        other => panic!("expected InvalidInput for {category}, got {other:?}"),
    }
}
