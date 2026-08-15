use crate::{
    SliceError,
    geometry::{CoordinateScale, ExPolygon, JoinType, offset_expolygon},
    project_slice::group_fills::SurfaceFill,
};

use super::{FillExtrusionCollection, FillExtrusionPath, LayerFillEntities, geometry_error};

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let spacing = scale
        .checked_scale(fill.params.spacing * 0.5)
        .ok_or_else(|| SliceError::InvalidInput("concentric spacing is out of range".into()))?;
    if spacing <= 0 {
        return Err(SliceError::InvalidInput(
            "concentric spacing must be positive".into(),
        ));
    }
    let mut paths = Vec::new();
    let attributes = FillAttributes {
        role: fill.params.extrusion_role,
        mm3_per_mm: fill.params.flow.mm3_per_mm,
        width: fill.params.flow.width,
        height: fill.params.flow.height,
    };
    for expolygon in fill.expolygons {
        append_rings(&mut paths, expolygon, spacing as f32, attributes)?;
    }
    if !paths.is_empty() {
        output.collections.push(FillExtrusionCollection {
            paths,
            no_sort: true,
        });
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct FillAttributes {
    role: crate::ExtrusionRole,
    mm3_per_mm: f64,
    width: f32,
    height: f32,
}

fn append_rings(
    output: &mut Vec<FillExtrusionPath>,
    current: ExPolygon,
    spacing: f32,
    attributes: FillAttributes,
) -> Result<(), SliceError> {
    let mut current = vec![current];
    loop {
        let mut next = Vec::new();
        for expolygon in current {
            append_expolygon(output, &expolygon, attributes);
            next.extend(
                offset_expolygon(&expolygon, -spacing, JoinType::Miter, 3.0)
                    .map_err(geometry_error)?,
            );
        }
        if next.is_empty() {
            return Ok(());
        }
        current = next;
    }
}

fn append_expolygon(
    output: &mut Vec<FillExtrusionPath>,
    expolygon: &ExPolygon,
    attributes: FillAttributes,
) {
    append_polygon(output, expolygon.contour(), attributes);
    for hole in expolygon.holes() {
        append_polygon(output, hole, attributes);
    }
}

fn append_polygon(
    output: &mut Vec<FillExtrusionPath>,
    polygon: &crate::geometry::Polygon,
    attributes: FillAttributes,
) {
    let polyline = polygon.split_at_first_point();
    if polyline.is_valid() {
        output.push(FillExtrusionPath {
            polyline,
            role: attributes.role,
            mm3_per_mm: attributes.mm3_per_mm,
            width: attributes.width,
            height: attributes.height,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ExtrusionRole,
        geometry::{Point, Polygon},
    };

    #[test]
    fn concentric_rings_follow_the_source_surface_until_the_area_is_consumed() {
        let expolygon = ExPolygon::new(
            Polygon::new(vec![
                Point::new(0, 0),
                Point::new(100_000, 0),
                Point::new(100_000, 100_000),
                Point::new(0, 100_000),
            ]),
            Vec::new(),
        );
        let mut output = Vec::new();
        append_rings(
            &mut output,
            expolygon,
            10_000.0,
            FillAttributes {
                role: ExtrusionRole::InternalInfill,
                mm3_per_mm: 1.0,
                width: 10.0,
                height: 2.0,
            },
        )
        .unwrap();
        assert!(output.len() > 1);
        assert_eq!(
            output.first().unwrap().polyline.points().first(),
            Some(&Point::new(0, 0))
        );
        assert!(
            output
                .iter()
                .all(|path| path.polyline.points().first() == path.polyline.points().last())
        );
    }
}
