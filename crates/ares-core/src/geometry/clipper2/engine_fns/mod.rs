// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use crate::geometry::clipper2::core::*;
use crate::geometry::clipper2::engine::*;

mod is_odd;
pub(crate) use is_odd::*;
mod is_valid_ael_order;
pub(crate) use is_valid_ael_order::*;
mod path2_contains_path1_outpt;
pub(crate) use path2_contains_path1_outpt::*;
