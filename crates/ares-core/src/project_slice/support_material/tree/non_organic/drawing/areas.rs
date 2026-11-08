//! `TreeSupport.cpp:2090-2192` node areas and mm-capped roof routing.

use super::super::nodes::{NodeArena, NodeId, NodeType, SupportNode};
use super::{
    collision::DrawCollision, difference_clipped, groups::AreaGroup, node_shapes::NodeShapes,
};
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType,
    clip_clipper_expolygons_with_subject_bbox, difference_ex_polygons, offset_expolygons,
};

#[derive(Default)]
pub(in super::super) struct LayerAreas {
    pub(in super::super) print_z: f64,
    pub(in super::super) height: f64,
    pub(in super::super) base: Vec<ExPolygon>,
    pub(in super::super) roof: Vec<ExPolygon>,
    pub(in super::super) roof_first: Vec<ExPolygon>,
    pub(super) roof_gap: Vec<ExPolygon>,
    pub(in super::super) floor: Vec<ExPolygon>,
    pub(super) floor_interface_as_base: bool,
    pub(super) polygons: Vec<ExPolygon>,
    pub(super) has_circle_node: bool,
    pub(super) need_extra_wall: bool,
    pub(in super::super) groups: Vec<AreaGroup>,
    pub(in super::super) slices: Vec<ExPolygon>,
    pub(super) slice_bounds: Vec<BoundingBox>,
}

pub(super) struct NodeDrawing<'a, 's> {
    pub(super) shapes: &'a NodeShapes<'s>,
    pub(super) collision: &'a mut DrawCollision<'s>,
    pub(super) top_interface_layers: usize,
    pub(super) raft_layers: usize,
    pub(super) buildplate_only: bool,
    pub(super) scale: CoordinateScale,
}

impl NodeDrawing<'_, '_> {
    pub(super) fn run(
        &mut self,
        arena: &NodeArena,
        nodes: &[NodeId],
    ) -> Result<LayerAreas, ClipperError> {
        let mut output = LayerAreas::default();
        for &id in nodes {
            let node = &arena.nodes[id];
            let mut area = self.node_area(node, &mut output)?;
            if self.collision.object_layer > 0 && node.distance_to_top < 0 {
                output.roof_gap.append(&mut area);
            } else if self.collision.object_layer > 0
                && node.roof_layers_below == 1
                && !node.is_sharp_tail
            {
                output.roof_first.append(&mut area);
            } else if self.collision.object_layer > 0
                && node.roof_layers_below > 1
                && !node.is_sharp_tail
            {
                output.roof.append(&mut area);
            } else {
                output.base.append(&mut area);
            }
        }
        Ok(output)
    }

    fn node_area(
        &mut self,
        node: &SupportNode,
        output: &mut LayerAreas,
    ) -> Result<Vec<ExPolygon>, ClipperError> {
        let sharp_tail = node.is_sharp_tail && node.distance_to_top <= 0;
        if node.node_type == NodeType::Polygon || (node.distance_to_top < 0 && !node.is_sharp_tail)
        {
            let expanded = expanded_overhang(node, self.collision.xy_distance, self.scale)?;
            let area = difference_clipped(&expanded, self.collision.get(sharp_tail)?, self.scale)?;
            if node.node_type == NodeType::Polygon {
                output.polygons.extend_from_slice(&area);
            }
            return Ok(area);
        }
        let circle = self
            .shapes
            .for_node(node, self.collision.object_layer, self.raft_layers)?;
        let mut area = largest_component(
            &ExPolygon::new(circle, Vec::new()),
            self.collision.get(sharp_tail)?,
        )?;
        output.has_circle_node |= !area.is_empty();
        output.need_extra_wall |= node.need_extra_wall;
        if self.top_interface_layers > 0
            && node.roof_layers_below > 0
            && !self.buildplate_only
            && !node.is_sharp_tail
        {
            area.extend(expanded_overhang(
                node,
                self.collision.xy_distance,
                self.scale,
            )?);
        }
        Ok(area)
    }
}

fn expanded_overhang(
    node: &SupportNode,
    xy_distance: f64,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError> {
    if node.overhang.contour().points().len() > 100 || node.overhang.holes().len() > 1 {
        Ok(vec![node.overhang.clone()])
    } else {
        offset_expolygons(
            std::slice::from_ref(&node.overhang),
            scale.scaled_delta(xy_distance) as f32,
            JoinType::Miter,
            3.0,
        )
    }
}

// TreeSupport.cpp:1884-1904 compares double area to a float running maximum.
fn largest_component(
    polygon: &ExPolygon,
    collision: &[ExPolygon],
) -> Result<Vec<ExPolygon>, ClipperError> {
    let Some(bounds) = BoundingBox::from_expolygon(polygon) else {
        return Ok(Vec::new());
    };
    let clip = clip_clipper_expolygons_with_subject_bbox(collision, bounds);
    let components = difference_ex_polygons(std::slice::from_ref(polygon), &clip)?;
    let mut maximum = 0.0_f32;
    let mut selected = None;
    for (index, component) in components.iter().enumerate() {
        let area = component.area();
        if area > f64::from(maximum) {
            maximum = area as f32;
            selected = Some(index);
        }
    }
    Ok(selected.map_or_else(Vec::new, |index| {
        vec![
            components
                .into_iter()
                .nth(index)
                .expect("selected area component exists"),
        ]
    }))
}
