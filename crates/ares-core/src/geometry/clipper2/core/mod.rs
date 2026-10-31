// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

/// Get bounding rectangle for a single path - using existing implementation
/// Direct port from clipper.core.h line 432 (template function)
pub(crate) use get_bounds_path as get_bounds;
use num_traits::Num;
use std::fmt::Debug;

mod fillrule;
pub(crate) use fillrule::*;
mod new_with_validity;
pub(crate) use new_with_validity::*;
mod from;
pub(crate) use from::*;
mod get_bounds_paths;
pub(crate) use get_bounds_paths::*;
mod is_collinear;
pub(crate) use is_collinear::*;
mod transform_paths;
pub(crate) use transform_paths::*;
mod ellipse_point64;
pub(crate) use ellipse_point64::*;
