//! `TreeSupport.cpp:1519-1598` roof, floor and first-roof extrusion.

use super::super::drawing::groups::{AreaGroup, AreaKind};
use super::{
    ToolpathInput,
    fill::{FillSettings, Filler},
    geometry_error,
    loops::perimeter_and_inner_brim,
    perimeter::PerimeterInfill,
};
use crate::{
    ExtrusionRole, ProcessSupportInterfacePattern, SliceError,
    geometry::{ExPolygon, JoinType, Point, offset_expolygon},
    project_slice::{
        perimeters::types::Flow, raft::fill_params::RaftFillPattern,
        support_material::SupportExtrusionEntity,
    },
};

pub(super) struct InterfaceArea<'a, 's> {
    pub(super) input: &'a ToolpathInput<'s>,
    pub(super) area: &'a ExPolygon,
    pub(super) group: &'a AreaGroup,
    pub(super) support_flow: Flow,
    pub(super) interface_flow: Flow,
    pub(super) layer_id: usize,
}

impl InterfaceArea<'_, '_> {
    pub(super) fn append(
        &self,
        destination: &mut Vec<SupportExtrusionEntity>,
    ) -> Result<(), SliceError> {
        let input = self.input;
        let wall_count = input.object.tree_support_wall_count.0 as usize;
        let areas = if self.layer_id == 0 {
            let flow = if input.raft_layers == 0 {
                input.brim_flow
            } else {
                self.support_flow
            };
            let role = if self.group.kind == AreaKind::Roof && !self.group.interface_as_base {
                ExtrusionRole::SupportMaterialInterface
            } else {
                ExtrusionRole::SupportMaterial
            };
            destination.extend(
                perimeter_and_inner_brim(self.area, wall_count, flow, role, input.scale)
                    .map_err(geometry_error)?,
            );
            offset_expolygon(
                self.area,
                -flow.scaled_spacing(input.scale) as f32,
                JoinType::Miter,
                3.0,
            )
            .map_err(geometry_error)?
        } else if self.group.kind == AreaKind::RoofFirst {
            offset_expolygon(
                self.area,
                (0.5 * self.support_flow.scaled_width(input.scale) as f64) as f32,
                JoinType::Miter,
                3.0,
            )
            .map_err(geometry_error)?
        } else {
            vec![self.area.clone()]
        };
        let fill = FillSettings {
            density: input.interface_density(self.group.kind == AreaKind::Floor),
            dont_sort: false,
            flow: if self.group.interface_as_base {
                self.support_flow
            } else {
                self.interface_flow
            },
            role: if self.group.interface_as_base {
                ExtrusionRole::SupportMaterial
            } else {
                ExtrusionRole::SupportMaterialInterface
            },
        };
        let mut filler = Filler {
            scale: input.scale,
            pattern: input.parameters.contact_fill_pattern,
            spacing: f64::from(self.interface_flow.spacing),
            angle: (f64::from(input.base_angle()) + std::f64::consts::FRAC_PI_2) as f32,
            reference: Some(Point::new(0, 0)),
        };
        if self.group.kind == AreaKind::RoofFirst {
            filler.pattern = RaftFillPattern::Rectilinear;
            filler.angle = input
                .parameters
                .support_interface_angle(input.object, self.group.interface_id);
            let mut entities = Vec::new();
            filler
                .perimeter_and_infill(
                    &mut entities,
                    self.area,
                    PerimeterInfill {
                        wall_count: 1,
                        fill: FillSettings {
                            density: input.interface_density(false),
                            dont_sort: true,
                            ..fill
                        },
                        infill_first: false,
                    },
                )
                .map_err(geometry_error)?;
            if !entities.is_empty() {
                destination.push(SupportExtrusionEntity::NoSort(entities));
            }
            return Ok(());
        }
        let mut fill = fill;
        match input.object.support_interface_pattern {
            ProcessSupportInterfacePattern::SpiralInset => {
                return Err(crate::SliceError::UnsupportedProjectFeature(
                    "support_interface_pattern: spiralinset".to_owned(),
                ));
            }
            ProcessSupportInterfacePattern::Grid => {
                filler.angle = input.base_angle();
                fill.dont_sort = true;
            }
            ProcessSupportInterfacePattern::RectilinearInterlaced => {
                filler.angle = (f64::from(input.base_angle())
                    + (self.group.interface_id & 1) as f64 * std::f64::consts::FRAC_PI_2)
                    as f32;
                fill.dont_sort = true;
            }
            ProcessSupportInterfacePattern::Auto
            | ProcessSupportInterfacePattern::Rectilinear
            | ProcessSupportInterfacePattern::Concentric => {}
        }
        filler
            .append(destination, &areas, fill)
            .map_err(geometry_error)?;
        Ok(())
    }
}
