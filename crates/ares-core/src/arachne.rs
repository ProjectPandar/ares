pub(crate) mod beading;
mod extrusion_line;
pub(crate) mod skeletal;
mod trapezoidation;

#[cfg(test)]
pub(crate) use extrusion_line::{ExtrusionJunction, ExtrusionLine};

#[cfg(test)]
mod tests;
