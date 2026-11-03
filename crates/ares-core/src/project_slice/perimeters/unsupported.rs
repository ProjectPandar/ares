//! `PerimeterGenerator.cpp:1391-1455`: stable unsupported tail and attachment waves.

use crate::geometry::{Line, LineDistanceTree};

use super::classic::{
    entity_collections::{ExtrusionEntity, ExtrusionEntityCollection},
    materialize::ExtrusionRole,
};

fn is_unsupported_loop(entity: &ExtrusionEntity) -> bool {
    matches!(entity, ExtrusionEntity::Loop(loop_) if !loop_.extrusion_loop.paths.is_empty()
        && loop_.extrusion_loop.paths.iter().all(|path| path.role == ExtrusionRole::OverhangPerimeter))
}

pub(super) fn apply(collection: &mut ExtrusionEntityCollection, touch_distance: f64) {
    let mut deferred: Vec<_> = collection
        .entities
        .extract_if(.., |entity| is_unsupported_loop(entity))
        .collect();
    if deferred.is_empty() {
        return;
    }
    deferred.sort_by_key(|entity| std::cmp::Reverse(entity.inset_idx()));
    let mut anchored = Vec::new();
    for entity in &collection.entities {
        collect_lines(entity, &mut anchored);
    }
    let mut unattached: Vec<_> = deferred.iter_mut().map(Some).collect();
    while !anchored.is_empty() {
        let lines = std::mem::take(&mut anchored);
        let distancer = LineDistanceTree::new(&lines);
        for entity in &mut unattached {
            let Some(loop_) = entity.as_deref() else {
                continue;
            };
            if loop_.as_polyline().iter().any(|&point| {
                distancer.nearest(point).unwrap().squared_distance.sqrt() < touch_distance
            }) {
                collect_lines(loop_, &mut anchored);
                *entity = None;
            }
        }
    }
    for entity in unattached.into_iter().flatten() {
        let ExtrusionEntity::Loop(loop_) = entity else {
            unreachable!("the unsupported partition contains only loops");
        };
        loop_.extrusion_loop.print_after_infill = true;
    }
    collection.entities.append(&mut deferred);
}

fn collect_lines(entity: &ExtrusionEntity, out: &mut Vec<Line>) {
    out.extend(entity.as_polyline().windows(2).map(|points| Line {
        a: points[0],
        b: points[1],
    }));
}
