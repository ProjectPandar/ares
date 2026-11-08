mod chain;
mod costs;
mod emit;
mod links;
mod neighbors;
mod path_matrix;
mod perimeter;
mod pinch;
mod regions;
pub(super) mod rng;
pub(crate) mod segments;
mod surface;
mod traverse;

pub(crate) use chain::chain_monotonic_regions;
pub(crate) use costs::compute_region_costs;
pub(crate) use emit::emit_monotonic_polylines;
pub(crate) use links::connect_contours;
pub(crate) use neighbors::connect_region_neighbors;
pub(crate) use pinch::insert_phony_outer_pairs;
pub(crate) use regions::{MonotonicRegion, generate_monotonic_regions};
pub(crate) use segments::remove_sticks_from_polygon;
pub(crate) use segments::{IntersectionKind, SegmentedLine};
pub(crate) use surface::{
    MonotonicFillOutput, MonotonicFillParams, fill_monotonic_surface, fill_rectilinear_surface,
};
