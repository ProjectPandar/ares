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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_slice::perimeters::classic::materialize::{
        ExtrusionRole, FittedArc, FittedMove, Point3, Polyline3,
    };

    #[test]
    fn original_collection_roles_and_region_owners_survive_empty_and_mixed_fills() {
        let entities = [
            crate::ExtrusionRole::InternalInfill,
            crate::ExtrusionRole::TopSolidInfill,
        ]
        .map(|role| {
            FillExtrusionEntity::Path(crate::project_slice::fill_entities::FillExtrusionPath {
                polyline: Polyline3::from_xy(
                    crate::geometry::Polyline::new(vec![
                        crate::geometry::Point::new(0, 0),
                        crate::geometry::Point::new(1_000_000, 0),
                    ]),
                    Vec::new(),
                ),
                z_contoured: false,
                role,
                mm3_per_mm: 1.0,
                width: 0.5,
                height: 0.2,
            })
        });
        let layer = LayerFillEntities {
            collections: vec![
                FillExtrusionCollection {
                    entities: Vec::new(),
                    no_sort: false,
                },
                FillExtrusionCollection {
                    entities: entities.into(),
                    no_sort: false,
                },
            ],
            collection_ranges: vec![(1, 0..1), (2, 1..2)],
            ..Default::default()
        };
        let captured = collect(&layer, CoordinateScale::Normal);
        assert!(!captured.infills[0].infill && !captured.infills[0].solid);
        assert!(captured.infills[1].infill && captured.infills[1].solid);
        assert_eq!(
            (
                captured.infills[0].id,
                captured.infills[0].region,
                captured.infills[0].role,
                captured.infills[0].volume
            ),
            (0, 1, crate::ExtrusionRole::None, 0.0)
        );
        assert_eq!(
            (
                captured.infills[1].id,
                captured.infills[1].region,
                captured.infills[1].role,
                captured.infills[1].volume
            ),
            (1, 2, crate::ExtrusionRole::Mixed, 2.0)
        );
    }

    #[test]
    fn source_volume_uses_three_dimensional_polyline_not_fitted_arc_length() {
        let extrusion = ExtrusionPath {
            z_contoured: false,
            polyline: Polyline3 {
                points: vec![
                    Point3 { x: 0, y: 0, z: 0 },
                    Point3 {
                        x: 3_000_000,
                        y: 0,
                        z: 4_000_000,
                    },
                ],
                fitting: vec![FittedMove {
                    start: 0,
                    end: 1,
                    arc: Some(FittedArc {
                        center: (0.0, 0.0),
                        radius: 1000.0,
                        length: 1000.0,
                        clockwise: false,
                    }),
                }],
            },
            role: ExtrusionRole::Perimeter,
            can_reverse: false,
            mm3_per_mm: 2.0,
            width: 0.5,
            height: 0.2,
        };
        assert_eq!(path(&extrusion, CoordinateScale::Normal), 10.0);
        // Native double unscale multiplication retains this rounding bit.
        assert_eq!(
            path(&extrusion, CoordinateScale::LargeBed),
            100.000_000_000_000_01
        );
        let fills = [1_000_000, 4_000_000].map(|length| {
            FillExtrusionEntity::Oriented(crate::project_slice::fill_entities::FillExtrusionPath {
                polyline: Polyline3::from_xy(
                    crate::geometry::Polyline::new(vec![
                        crate::geometry::Point::new(0, 0),
                        crate::geometry::Point::new(length, 0),
                    ]),
                    Vec::new(),
                ),
                z_contoured: false,
                role: crate::ExtrusionRole::InternalInfill,
                mm3_per_mm: 1.0,
                width: 0.5,
                height: 0.2,
            })
        });
        let layer = LayerFillEntities {
            collections: vec![FillExtrusionCollection {
                entities: fills.into(),
                no_sort: false,
            }],
            thin_fills: vec![GapFillEntity::Path(extrusion)],
            collection_ranges: vec![(0, 0..1)],
            thin_fill_ranges: std::iter::once(0..1).collect(),
            ..Default::default()
        };
        let volumes = collect(&layer, CoordinateScale::Normal);
        assert_eq!(
            volumes
                .infills
                .iter()
                .map(|entry| entry.volume)
                .collect::<Vec<_>>(),
            vec![5.0, 10.0]
        );
        assert_eq!(
            volumes
                .infills
                .iter()
                .map(|entry| (entry.id, entry.region))
                .collect::<Vec<_>>(),
            vec![(0, 0), (1, 0)]
        );
        assert!(volumes.perimeters.is_empty());
    }
}
