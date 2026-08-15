mod graph;
mod operations;
mod payload;

pub(crate) use graph::{EdgeId, NodeId, SkeletalGraph};
#[cfg(test)]
pub(crate) use payload::BeadingPropagation;
#[cfg(test)]
pub(crate) use payload::TransitionEnd;
pub(crate) use payload::{EdgeType, SkeletalEdge, SkeletalJoint};
pub(crate) use payload::{Shared, TransitionMiddle};

#[cfg(test)]
mod tests;
