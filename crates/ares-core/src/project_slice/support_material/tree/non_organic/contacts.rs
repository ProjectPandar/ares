//! `TreeSupport.cpp:3402-3634::generate_contact_points`.

use std::collections::HashSet;

use super::{
    data::TreeSupportData,
    nodes::{NodeArena, NodeId, NodeSeed, NodeType},
    sampling::{self, EdgeCache},
};
use crate::{
    ObjectOptions, ProcessSupportStyle,
    geometry::{
        BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polygon,
        difference_ex, intersection_ex, offset_expolygons, point_in_polygon,
    },
    project_slice::{
        layers::PlannedLayer,
        support_material::tree::overhangs::{OverhangLayer, OverhangType},
    },
};

pub(in crate::project_slice::support_material) struct VerticalEnforcer {
    pub(in crate::project_slice::support_material) position: [f32; 3],
    pub(in crate::project_slice::support_material) normal: [f32; 3],
}

pub(super) struct ContactInput<'a> {
    pub(super) object: &'a ObjectOptions,
    pub(super) layers: &'a [PlannedLayer],
    pub(super) overhangs: &'a [OverhangLayer],
    pub(super) vertical_enforcers: &'a [VerticalEnforcer],
    pub(super) bounds: BoundingBox,
    pub(super) top_z_distance: f64,
    pub(super) style: ProcessSupportStyle,
    pub(super) scale: CoordinateScale,
}

pub(super) struct ContactNodes {
    pub(super) layers: Vec<Vec<NodeId>>,
    pub(super) average_per_layer: usize,
    pub(super) angle: f32,
}

struct LayerContacts<'a> {
    arena: &'a mut NodeArena,
    nodes: Vec<NodeId>,
    inserted: HashSet<(i64, i64)>,
    radius_scaled: i64,
    layer: usize,
    roof_layers: i32,
    gap_layers: i32,
    bottom_z: f64,
    top_z_distance: f64,
    is_sharp_tail: bool,
}

impl LayerContacts<'_> {
    fn overhang(
        &mut self,
        overhang: &ExPolygon,
        grid: &[Point],
        input: &ContactInput<'_>,
        data: &TreeSupportData,
    ) -> Result<(), ClipperError> {
        let threshold = input.scale.scaled_delta(10.0).powi(2);
        if input.style != ProcessSupportStyle::TreeHybrid
            || overhang.area() <= threshold
            || self.is_sharp_tail
        {
            return self.regular(overhang, grid, input);
        }
        let regular = offset_expolygons(
            &intersection_ex(
                std::slice::from_ref(overhang),
                &data.outlines_below[self.layer - 1],
            )?,
            self.radius_scaled as f32,
            JoinType::Miter,
            3.0,
        )?;
        let normal = difference_ex(std::slice::from_ref(overhang), &regular)?;
        if normal.iter().map(ExPolygon::area).sum::<f64>() <= threshold {
            return self.regular(overhang, grid, input);
        }
        for polygon in normal {
            let bounds =
                BoundingBox::from_expolygon(&polygon).expect("contact polygon has a contour");
            let id = self.create(
                bounds.center(),
                &polygon,
                sampling::radius(bounds) * input.scale.factor(),
                true,
            );
            self.arena.nodes[id].node_type = NodeType::Polygon;
            self.nodes.push(id);
        }
        for polygon in regular {
            self.regular(&polygon, grid, input)?;
        }
        Ok(())
    }

    fn insert(
        &mut self,
        point: Point,
        overhang: &ExPolygon,
        radius: f64,
        interface: bool,
    ) -> Option<NodeId> {
        let hash = (
            point.x() / (self.radius_scaled + 1),
            point.y() / (self.radius_scaled + 1),
        );
        if self.inserted.contains(&hash) {
            return None;
        }
        Some(self.create(point, overhang, radius, interface))
    }

    fn create(
        &mut self,
        point: Point,
        overhang: &ExPolygon,
        radius: f64,
        interface: bool,
    ) -> NodeId {
        let hash = (
            point.x() / (self.radius_scaled + 1),
            point.y() / (self.radius_scaled + 1),
        );
        self.inserted.insert(hash);
        let id = self.arena.create_node(NodeSeed {
            position: point,
            distance_to_top: -self.gap_layers,
            roof_layers_below: if interface { self.roof_layers } else { 0 },
            to_buildplate: true,
            parent: None,
            print_z: self.bottom_z,
            height: self.top_z_distance,
            distance_mm_to_top: 0.0,
            radius,
        });
        let node = &mut self.arena.nodes[id];
        node.overhang = overhang.clone();
        node.is_sharp_tail = self.is_sharp_tail;
        self.nodes.push(id);
        id
    }

    fn regular(
        &mut self,
        overhang: &ExPolygon,
        grid: &[Point],
        input: &ContactInput<'_>,
    ) -> Result<(), ClipperError> {
        let interface =
            overhang.area() > input.scale.scaled_delta(1.0).powi(2) && !self.is_sharp_tail;
        let bounds = BoundingBox::from_expolygon(overhang).expect("contact overhang has a contour");
        let radius = (sampling::radius(bounds) * input.scale.factor()).clamp(
            0.4,
            input.object.tree_support_branch_diameter.0.max(0.8) / 2.0,
        );
        let points = overhang.contour().points();
        for (index, &point) in points.iter().enumerate() {
            let previous = points[(index + points.len() - 1) % points.len()];
            let next = points[(index + 1) % points.len()];
            let a = [
                (point.x() - previous.x()) as f64,
                (point.y() - previous.y()) as f64,
            ];
            let b = [(point.x() - next.x()) as f64, (point.y() - next.y()) as f64];
            let a_length = (a[0] * a[0] + a[1] * a[1]).sqrt();
            let b_length = (b[0] * b[0] + b[1] * b[1]).sqrt();
            let dot = a[0] / a_length * (b[0] / b_length) + a[1] / a_length * (b[1] / b_length);
            if dot > -0.7
                && let Some(id) = self.insert(point, overhang, radius, interface)
            {
                self.arena.nodes[id].is_corner = true;
            }
        }
        let spread = input
            .scale
            .scaled_delta(input.object.tree_support_branch_distance.0);
        for polygon in std::iter::once(overhang.contour()).chain(overhang.holes()) {
            let cache = EdgeCache::new(polygon);
            let step = spread / cache.circumference();
            let mut distance = 0.0;
            while distance < 1.0 {
                self.insert(cache.coords(distance), overhang, radius, interface);
                distance += step;
            }
        }
        if self.is_sharp_tail {
            return Ok(());
        }
        let inner = offset_expolygons(
            std::slice::from_ref(overhang),
            -self.radius_scaled as f32,
            JoinType::Miter,
            3.0,
        )?;
        for &point in grid {
            if sampling::contains(bounds, point)
                && inner.iter().any(|polygon| {
                    polygon.contour().contains(&point)
                        && !polygon
                            .holes()
                            .iter()
                            .any(|hole| point_in_polygon(point, hole.points()) == 1)
                })
            {
                self.insert(point, overhang, radius, interface);
            }
        }
        Ok(())
    }
}

