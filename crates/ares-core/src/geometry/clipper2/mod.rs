#![forbid(unsafe_code)]

// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See LICENSE-Boost-1.0 and SOURCE.md.

mod core;
mod engine;
mod engine_fns;
mod engine_public;
mod offset;

pub(crate) use core::constants::MAX_COORD;
pub(crate) use core::{FillRule, Path64, Paths64, Point64, Rect64, area, get_bounds_path};
pub(crate) use engine::ClipType;
pub(crate) use engine_public::{Clipper64, PolyTree64};
pub(crate) use offset::{ClipperOffset, EndType, JoinType};
