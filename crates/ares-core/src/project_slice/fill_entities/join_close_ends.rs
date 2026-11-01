//! FillBase.cpp:213–215: shared non-Arachne sparse-fill materialization gate.
use super::{FillExtrusionCollection, FillExtrusionEntity};
use crate::geometry::{CoordinateScale, Point, Polyline};
use crate::project_slice::perimeters::classic::materialize::Polyline3;

pub(super) fn join(collection: &mut FillExtrusionCollection, spacing: f64, scale: CoordinateScale) {
    if collection.no_sort {
        return;
    }
    let mut entities = std::mem::take(&mut collection.entities)
        .into_iter()
        .peekable();
    while let Some(entity) = entities.next() {
        let (mut template, oriented) = match entity {
            FillExtrusionEntity::Path(p)
                if p.role == crate::ExtrusionRole::InternalInfill && !p.z_contoured =>
            {
                (p, false)
            }
            FillExtrusionEntity::Oriented(p)
                if p.role == crate::ExtrusionRole::InternalInfill && !p.z_contoured =>
            {
                (p, true)
            }
            other => {
                collection.entities.push(other);
                continue;
            }
        };
        let mut paths = vec![Polyline::new(
            template
                .polyline
                .points()
                .iter()
                .map(|p| Point::new(p.x(), p.y()))
                .collect(),
        )];
        while entities.peek().is_some_and(|e| match e {
            FillExtrusionEntity::Path(p) if !oriented => {
                p.role == template.role
                    && !p.z_contoured
                    && p.width == template.width
                    && p.height == template.height
                    && p.mm3_per_mm == template.mm3_per_mm
            }
            FillExtrusionEntity::Oriented(p) if oriented => {
                p.role == template.role
                    && !p.z_contoured
                    && p.width == template.width
                    && p.height == template.height
                    && p.mm3_per_mm == template.mm3_per_mm
            }
            _ => false,
        }) {
            let p = match entities.next().unwrap() {
                FillExtrusionEntity::Path(p) | FillExtrusionEntity::Oriented(p) => p,
                FillExtrusionEntity::VariableWidth(_) => unreachable!(),
            };
            paths.push(Polyline::new(
                p.polyline
                    .points()
                    .iter()
                    .map(|p| Point::new(p.x(), p.y()))
                    .collect(),
            ));
        }
        for polyline in crate::fill::join_close_ends::join_close_ends(
            paths,
            2.0 * spacing / scale.factor(),
            !oriented,
            1e-4 / scale.factor(),
        ) {
            template.polyline = Polyline3::from_xy(polyline, Vec::new());
            collection.entities.push(if oriented {
                FillExtrusionEntity::Oriented(template.clone())
            } else {
                FillExtrusionEntity::Path(template.clone())
            });
        }
    }
}
