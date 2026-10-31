//! `ExtrusionEntity::{length,total_volume}` before path simplification.
//! Volume uses the original polyline, not fitted arc/move lengths.
use crate::{
    geometry::CoordinateScale,
    project_slice::{
        fill_entities::{FillExtrusionCollection, FillExtrusionEntity, LayerFillEntities},
        perimeters::classic::{
            entity_collections::{ExtrusionEntity, ExtrusionEntityCollection},
            gap_extrusion::GapFillEntity,
            materialize::ExtrusionPath,
        },
    },
};

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct CollectionVolume {
    pub(in crate::project_slice) id: usize,
    pub(in crate::project_slice) region: usize,
    pub(in crate::project_slice) role: crate::ExtrusionRole,
    pub(in crate::project_slice) infill: bool,
    pub(in crate::project_slice) solid: bool,
    pub(in crate::project_slice) volume: f64,
}

#[derive(Debug, Default, PartialEq)]
pub(in crate::project_slice) struct LayerVolumes {
    pub(in crate::project_slice) perimeters: Vec<CollectionVolume>,
    pub(in crate::project_slice) infills: Vec<CollectionVolume>,
}

pub(super) fn collect(layer: &LayerFillEntities, scale: CoordinateScale) -> LayerVolumes {
    let perimeters = layer
        .perimeter_ranges
        .iter()
        .enumerate()
        .flat_map(|(region, range)| {
            range.clone().map(move |id| {
                let entity = &layer.perimeters[id];
                collection(
                    id,
                    region,
                    entity.entities.iter().map(super::wall_role),
                    perimeter(entity, scale),
                )
            })
        })
        .collect();
    let mut infills = layer
        .collection_ranges
        .iter()
        .flat_map(|(region, range)| {
            range.clone().map(move |id| {
                let entity = &layer.collections[id];
                collection(
                    id,
                    *region,
                    entity.entities.iter().map(super::fill_role),
                    fill(entity, scale),
                )
            })
        })
        .collect::<Vec<_>>();
    for (region, range) in layer.thin_fill_ranges.iter().enumerate() {
        for index in range.clone() {
            let entity = &layer.thin_fills[index];
            infills.push(collection(
                layer.collections.len() + index,
                region,
                std::iter::once(super::gap_role(entity)),
                gap(entity, scale),
            ));
        }
    }
    LayerVolumes {
        perimeters,
        infills,
    }
}

fn collection(
    id: usize,
    region: usize,
    roles: impl Iterator<Item = crate::ExtrusionRole>,
    volume: f64,
) -> CollectionVolume {
    let (role, infill, solid) = roles.fold(
        (crate::ExtrusionRole::None, false, false),
        |(current, infill, solid), role| {
            (
                if current == crate::ExtrusionRole::None || current == role {
                    role
                } else {
                    crate::ExtrusionRole::Mixed
                },
                infill || role.is_infill(),
                solid || role.is_solid_infill(),
            )
        },
    );
    CollectionVolume {
        id,
        region,
        role,
        infill,
        solid,
        volume,
    }
}

fn perimeter(collection: &ExtrusionEntityCollection, scale: CoordinateScale) -> f64 {
    collection
        .entities
        .iter()
        .map(|entity| match entity {
            ExtrusionEntity::Loop(loop_) => loop_
                .extrusion_loop
                .paths
                .iter()
                .map(|p| path(p, scale))
                .sum(),
            ExtrusionEntity::MultiPath(multi) => multi.paths.iter().map(|p| path(p, scale)).sum(),
            ExtrusionEntity::Path(p) => path(p, scale),
        })
        .sum()
}

fn fill(collection: &FillExtrusionCollection, scale: CoordinateScale) -> f64 {
    collection
        .entities
        .iter()
        .map(|entity| match entity {
            FillExtrusionEntity::Path(p) | FillExtrusionEntity::Oriented(p) => {
                let length: f64 = p
                    .polyline
                    .points()
                    .windows(2)
                    .map(|pair| {
                        let dx = (pair[1].x() - pair[0].x()) as f64;
                        let dy = (pair[1].y() - pair[0].y()) as f64;
                        (dx * dx + dy * dy).sqrt()
                    })
                    .sum();
                p.mm3_per_mm * (length * scale.factor())
            }
            FillExtrusionEntity::VariableWidth(entity) => gap(entity, scale),
        })
        .sum()
}

fn gap(entity: &GapFillEntity, scale: CoordinateScale) -> f64 {
    match entity {
        GapFillEntity::Path(p) => path(p, scale),
        GapFillEntity::Loop(paths) => paths.iter().map(|p| path(p, scale)).sum(),
    }
}

fn path(path: &ExtrusionPath, scale: CoordinateScale) -> f64 {
    let length: f64 = path
        .polyline
        .points
        .windows(2)
        .map(|pair| {
            let dx = (pair[1].x - pair[0].x) as f64;
            let dy = (pair[1].y - pair[0].y) as f64;
            let dz = (pair[1].z - pair[0].z) as f64;
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum();
    path.mm3_per_mm * (length * scale.factor())
}
