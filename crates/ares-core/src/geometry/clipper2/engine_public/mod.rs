// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use crate::geometry::clipper2::core::*;
use crate::geometry::clipper2::engine::*;
use crate::geometry::clipper2::engine_fns::*;

mod polypath64;
pub(crate) use polypath64::*;
mod is_hole;
pub(crate) use is_hole::*;
