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

pub(super) struct Candidate {
    pub(super) point: Point,
    pub(super) physical_extruder: usize,
    distance_squared: i64,
    order: (PathOrder, usize),
}

#[derive(Default)]
struct Maximum(Option<Candidate>);

impl Maximum {
    fn collect(&mut self, polyline: &Polyline3, shift: Point, physical: usize, order: PathOrder) {
        let mut consider = |index: usize| {
            let p = &polyline.points[index];
            let point = Point::new(p.x + shift.x(), p.y + shift.y());
            let distance_squared = point.x() * point.x() + point.y() * point.y();
            let order = (order, index);
            // Island tours reorder collections; retained source keys recover
            // the LayerRegion traversal's strict first-wins ties.
            if self.0.as_ref().is_none_or(|old| {
                distance_squared > old.distance_squared
                    || (distance_squared == old.distance_squared && order < old.order)
            }) {
                self.0 = Some(Candidate {
                    point,
                    physical_extruder: physical,
                    distance_squared,
                    order,
                });
            }
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

struct Search<'a> {
    traversal: &'a PreparedPostClassicTraversal,
    external: Maximum,
    fallback: Maximum,
}

impl Search<'_> {
    fn physical(&self, filament: usize) -> usize {
        (self
            .traversal
            .resolved
            .views
            .full
            .project
            .gcode
            .filament_map
            .0[filament]
            .0
            - 1) as usize
    }
}

pub(super) fn compute(
    traversal: &PreparedPostClassicTraversal,
    views: LayerViews<'_>,
    tools: &LayerTools,
) -> Option<Candidate> {
    let LayerViews {
        entries,
        objects,
        support: supports,
        raft,
        instances,
    } = views;
    let mut search = Search {
        traversal,
        external: Maximum::default(),
        fallback: Maximum::default(),
    };
    for entry in entries {
        let object = entry.object_index;
        let support_layer = entry.support_layer.map(|index| &supports[object][index]);
        let raft_layer = entry.raft_layer.map(|index| &raft[object][index]);
        let support_tools = support::tools(traversal, object, support_layer, raft_layer, tools);
        for (copy, instance) in instances[object].iter().enumerate() {
            let placement = (object, copy, instance.shift);
            if let Some(layer) = entry.object_layer {
                object::collect(&mut search, &objects[object][layer], placement);
            }
            support::collect(
                &mut search,
                (support_layer, raft_layer),
                &support_tools,
                placement,
            );
        }
    }
    search
        .external
        .0
        .filter(|candidate| candidate.distance_squared > 0)
        .or_else(|| {
            search
                .fallback
                .0
                .filter(|candidate| candidate.distance_squared > 0)
        })
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
