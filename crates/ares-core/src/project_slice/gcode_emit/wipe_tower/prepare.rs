//! `Print.cpp:3444,3572`: dispatch the owning upstream tower implementation.
use super::super::tool_ordering::{LayerTools, ToolOrdering};
use super::{Data, planning, type1};
use crate::{
    ProjectSettings,
    geometry::{ClipperError, CoordinateScale},
    project_slice::{
        extrusion_islands::PreparedPostExtrusionIslands,
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        support_material::InstalledSupportLayer,
    },
};

pub(in crate::project_slice::gcode_emit) enum Planned {
    Type1(Box<type1::Planned>),
    Type2(planning::Planned),
}

pub(in crate::project_slice::gcode_emit) fn prepare(
    traversal: &PreparedPostClassicTraversal,
    original: &PreparedPostExtrusionIslands,
    support: &[Vec<InstalledSupportLayer>],
    ordering: &mut ToolOrdering,
) -> Option<Planned> {
    match ordering.tower_type? {
        crate::WipeTowerType::Type1 => Some(Planned::Type1(Box::new(type1::prepare(
            traversal, original, support, ordering,
        )))),
        crate::WipeTowerType::Type2 => {
            planning::prepare(traversal, original, support, ordering).map(Planned::Type2)
        }
    }
}
impl Planned {
    pub(in crate::project_slice::gcode_emit) fn generate(
        self,
        settings: &ProjectSettings,
        scale: CoordinateScale,
        tools: (&[usize], &LayerTools),
        height: f64,
    ) -> Result<Data, ClipperError> {
        match self {
            Self::Type1(plan) => (*plan).generate(settings, scale, tools.1, height),
            Self::Type2(plan) => plan.generate(settings, scale, tools, height),
        }
    }
}
