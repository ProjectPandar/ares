//! `TreeSupport.cpp:1978-2214` node drawing and roof/base closure prefix.

pub(super) mod areas;
mod bottom_interfaces;
mod collision;
mod contour_intersection;
pub(super) mod groups;
mod hole_propagation;
mod hybrid_expansion;
pub(super) mod lightning;
mod node_shapes;
mod trimming;

use areas::{LayerAreas, NodeDrawing};
use bottom_interfaces::BottomInterfaces;
use collision::DrawCollision;
use hybrid_expansion::HybridExpansion;
use node_shapes::NodeShapes;
use trimming::TreeTrimming;

use super::{
    contacts::ContactNodes, data::TreeSupportData, heights::LayerHeightData, nodes::NodeArena,
};
use crate::{
    FloatOrPercent, ObjectOptions, ProcessSupportBasePattern, ProcessSupportInterfacePattern,
    ProcessSupportStyle,
    fill::lightning::generator::SupportSettings,
    geometry::{
        BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon,
        clip_clipper_expolygons_with_subject_bbox, difference_ex_polygons,
        intersection_ex_polygons, offset2_ex, simplification::append_simplified_expolygon,
    },
    project_slice::{
        layers::PlannedLayer,
        perimeters::types::Flow,
        support_material::{
            interface_counts::InterfaceCounts, layer_input::LayerInput, trim_object::TrimLayer,
        },
    },
};

pub(super) struct Drawing<'a> {
    pub(super) object: &'a ObjectOptions,
    pub(super) arena: &'a NodeArena,
    pub(super) contacts: &'a ContactNodes,
    pub(super) data: &'a mut TreeSupportData,
    pub(super) heights: &'a [LayerHeightData],
    pub(super) layers: &'a [LayerInput<'a>],
    pub(super) object_layers: &'a [PlannedLayer],
    pub(super) tops: &'a [Vec<Polygon>],
    pub(super) bottoms: &'a [Vec<Polygon>],
    pub(super) raft_layers: usize,
    pub(super) top_z_distance: f64,
    pub(super) line_width: f64,
    pub(super) support_width: f64,
    pub(super) bottom_gap: f64,
    pub(super) nominal_height: f64,
    pub(super) nominal_support_flow: &'a Flow,
    pub(super) lightning_settings: &'a SupportSettings,
    pub(super) counts: &'a InterfaceCounts,
    pub(super) trim_layers: &'a [TrimLayer<'a>],
    pub(super) zero_gap_interface_top: bool,
    pub(super) gap_xy_first_layer: f64,
    pub(super) first_layer_width_scaled: i64,
    pub(super) brim_skirt_layers: usize,
    pub(super) bed: &'a [Polygon],
    pub(super) scale: CoordinateScale,
}

pub(super) struct DrawnLayers {
    pub(super) layers: Vec<LayerAreas>,
    pub(super) lightning: Option<lightning::SupportLightning>,
}

