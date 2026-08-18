mod chain;
mod costs;
mod emit;
mod links;
mod neighbors;
mod path_matrix;
mod perimeter;
mod pinch;
mod regions;
mod rng;
mod segments;
mod surface;

pub(crate) use chain::chain_monotonic_regions;
pub(crate) use costs::compute_region_costs;
pub(crate) use emit::emit_monotonic_polylines;
pub(crate) use links::connect_contours;
pub(crate) use neighbors::connect_region_neighbors;
pub(crate) use path_matrix::MonotonicPathMatrix;
pub(crate) use perimeter::{
    append_contour_segment, contour_segment_length, directed_segment_distance, emit_horizontal_arc,
    emit_vertical_arc, measure_horizontal_arc, measure_vertical_arc,
};
pub(crate) use pinch::insert_phony_outer_pairs;
pub(crate) use regions::{MonotonicRegion, RegionBoundary, generate_monotonic_regions};
#[cfg(test)]
pub(crate) use segments::remove_sticks_from_polygon;
pub(crate) use segments::{
    IntersectionKind, LinkQuality, LinkType, SegmentIntersection, SegmentedLine,
    prepare_rectilinear_slice, slice_vertical_lines,
};
pub(crate) use surface::{MonotonicFillParams, fill_monotonic_surface};

#[cfg(test)]
mod tests;
