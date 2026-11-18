//! `GCode.cpp:6304–6457`: original-instance search before tool tours consume paths.

mod object;
mod support;

use super::*;
use crate::{
    ExtrusionRole,
    geometry::Point,
    project_slice::{
        fill_entities::FillExtrusionEntity,
        island_print_order::IslandPrintEntity,
        perimeters::classic::{
            gap_extrusion::GapFillEntity,
            materialize::{ExtrusionRole as WallRole, Polyline3},
        },
    },
};

type PathOrder = (usize, usize, usize, usize, usize);
type Placement = (usize, usize, Point);

#[derive(Clone, Copy)]
pub(super) struct Candidate {
    pub(super) point: Point,
    // Schedule construction can refresh the filament map.
    pub(super) filament: usize,
    distance_squared: i64,
    order: (PathOrder, usize),
}

#[derive(Default)]
struct Maximum(Option<Candidate>);

impl Maximum {
    fn consider(&mut self, next: Candidate) {
        // Island tours reorder collections; retained source keys recover
        // the LayerRegion traversal's strict first-wins ties.
        if self.0.is_none_or(|old| {
            next.distance_squared > old.distance_squared
                || (next.distance_squared == old.distance_squared && next.order < old.order)
        }) {
            self.0 = Some(next);
        }
    }

    fn collect(&mut self, polyline: &Polyline3, shift: Point, filament: usize, order: PathOrder) {
        let mut consider = |index: usize| {
            let p = &polyline.points[index];
            let point = Point::new(p.x + shift.x(), p.y + shift.y());
            self.consider(Candidate {
                point,
                filament,
                distance_squared: point.x() * point.x() + point.y() * point.y(),
                order: (order, index),
            });
        };
        if polyline.fitting.is_empty() {
            for i in 0..polyline.points.len() {
                consider(i);
            }
            return;
        }
        consider(polyline.fitting[0].start);
        for fitted in &polyline.fitting {
            let start = if fitted.arc.is_some() {
                fitted.end
            } else {
                fitted.start
            };
            for i in start..=fitted.end {
                consider(i);
            }
        }
    }
}

#[derive(Default)]
struct Maxima {
    external: Maximum,
    fallback: Maximum,
}

impl Maxima {
    fn for_object_layer(
        traversal: &PreparedPostClassicTraversal,
        object: usize,
        layer: &OrderedExtrusionLayer,
        copies: &[Instance],
    ) -> Self {
        let mut maxima = Self::default();
        for (copy, instance) in copies.iter().enumerate() {
            object::collect(
                &mut maxima,
                traversal,
                layer,
                (object, copy, instance.shift),
            );
        }
        maxima
    }

    fn merge(&mut self, other: &Self) {
        if let Some(candidate) = other.external.0 {
            self.external.consider(candidate);
        }
        if let Some(candidate) = other.fallback.0 {
            self.fallback.consider(candidate);
        }
    }
}

// GCode.cpp:6390–6435 scans fitted paths before :8547 changes the seam.
pub(in crate::project_slice::gcode_emit) struct ObjectTargets(Vec<Vec<Maxima>>);

impl ObjectTargets {
    pub(in crate::project_slice::gcode_emit) fn capture(
        traversal: &PreparedPostClassicTraversal,
        objects: &[Vec<OrderedExtrusionLayer>],
        instances: &[Vec<Instance>],
    ) -> Option<Self> {
        let full = &traversal.resolved.views.full;
        let enabled = full.printer.gcode.farthest_point_timelapse.0
            && full.process.print.timelapse_type == crate::ProcessTimelapseType::Traditional
            && traversal.resolved.views.runtime_gcode.printer_structure
                != crate::PrinterStructure::I3;
        enabled.then(|| {
            Self(
                objects
                    .iter()
                    .zip(instances)
                    .enumerate()
                    .map(|(object, (layers, copies))| {
                        layers
                            .iter()
                            .map(|layer| Maxima::for_object_layer(traversal, object, layer, copies))
                            .collect()
                    })
                    .collect(),
            )
        })
    }

    pub(super) fn select(
        &self,
        traversal: &PreparedPostClassicTraversal,
        views: LayerViews<'_>,
        tools: &LayerTools,
    ) -> Option<Candidate> {
        let mut maxima = Maxima::default();
        for entry in views.entries {
            let object = entry.object_index;
            if let Some(layer) = entry.object_layer {
                maxima.merge(&self.0[object][layer]);
            }
            let support_layer = entry
                .support_layer
                .map(|index| &views.support[object][index]);
            let raft_layer = entry.raft_layer.map(|index| &views.raft[object][index]);
            let support_tools = support::tools(traversal, object, support_layer, raft_layer, tools);
            for (copy, instance) in views.instances[object].iter().enumerate() {
                support::collect(
                    &mut maxima.fallback,
                    (support_layer, raft_layer),
                    &support_tools,
                    (object, copy, instance.shift),
                );
            }
        }
        maxima
            .external
            .0
            .filter(|candidate| candidate.distance_squared > 0)
            .or(maxima
                .fallback
                .0
                .filter(|candidate| candidate.distance_squared > 0))
    }
}

fn visit_fill(entity: &IslandPrintEntity, visit: &mut impl FnMut(&Polyline3, ExtrusionRole)) {
    match entity {
        IslandPrintEntity::Fill(entity) => fill(entity, visit),
        IslandPrintEntity::FillCollection(collection) => {
            for entity in &collection.entities {
                fill(entity, visit);
            }
        }
        IslandPrintEntity::Thin(entity) => gap(entity, visit),
        IslandPrintEntity::Perimeter(_) => unreachable!("fill visitor receives fill entities"),
    }
}

fn fill(entity: &FillExtrusionEntity, visit: &mut impl FnMut(&Polyline3, ExtrusionRole)) {
    match entity {
        FillExtrusionEntity::Path(path) | FillExtrusionEntity::Oriented(path) => {
            visit(&path.polyline, path.role)
        }
        FillExtrusionEntity::VariableWidth(entity) => gap(entity, visit),
    }
}

fn gap(entity: &GapFillEntity, visit: &mut impl FnMut(&Polyline3, ExtrusionRole)) {
    let paths = match entity {
        GapFillEntity::Path(path) => std::slice::from_ref(path),
        GapFillEntity::Loop(paths) => paths,
    };
    for path in paths {
        let role = match path.role {
            WallRole::SolidInfill => ExtrusionRole::SolidInfill,
            WallRole::TopSolidInfill => ExtrusionRole::TopSolidInfill,
            WallRole::ExternalPerimeter
            | WallRole::Perimeter
            | WallRole::OverhangPerimeter
            | WallRole::GapFill
            | WallRole::BottomSurface => continue,
        };
        visit(&path.polyline, role);
    }
}
