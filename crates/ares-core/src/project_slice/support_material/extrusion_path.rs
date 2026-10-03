//! `ExtrusionEntity.hpp:592-602` paths shared by support loops and infill.

mod chain;
mod loop_split;

use crate::project_slice::raft::fills::RaftEntity;
use crate::{
    ExtrusionRole,
    geometry::{Polygon, Polyline},
    project_slice::perimeters::{
        classic::materialize::{Point3, Polyline3},
        types::Flow,
    },
};

pub(in crate::project_slice) struct SupportExtrusionPath {
    pub(in crate::project_slice) polyline: Polyline3,
    pub(in crate::project_slice) role: ExtrusionRole,
    pub(super) can_reverse: bool,
    pub(in crate::project_slice) mm3_per_mm: f64,
    pub(in crate::project_slice) width: f32,
    pub(in crate::project_slice) height: f32,
}

impl SupportExtrusionPath {
    fn new(line: Polyline, role: ExtrusionRole, flow: Flow, can_reverse: bool) -> Self {
        Self::from_parameters(
            line,
            role,
            (flow.mm3_per_mm, flow.width, flow.height),
            can_reverse,
        )
    }

    pub(super) fn from_parameters(
        line: Polyline,
        role: ExtrusionRole,
        (mm3_per_mm, width, height): (f64, f32, f32),
        can_reverse: bool,
    ) -> Self {
        Self {
            polyline: Polyline3 {
                points: line
                    .into_points()
                    .into_iter()
                    .map(|point| Point3 {
                        x: point.x(),
                        y: point.y(),
                        z: 0,
                    })
                    .collect(),
                fitting: Vec::new(),
            },
            role,
            can_reverse,
            mm3_per_mm,
            width,
            height,
        }
    }
}

pub(in crate::project_slice) enum SupportExtrusionEntity {
    Path(SupportExtrusionPath),
    Loop(SupportExtrusionPath),
    NoSort(Vec<SupportExtrusionEntity>),
    MultiPath(Vec<SupportExtrusionPath>),
}

impl SupportExtrusionEntity {
    pub(super) fn append_loops(
        destination: &mut Vec<Self>,
        loops: Vec<Polygon>,
        role: ExtrusionRole,
        parameters: (f64, f32, f32),
    ) {
        destination.reserve(loops.len());
        destination.extend(
            loops
                .into_iter()
                .filter(|polygon| polygon.points().len() >= 3)
                .map(|polygon| {
                    Self::Loop(SupportExtrusionPath::from_parameters(
                        polygon.split_at_first_point(),
                        role,
                        parameters,
                        false,
                    ))
                }),
        );
    }

    pub(super) fn append_paths(
        destination: &mut Vec<Self>,
        lines: Vec<Polyline>,
        role: ExtrusionRole,
        flow: Flow,
    ) {
        destination.reserve(lines.len());
        destination.extend(
            lines
                .into_iter()
                .filter(|line| line.points().len() >= 2)
                .map(|line| Self::Path(SupportExtrusionPath::new(line, role, flow, true))),
        );
    }

    pub(super) fn from_fill_entity(entity: RaftEntity, role: ExtrusionRole, flow: Flow) -> Self {
        match entity {
            RaftEntity::Path(line) => Self::Path(SupportExtrusionPath::new(line, role, flow, true)),
            RaftEntity::Oriented(line) => {
                Self::Path(SupportExtrusionPath::new(line, role, flow, false))
            }
            RaftEntity::NoSort(lines) => Self::NoSort(
                lines
                    .into_iter()
                    .map(|line| Self::Path(SupportExtrusionPath::new(line, role, flow, true)))
                    .collect(),
            ),
        }
    }

    pub(in crate::project_slice) fn role(&self) -> ExtrusionRole {
        match self {
            Self::Path(path) | Self::Loop(path) => path.role,
            Self::MultiPath(paths) => paths[0].role,
            Self::NoSort(entities) => entities.iter().fold(ExtrusionRole::None, |role, entity| {
                let next = entity.role();
                if role == ExtrusionRole::None || role == next {
                    next
                } else {
                    ExtrusionRole::Mixed
                }
            }),
        }
    }

    pub(in crate::project_slice) fn for_each_path(
        &self,
        visit: &mut impl FnMut(&SupportExtrusionPath),
    ) {
        match self {
            Self::Path(path) | Self::Loop(path) => visit(path),
            Self::MultiPath(paths) => paths.iter().for_each(visit),
            Self::NoSort(entities) => {
                for entity in entities {
                    entity.for_each_path(visit);
                }
            }
        }
    }

    pub(in crate::project_slice) fn for_each_path_mut(
        &mut self,
        visit: &mut impl FnMut(&mut SupportExtrusionPath),
    ) {
        match self {
            Self::Path(path) | Self::Loop(path) => visit(path),
            Self::MultiPath(paths) => paths.iter_mut().for_each(visit),
            Self::NoSort(entities) => {
                for entity in entities {
                    entity.for_each_path_mut(visit);
                }
            }
        }
    }
}