impl Drawing<'_> {
    pub(super) fn draw_node_areas(&mut self) -> Result<DrawnLayers, ClipperError> {
        let shapes = NodeShapes::new(self.contacts, self.object, self.scale);
        // TreeSupport.cpp:2184 uses the raw configured width, not the resolved flow width.
        let width = match self.object.support_line_width {
            FloatOrPercent::Float(value) => value,
            FloatOrPercent::Percent(value) => value.0,
        };
        let max_move_sq = self.scale.scaled_delta(width / 2.0).powi(2);
        let mut output = Vec::with_capacity(self.heights.len());
        for (layer, height) in self.heights.iter().enumerate() {
            let nodes = &self.contacts.layers[layer];
            if nodes.is_empty() {
                output.push(LayerAreas::default());
                continue;
            }
            if height.height < 0.0001 {
                output.push(LayerAreas {
                    print_z: height.print_z,
                    height: height.height,
                    ..LayerAreas::default()
                });
                continue;
            }
            let mut collision = DrawCollision {
                object: self.object,
                outlines: &self.data.outlines,
                layers: self.layers,
                object_layer: height.object_layer,
                top_z_distance: self.top_z_distance,
                xy_distance: self.data.xy_distance,
                scale: self.scale,
                base: Vec::new(),
                sharp_tail: Vec::new(),
            };
            let (mut areas, extra_wall_areas) = NodeDrawing {
                shapes: &shapes,
                collision: &mut collision,
                top_interface_layers: self.counts.top,
                raft_layers: self.raft_layers,
                buildplate_only: self.object.support_on_build_plate_only.0,
                scale: self.scale,
                max_move_sq,
            }
            .run(self.arena, nodes)?;
            areas.print_z = height.print_z;
            areas.height = height.height;
            self.close_roofs(&mut areas, &mut collision)?;
            if shapes.square {
                self.simplify_base(&mut areas)?;
            }
            if !areas.base.is_empty()
                && !self.object.support_on_build_plate_only.0
                && (self.bottom_gap > 0.0001 || self.counts.bottom > 0)
            {
                BottomInterfaces {
                    object_layers: self.object_layers,
                    tops: self.tops,
                    bottoms: self.bottoms,
                    heights: self.heights,
                    gap: self.bottom_gap,
                    nominal_height: self.nominal_height,
                    interface_layers: self.counts.bottom,
                    base_interface_layers: self.counts.bottom_base,
                    support_width: self.support_width,
                    scale: self.scale,
                }
                .apply(&mut areas, layer, height.object_layer)?;
            }
            self.finish_layer(&mut areas, &extra_wall_areas, layer)?;
            output.push(areas);
        }
        groups::sequence_interfaces(
            &mut output,
            self.object.support_interface_pattern
                == ProcessSupportInterfacePattern::RectilinearInterlaced,
            self.counts.top_base,
        );
        let with_infill = match self.object.support_base_pattern {
            ProcessSupportBasePattern::Hollow => false,
            ProcessSupportBasePattern::Default => {
                super::super::super::style::resolve(
                    self.object.support_type,
                    self.object.support_style,
                ) == ProcessSupportStyle::TreeHybrid
            }
            ProcessSupportBasePattern::Rectilinear
            | ProcessSupportBasePattern::RectilinearGrid
            | ProcessSupportBasePattern::Honeycomb
            | ProcessSupportBasePattern::Lightning => true,
        };
        let lightning = if self.object.support_base_pattern == ProcessSupportBasePattern::Lightning
        {
            Some(
                lightning::LightningInput {
                    contacts: self.contacts,
                    support_width: self.support_width,
                    nominal_flow: self.nominal_support_flow,
                    base_pattern_spacing: self.object.support_base_pattern_spacing.0,
                    settings: self.lightning_settings,
                }
                .generate(&output)?,
            )
        } else {
            None
        };
        if lightning.is_none() && !with_infill {
            hole_propagation::HolePropagation {
                contacts: self.contacts,
                heights: self.heights,
                data: self.data,
                line_width_scaled: self.scale.scaled_delta(self.line_width),
                scale: self.scale,
            }
            .run(&mut output)?;
        }
        output.retain(|layer| {
            let remove = layer.height < 0.0001;
            !remove
        });
        Ok(DrawnLayers {
            layers: output,
            lightning,
        })
    }

    fn finish_layer(
        &self,
        areas: &mut LayerAreas,
        extra_wall_areas: &[ExPolygon],
        layer: usize,
    ) -> Result<(), ClipperError> {
        if layer == 0
            && self.raft_layers == 0
            && super::super::super::style::resolve(
                self.object.support_type,
                self.object.support_style,
            ) == ProcessSupportStyle::TreeHybrid
            && self.object.raft_first_layer_expansion.0 > 0.0
        {
            HybridExpansion {
                object_slices: self.layers[0].slices,
                expansion: self.object.raft_first_layer_expansion.0,
                gap_xy: self.gap_xy_first_layer,
                first_layer_width_scaled: self.first_layer_width_scaled,
                scale: self.scale,
            }
            .apply(areas)?;
        }
        if !areas.base.is_empty() {
            let trimming = TreeTrimming {
                planned: self.object_layers,
                layers: self.trim_layers,
                gap_below: self.bottom_gap,
                zero_gap_interface_top: self.zero_gap_interface_top,
                thick_bridges: self.object.thick_bridges.0,
                scale: self.scale,
            }
            .regions(areas.print_z, areas.height)?;
            if !trimming.is_empty() {
                areas.base = difference_ex_polygons(&areas.base, &trimming)?;
            }
        }
        groups::build_groups(
            areas,
            extra_wall_areas,
            layer < self.brim_skirt_layers,
            self.scale,
        )
    }

    fn simplify_base(&self, areas: &mut LayerAreas) -> Result<(), ClipperError> {
        let mut simplified = Vec::with_capacity(areas.base.len());
        for area in std::mem::take(&mut areas.base) {
            append_simplified_expolygon(
                area,
                self.scale.scaled_delta(self.line_width / 2.0),
                &mut simplified,
            )?;
        }
        areas.base = simplified;
        Ok(())
    }

    fn close_roofs(
        &self,
        areas: &mut LayerAreas,
        collision: &mut DrawCollision<'_>,
    ) -> Result<(), ClipperError> {
        let width = self.scale.scaled_delta(self.line_width) as f32;
        areas.roof = difference_clipped(
            &offset2_ex(&areas.roof, width, -width, JoinType::Miter, 3.0)?,
            collision.get(false)?,
            self.scale,
        )?;
        areas.roof = intersection_ex_polygons(&areas.roof, self.bed)?;
        areas.roof_first = difference_clipped(
            &offset2_ex(&areas.roof_first, width, -width, JoinType::Miter, 3.0)?,
            collision.get(false)?,
            self.scale,
        )?;
        areas.roof_first = difference_bbox(&areas.roof_first, &areas.roof, 0)?;
        areas.roof_first = intersection_ex_polygons(&areas.roof_first, self.bed)?;
        let mut roofs =
            Vec::with_capacity(areas.roof_first.len() + areas.roof.len() + areas.roof_gap.len());
        roofs.extend_from_slice(&areas.roof_first);
        roofs.extend_from_slice(&areas.roof);
        roofs.extend_from_slice(&areas.roof_gap);
        areas.base = difference_bbox(&areas.base, &roofs, 0)?;
        areas.base = intersection_ex_polygons(&areas.base, self.bed)?;
        Ok(())
    }
}

fn difference_clipped(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError> {
    difference_bbox(subject, clip, scale.scaled_delta(0.0001) as i64)
}

fn difference_bbox(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
    inflation: i64,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let Some(bounds) = BoundingBox::from_expolygons(subject) else {
        return Ok(Vec::new());
    };
    difference_ex_polygons(
        subject,
        &clip_clipper_expolygons_with_subject_bbox(clip, bounds.inflated(inflation)),
    )
}
