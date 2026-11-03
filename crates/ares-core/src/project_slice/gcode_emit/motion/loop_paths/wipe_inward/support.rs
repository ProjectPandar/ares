//! `WipePathHelpers.hpp:12-18`, `.cpp:21-38`: region-scoped printed entities.

use super::math::lines;
use crate::geometry::Line;
use crate::project_slice::perimeters::classic::{
    entity_collections::ExtrusionEntity, materialize::ExtrusionRole,
};

#[derive(Default)]
pub(in crate::project_slice::gcode_emit::motion) struct WipeInwardSupport {
    pub(super) printed_lines: Vec<Line>,
    pub(super) inner_lines: Vec<Line>,
}

impl WipeInwardSupport {
    pub(in crate::project_slice::gcode_emit::motion) fn has_inner(&self) -> bool {
        !self.inner_lines.is_empty()
    }

    pub(in crate::project_slice::gcode_emit::motion) fn append(
        &mut self,
        entity: &ExtrusionEntity,
    ) {
        let inner = match entity {
            ExtrusionEntity::Loop(loop_) => loop_
                .extrusion_loop
                .paths
                .iter()
                .any(|path| path.role == ExtrusionRole::Perimeter),
            ExtrusionEntity::MultiPath(multi) => multi
                .paths
                .iter()
                .any(|path| path.role == ExtrusionRole::Perimeter),
            ExtrusionEntity::Path(path) => path.role == ExtrusionRole::Perimeter,
        };
        let lines = lines(&entity.as_polyline());
        self.printed_lines.extend_from_slice(&lines);
        if inner {
            self.inner_lines.extend(lines);
        }
    }
}
