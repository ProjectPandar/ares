// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use crate::geometry::clipper2::core::*;
use crate::geometry::clipper2::engine_fns::*;
use crate::geometry::clipper2::engine_public::PolyTree64;
use std::collections::BinaryHeap;

mod none;
pub(crate) use none::*;
mod horzjoin;
pub(crate) use horzjoin::*;
mod add_local_max_poly;
mod add_trial_horz_join;
mod check_join_left;
mod convert_horz_segs_to_joins;
mod do_horizontal;
mod do_intersections;
mod execute_internal;
mod find_and_add_local_minima_closed;
mod fix_self_intersects;
mod insert_local_minima_into_ael;
mod intersect_edges;
mod is_contributing_open;
