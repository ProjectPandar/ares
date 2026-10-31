// OrcaSlicer 345efec64: ClipperUtils.cpp wrappers over the single Clipper2 engine.
mod backend;
mod boolean_ex;
mod boolean_paths;
mod offset;
pub(crate) mod ordering;
mod point_in_polygon;
mod polyline;
mod polytree;
mod simplify;
mod variable_offset;
pub(in crate::geometry) mod z;
pub(crate) mod z_clip;

pub(crate) use crate::geometry::clipper2::FillRule;
pub(crate) use boolean_ex::*;
pub(crate) use boolean_paths::*;
pub(crate) use offset::*;
pub(crate) use point_in_polygon::point_in_polygon;
pub(crate) use polyline::*;
pub(crate) use polytree::{PolyTree, union_contours, union_ex};
pub(super) use simplify::simplify_polygons;
pub(crate) use variable_offset::variable_offset_inner_ex;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClipOperation {
    Intersection,
    Union,
    Difference,
    Xor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClipperError {
    CoordinateOutOfRange,
}
