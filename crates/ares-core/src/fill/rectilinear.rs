mod links;
mod segments;

pub(crate) use links::connect_contours;
pub(crate) use segments::{IntersectionKind, LinkQuality, LinkType, slice_vertical_lines};

#[cfg(test)]
mod tests;