pub(super) fn generate_contact_points(
    input: &ContactInput<'_>,
    data: &TreeSupportData,
    arena: &mut NodeArena,
) -> Result<ContactNodes, ClipperError> {
    let scale = input.scale;
    let height = scale.scaled_delta(input.object.layer_height.0);
    let gap = scale.scaled_delta(input.top_z_distance);
    let top_layers = ((gap + height - 1.0) / height) as usize + 1;
    let mut output = ContactNodes {
        layers: Vec::new(),
        average_per_layer: 0,
        angle: 0.0,
    };
    if input.layers.len() <= top_layers + 1 {
        return Ok(output);
    }
    output.layers = vec![Vec::new(); input.layers.len()];
    let grid = sampling::grid_points(
        input.bounds,
        scale
            .scaled_delta(input.object.tree_support_branch_distance.0)
            .max(scale.scaled_delta(input.object.max_bridge_length.0) / 2.0),
    );
    let radius_scaled =
        scale.scaled_delta((input.object.tree_support_branch_diameter.0 / 2.0).max(0.4)) as i64;
    let object_z = input
        .layers
        .iter()
        .map(|layer| layer.slice_z as f32)
        .collect::<Vec<_>>();
    let mut vertical = vec![Vec::new(); input.layers.len()];
    for point in input.vertical_enforcers {
        let layer = object_z.partition_point(|&z| z < point.position[2]);
        if layer > 0 && layer < input.layers.len() {
            vertical[layer].push(point);
        }
    }
    for (layer, vertical_points) in vertical.iter().enumerate().skip(1) {
        let mut contacts = LayerContacts {
            arena,
            nodes: Vec::new(),
            inserted: HashSet::new(),
            radius_scaled,
            layer,
            roof_layers: input.object.support_interface_top_layers.0,
            gap_layers: i32::from(input.top_z_distance != 0.0),
            bottom_z: input.layers[layer].print_z - input.layers[layer].height,
            top_z_distance: input.top_z_distance,
            is_sharp_tail: false,
        };
        for (overhang, &kind) in input.overhangs[layer]
            .overhangs
            .iter()
            .zip(&input.overhangs[layer].overhang_types)
        {
            contacts.is_sharp_tail = kind == OverhangType::SharpTail;
            contacts.overhang(overhang, &grid, input, data)?;
        }
        let empty = ExPolygon::new(Polygon::new(Vec::new()), Vec::new());
        for point in vertical_points {
            contacts.is_sharp_tail = true;
            let position = Point::new(point.position[0] as i64, point.position[1] as i64);
            if let Some(id) = contacts.insert(position, &empty, 0.0, true) {
                contacts.arena.nodes[id].skin_direction = Point::new(
                    (point.normal[0] / scale.factor() as f32) as i64,
                    (point.normal[1] / scale.factor() as f32) as i64,
                );
            }
        }
        output.layers[layer - 1] = contacts.nodes;
    }
    statistics(&mut output, arena, scale);
    Ok(output)
}

fn statistics(output: &mut ContactNodes, arena: &NodeArena, scale: CoordinateScale) {
    let count = output.layers.iter().map(Vec::len).sum::<usize>();
    if count == 0 {
        return;
    }
    output.average_per_layer = count
        / output
            .layers
            .iter()
            .filter(|nodes| !nodes.is_empty())
            .count();
    let (mut mx, mut my, mut mxy, mut mx2) = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
    for &id in output.layers.iter().flatten() {
        let position = arena.nodes[id].position;
        let x = (f64::from(position.x() as f32) * scale.factor()) as f32;
        let y = (f64::from(position.y() as f32) * scale.factor()) as f32;
        mx += x;
        my += y;
        mxy += x * y;
        mx2 += x * x;
    }
    output.angle = (count as f32 * mxy - mx * my).atan2(count as f32 * mx2 - mx * mx);
}
