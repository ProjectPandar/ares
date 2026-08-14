mod graph;
mod operations;
mod payload;

#[cfg(test)]
pub(crate) use graph::{EdgeId, NodeId, SkeletalGraph};
#[cfg(test)]
pub(crate) use payload::{
    BeadingPropagation, EdgeType, Shared, SkeletalEdge, SkeletalJoint, TransitionEnd,
    TransitionMiddle,
};

#[cfg(test)]
mod tests;
