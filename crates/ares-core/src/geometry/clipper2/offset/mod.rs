// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use crate::geometry::clipper2::FillRule;
use crate::geometry::clipper2::core::{
    Path64, PathD, Paths64, Point64, PointD, Rect64, area, constants, cross_product_two_vectors,
    distance_sqr, dot_product_two_vectors, ellipse_point64, get_line_intersect_pt_d, reflect_point,
    strip_duplicates_path, translate_point,
};
use crate::geometry::clipper2::engine::ClipType;
use crate::geometry::clipper2::engine::ZCallback64;
use crate::geometry::clipper2::engine_public::{Clipper64, PolyTree64};
use std::{cell::RefCell, rc::Rc};

mod floating_point_tolerance;
pub(crate) use floating_point_tolerance::*;
mod do_group_offset;
mod do_square;
mod set_arc_tolerance;
