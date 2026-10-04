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

#[derive(Debug, Default, PartialEq)]
pub(in crate::project_slice) struct LayerVolumes {
    pub(in crate::project_slice) perimeters: Vec<f64>,
    pub(in crate::project_slice) infills: Vec<f64>,
}

pub(super) fn collect(layer: &LayerFillEntities, scale: CoordinateScale) -> LayerVolumes {
    LayerVolumes {
        perimeters: layer
            .perimeters
            .iter()
            .map(|p| perimeter(p, scale))
            .collect(),
        infills: layer
            .collections
            .iter()
            .map(|c| fill(c, scale))
            .chain(layer.thin_fills.iter().map(|e| gap(e, scale)))
            .collect(),
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
    fn source_volume_uses_three_dimensional_polyline_not_fitted_arc_length() {
        let extrusion = ExtrusionPath {
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
                polyline: crate::geometry::Polyline::new(vec![
                    crate::geometry::Point::new(0, 0),
                    crate::geometry::Point::new(length, 0),
                ]),
                fitting: Vec::new(),
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
            ..Default::default()
        };
        let volumes = collect(&layer, CoordinateScale::Normal);
        assert_eq!(volumes.infills, vec![5.0, 10.0]);
        assert!(volumes.perimeters.is_empty());
    }
}
