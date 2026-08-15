mod graph;
mod operations;
mod payload;

pub(crate) use graph::{EdgeId, NodeId, SkeletalGraph};
#[cfg(test)]
pub(crate) use payload::{BeadingPropagation, Shared, TransitionEnd, TransitionMiddle};
pub(crate) use payload::{EdgeType, SkeletalEdge, SkeletalJoint};

#[cfg(test)]
mod tests;
