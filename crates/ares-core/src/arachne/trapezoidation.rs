mod builder;
mod discretize;
mod filtering;
mod index;
mod transitions;
mod voronoi;

use std::collections::HashMap;

use boostvoronoi::prelude::{EdgeIndex, VertexIndex};

use crate::geometry::{CoordinateScale, Polygon};

use super::{
    beading::base::BeadingStrategy,
    skeletal::{EdgeId, NodeId, SkeletalGraph},
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct TrapezoidationConfig {
    pub(crate) transitioning_angle: f64,
    pub(crate) discretization_step_size: i64,
    pub(crate) transition_filter_dist: i64,
    pub(crate) allowed_filter_deviation: i64,
    #[expect(
        dead_code,
        reason = "consumed by the deferred beading propagation stage"
    )]
    pub(crate) beading_propagation_transition_dist: i64,
    pub(crate) coordinate_scale: CoordinateScale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TrapezoidationError {
    EmptyPolygon,
    VoronoiConstruction,
    InvalidTopology,
}

pub(crate) struct SkeletalTrapezoidation<'a> {
    pub(crate) graph: SkeletalGraph,
    beading_strategy: &'a dyn BeadingStrategy,
    config: TrapezoidationConfig,
    vd_edge_to_he_edge: HashMap<EdgeIndex, EdgeId>,
    vd_node_to_he_node: HashMap<VertexIndex, NodeId>,
    transition_storage: Vec<super::skeletal::Shared<super::skeletal::TransitionMiddle>>,
    transition_end_storage: Vec<super::skeletal::Shared<super::skeletal::TransitionEnd>>,
}

impl<'a> SkeletalTrapezoidation<'a> {
    pub(crate) fn new(
        polygons: &[Polygon],
        beading_strategy: &'a dyn BeadingStrategy,
        config: TrapezoidationConfig,
    ) -> Result<Self, TrapezoidationError> {
        let mut result = Self {
            graph: SkeletalGraph::default(),
            beading_strategy,
            config,
            vd_edge_to_he_edge: HashMap::new(),
            vd_node_to_he_node: HashMap::new(),
            transition_storage: Vec::new(),
            transition_end_storage: Vec::new(),
        };
        result.construct_from_polygons(polygons)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests;
