mod links;
mod neighbors;
mod pinch;
mod regions;
mod segments;

pub(crate) use links::connect_contours;
pub(crate) use neighbors::connect_region_neighbors;
pub(crate) use pinch::insert_phony_outer_pairs;
pub(crate) use regions::{MonotonicRegion, RegionBoundary, generate_monotonic_regions};
pub(crate) use segments::{
    IntersectionKind, LinkQuality, LinkType, SegmentIntersection, SegmentedLine,
    prepare_rectilinear_slice, slice_vertical_lines,
};

#[cfg(test)]
mod tests;
