use crate::{
    SliceError,
    arachne::wall_toolpaths::{RawWallToolPathConfig, generate},
    geometry::{CoordinateScale, ExPolygon, ThickPolyline},
    project_slice::{
        group_fills::SurfaceFill,
        perimeters::classic::{
            gap_extrusion::variable_width, materialize::ExtrusionRole as MaterializedRole,
        },
    },
};

use super::LayerFillEntities;

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    minimum_nozzle_diameter: f64,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let spacing = scale
        .checked_scale(fill.params.spacing)
        .ok_or_else(|| SliceError::InvalidInput("concentric spacing is out of range".into()))?;
    if spacing <= 0 {
        return Err(SliceError::InvalidInput(
            "concentric spacing must be positive".into(),
        ));
    }
    let mut polylines = Vec::new();
    for expolygon in fill.no_overlap_expolygons {
        polylines.extend(generate_thick_polylines(
            expolygon,
            spacing,
            scale
                .checked_scale(f64::from(fill.params.flow.height))
                .unwrap(),
            minimum_nozzle_diameter,
            scale,
        )?);
    }
    let converted = variable_width::convert_with_role(
        &polylines,
        fill.params.flow,
        scale,
        MaterializedRole::SolidInfill,
    )?;
    output.thin_fills.extend(converted.entities);
    Ok(())
}

fn generate_thick_polylines(
    expolygon: ExPolygon,
    spacing: i64,
    layer_height: i64,
    minimum_nozzle_diameter: f64,
    scale: CoordinateScale,
) -> Result<Vec<ThickPolyline>, SliceError> {
    let (contour, holes) = expolygon.into_parts();
    let (minimum_x, maximum_x, minimum_y, maximum_y) = contour.points().iter().fold(
        (i64::MAX, i64::MIN, i64::MAX, i64::MIN),
        |(minimum_x, maximum_x, minimum_y, maximum_y), point| {
            (
                minimum_x.min(point.x()),
                maximum_x.max(point.x()),
                minimum_y.min(point.y()),
                maximum_y.max(point.y()),
            )
        },
    );
    let loops_count =
        usize::try_from((maximum_x - minimum_x).max(maximum_y - minimum_y) / spacing + 1).unwrap();
    let mut polygons = Vec::with_capacity(1 + holes.len());
    polygons.push(contour);
    polygons.extend(holes);
    let generated = generate(
        &polygons,
        RawWallToolPathConfig {
            outer_spacing: spacing,
            inner_spacing: spacing,
            inset_count: loops_count,
            outer_wall_inset: 0,
            layer_height,
            min_bead_width: scale.checked_scale(0.85 * minimum_nozzle_diameter).unwrap(),
            min_feature_size: scale.checked_scale(0.25 * minimum_nozzle_diameter).unwrap(),
            transition_length: scale.checked_scale(0.4).unwrap(),
            transitioning_angle: 10.0_f64.to_radians(),
            transition_filter_deviation: scale
                .checked_scale(0.25 * minimum_nozzle_diameter)
                .unwrap(),
            wall_distribution_count: 1,
            min_length_factor: 0.5,
            is_top_or_bottom_layer: false,
            coordinate_scale: scale,
        },
    )
    .map_err(|error| {
        SliceError::InvalidInput(format!("concentric Arachne generation failed: {error:?}"))
    })?;
    Ok(generated
        .toolpaths
        .into_iter()
        .flatten()
        .filter(|line| line.junctions.len() >= 2)
        .map(|line| line.to_thick_polyline())
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon};

    use super::generate_thick_polylines;

    #[test]
    fn task22o200_concentric_internal_generates_positive_variable_width_loops() {
        let scale = CoordinateScale::Normal;
        let scaled = |value| scale.checked_scale(value).unwrap();
        let expolygon = ExPolygon::new(
            Polygon::new(vec![
                Point::new(0, 0),
                Point::new(scaled(10.0), 0),
                Point::new(scaled(10.0), scaled(10.0)),
                Point::new(0, scaled(10.0)),
            ]),
            Vec::new(),
        );

        let output =
            generate_thick_polylines(expolygon, scaled(0.4), scaled(0.2), 0.4, scale).unwrap();

        assert!(output.len() > 1);
        assert!(output.iter().all(|line| {
            line.points.len() >= 2
                && line.width.len() == 2 * (line.points.len() - 1)
                && line.width.iter().all(|width| *width > 0.0)
        }));
    }
}
