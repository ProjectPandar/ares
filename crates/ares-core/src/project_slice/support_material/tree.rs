//! `TreeSupport.cpp` and `TreeSupport3D.cpp` source-owned support generators.

mod model_volumes;
mod necessity;
pub(super) use necessity::is_support_necessary;
mod non_organic;
pub(super) use non_organic::VerticalEnforcer;
pub(super) use non_organic::generation::{NonOrganicInput, generate as generate_non_organic};
mod organic;
pub(super) use organic::generation::{TreeInput, generate};
mod overhangs;
pub(super) mod remove_bridges;
