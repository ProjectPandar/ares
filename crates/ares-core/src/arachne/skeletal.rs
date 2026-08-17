mod graph;
mod operations;
mod payload;

pub(crate) use graph::{EdgeId, NodeId, SkeletalGraph};
pub(crate) use payload::{
    BeadingPropagation, EdgeType, Shared, SkeletalEdge, SkeletalJoint, TransitionEnd,
    TransitionMiddle,
};

#[cfg(test)]
mod tests;
