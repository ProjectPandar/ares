//! `TreeSupport.cpp:1599-1684` per-area base outline and infill ordering.

use super::super::drawing::{groups::AreaGroup, lightning::SupportLightning};
use super::{
    ToolpathInput,
    fill::{FillSettings, Filler},
    geometry_error,
    perimeter::PerimeterInfill,
};
use crate::{
    ExtrusionRole, ProcessSupportBasePattern, ProcessSupportStyle, SliceError,
    geometry::{ExPolygon, JoinType, Line, Point, offset_expolygon},
    project_slice::{
        perimeters::{classic::shortest_path::chain_polylines, types::Flow},
        raft::{
            fill_params::{RaftFillPattern, tree_double_wall_area},
            fills::{RaftFillPreprocess, RaftFillSpec, raft_layer_entities},
        },
        support_material::{SupportExtrusionEntity, layer_input::to_polygons},
    },
};

pub(super) struct BaseArea<'a, 's> {
    pub(super) input: &'a ToolpathInput<'s>,
    pub(super) area: &'a ExPolygon,
    pub(super) group: &'a AreaGroup,
    pub(super) support_flow: Flow,
    pub(super) layer_id: usize,
    pub(super) print_z: f64,
    pub(super) lightning: Option<&'a SupportLightning>,
}

impl BaseArea<'_, '_> {
    pub(super) fn append(
        &self,
        destination: &mut Vec<SupportExtrusionEntity>,
    ) -> Result<(), SliceError> {
        let input = self.input;
        let on_bed = self.layer_id == 0 && input.raft_layers == 0;
        let flow = if on_bed {
            input.parameters.first_layer_flow
        } else {
            self.support_flow
        };
        let spacing =
            input.object.support_base_pattern_spacing.0 + f64::from(self.support_flow.spacing);
        let density = (f64::from(self.support_flow.spacing) / spacing).min(1.0);
        let pattern = input.object.support_base_pattern;
        let with_infill = match pattern {
            ProcessSupportBasePattern::Hollow => false,
            ProcessSupportBasePattern::Default => {
                super::super::super::super::style::resolve(
                    input.object.support_type,
                    input.object.support_style,
                ) == ProcessSupportStyle::TreeHybrid
            }
            ProcessSupportBasePattern::Rectilinear
            | ProcessSupportBasePattern::RectilinearGrid
            | ProcessSupportBasePattern::Honeycomb
            | ProcessSupportBasePattern::Lightning => true,
        };
        let need_infill = with_infill
            && (pattern != ProcessSupportBasePattern::Default || self.group.need_infill);
        let filler = Filler {
            scale: input.scale,
            pattern: if on_bed {
                RaftFillPattern::Rectilinear
            } else {
                input.parameters.base_fill_pattern
            },
            spacing: if on_bed {
                f64::from(flow.spacing)
            } else {
                spacing * density
            },
            angle: input.base_angle(),
            reference: Some(Point::new(0, 0)),
        };
        let mut entities = Vec::new();
        if self.layer_id == 0 || !need_infill || pattern == ProcessSupportBasePattern::Lightning {
            let spec = RaftFillSpec {
                angle: f64::from(filler.angle),
                spacing: filler.spacing,
                sheath_spacing: f64::from(flow.spacing),
                density: if self.layer_id == 0 {
                    (input.object.raft_first_layer_density.0 * 0.01) as f32
                } else {
                    0.0
                },
                sheath: true,
                no_sort: false,
                tree_double_wall_area: if self.group.need_extra_wall
                    && input.object.tree_support_wall_count.0 == 0
                {
                    0.1
                } else {
                    tree_double_wall_area(
                        input.object.tree_support_wall_count.0,
                        1.0 / input.scale.factor(),
                    )
                },
                width_mm: f64::from(flow.width),
                pattern: filler.pattern,
                preprocess: RaftFillPreprocess::Closing,
            };
            entities.extend(
                raft_layer_entities(
                    &to_polygons(std::slice::from_ref(self.area)),
                    spec,
                    input.scale,
                )
                .map_err(geometry_error)?
                .into_iter()
                .map(|entity| {
                    SupportExtrusionEntity::from_fill_entity(
                        entity,
                        ExtrusionRole::SupportMaterial,
                        flow,
                    )
                }),
            );
        } else {
            let minimum = usize::from(
                offset_expolygon(
                    self.area,
                    -(input.scale.scaled_delta(spacing * 1.5) as i64) as f32,
                    JoinType::Miter,
                    3.0,
                )
                .map_err(geometry_error)?
                .is_empty(),
            );
            filler
                .perimeter_and_infill(
                    &mut entities,
                    self.area,
                    PerimeterInfill {
                        wall_count: minimum.max(input.object.tree_support_wall_count.0 as usize),
                        fill: FillSettings {
                            density,
                            dont_sort: false,
                            flow,
                            role: ExtrusionRole::SupportMaterial,
                        },
                        infill_first: false,
                    },
                )
                .map_err(geometry_error)?;
        }
        if pattern == ProcessSupportBasePattern::Lightning {
            self.append_lightning(&mut entities, flow)?;
        }
        if !entities.is_empty() {
            destination.push(SupportExtrusionEntity::NoSort(entities));
        }
        Ok(())
    }

    fn append_lightning(
        &self,
        destination: &mut Vec<SupportExtrusionEntity>,
        flow: Flow,
    ) -> Result<(), SliceError> {
        let lightning = self
            .lightning
            .expect("Lightning drawing installs its generator");
        let Some(layer) = lightning.layer_index(self.print_z) else {
            return Ok(());
        };
        let scale = self.input.scale;
        let flow = Flow {
            mm3_per_mm: f64::from(flow.mm3_per_mm as f32),
            ..flow
        };
        for area in offset_expolygon(
            self.area,
            -flow.scaled_spacing(scale) as f32,
            JoinType::Miter,
            3.0,
        )
        .map_err(geometry_error)?
        {
            let mut lines = lightning
                .generator
                .convert_to_lines(layer, &to_polygons(std::slice::from_ref(&area)), 0)
                .map_err(geometry_error)?;
            lines.retain(|line| {
                let length = line
                    .points()
                    .windows(2)
                    .map(|points| Line::new(points[0], points[1]).length())
                    .sum::<f64>();
                let too_short = length < (scale.scaled_delta(1.0) as i64) as f64;
                !too_short
            });
            chain_polylines(&mut lines, scale);
            SupportExtrusionEntity::append_paths(
                destination,
                lines,
                ExtrusionRole::SupportMaterial,
                flow,
            );
        }
        Ok(())
    }
}
