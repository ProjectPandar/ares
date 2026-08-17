mod graph;
mod operations;
mod payload;

pub(crate) use graph::{EdgeId, NodeId, SkeletalGraph};
#[cfg(test)]
pub(crate) use payload::BeadingPropagation;
pub(crate) use payload::{
    EdgeType, Shared, SkeletalEdge, SkeletalJoint, TransitionEnd, TransitionMiddle,
};

#[cfg(test)]
mod tests;
