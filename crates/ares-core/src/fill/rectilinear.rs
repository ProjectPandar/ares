mod links;
mod pinch;
mod regions;
mod segments;

pub(crate) use links::connect_contours;
pub(crate) use pinch::insert_phony_outer_pairs;
pub(crate) use regions::generate_monotonic_regions;
pub(crate) use segments::{
    IntersectionKind, LinkQuality, LinkType, SegmentIntersection, SegmentedLine,
    slice_vertical_lines,
};

#[cfg(test)]
mod tests;
