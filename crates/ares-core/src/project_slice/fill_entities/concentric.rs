use crate::{
    SliceError,
    arachne::wall_toolpaths::{RawWallToolPathConfig, generate},
    geometry::{
        CoordinateScale, ExPolygon, Point, ThickPolyline, intersection_ex_with_safety_offset,
    },
    project_slice::{
        group_fills::SurfaceFill,
        perimeters::classic::{
            gap_extrusion::variable_width, materialize::ExtrusionRole as MaterializedRole,
            shortest_path::reorder_thick_polylines,
        },
    },
};

use super::{LayerFillEntities, geometry_error};

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
    for expolygon in fill.expolygons {
        let mut polylines = Vec::new();
        for domain in intersect_no_overlap_domains(&fill.no_overlap_expolygons, &expolygon)? {
            let first_polyline = polylines.len();
            polylines.extend(generate_thick_polylines(
                domain,
                spacing,
                scale
                    .checked_scale(f64::from(fill.params.flow.height))
                    .unwrap(),
                minimum_nozzle_diameter,
                scale,
            )?);
            finalize_polylines(
                &mut polylines,
                first_polyline,
                fill.params.loop_clipping as f64,
            );
        }
        let mut converted = variable_width::convert_with_role(
            &polylines,
            fill.params.flow,
            scale,
            MaterializedRole::SolidInfill,
        )?;
        for entity in &mut converted.entities {
            match entity {
                crate::project_slice::perimeters::classic::gap_extrusion::GapFillEntity::Path(
                    path,
                ) => {
                    path.can_reverse = false;
                }
                crate::project_slice::perimeters::classic::gap_extrusion::GapFillEntity::Loop(
                    paths,
                ) => {
                    for path in paths {
                        path.can_reverse = false;
                    }
                }
            }
        }
        output.thin_fills.extend(converted.entities);
    }
    Ok(())
}

fn intersect_no_overlap_domains(
    no_overlap: &[ExPolygon],
    expolygon: &ExPolygon,
) -> Result<Vec<ExPolygon>, SliceError> {
    intersection_ex_with_safety_offset(no_overlap, std::slice::from_ref(expolygon))
        .map_err(geometry_error)
}

fn finalize_polylines(
    polylines: &mut Vec<ThickPolyline>,
    first_polyline: usize,
    loop_clipping: f64,
) {
    for polyline in &mut polylines[first_polyline..] {
        if polyline.points.first() == polyline.points.last()
            && polyline.width.first() == polyline.width.last()
        {
            let nearest = polyline
                .points
                .iter()
                .enumerate()
                .min_by_key(|(_, point)| squared_distance(**point, Point::new(0, 0)))
                .unwrap()
                .0;
            polyline.start_at_index(nearest);
        }
    }
    let mut write_index = first_polyline;
    for read_index in first_polyline..polylines.len() {
        polylines[read_index].clip_end(loop_clipping);
        if polylines[read_index].points.len() >= 2 {
            polylines.swap(write_index, read_index);
            write_index += 1;
        }
    }
    polylines.truncate(write_index);
    reorder_thick_polylines(polylines);
}

fn squared_distance(left: Point, right: Point) -> i128 {
    let dx = i128::from(left.x() - right.x());
    let dy = i128::from(left.y() - right.y());
    dx * dx + dy * dy
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
    let config = RawWallToolPathConfig {
        outer_spacing: spacing,
        inner_spacing: spacing,
        inset_count: loops_count,
        outer_wall_inset: 0,
        layer_height,
        min_bead_width: scale.checked_scale(0.85 * minimum_nozzle_diameter).unwrap(),
        min_feature_size: scale.checked_scale(0.25 * minimum_nozzle_diameter).unwrap(),
        transition_length: scale.checked_scale(0.4).unwrap(),
        transitioning_angle: f64::from((std::f32::consts::PI * 10.0_f32) / 180.0_f32),
        transition_filter_deviation: scale.checked_scale(0.25 * minimum_nozzle_diameter).unwrap(),
        wall_distribution_count: 1,
        min_length_factor: 0.5,
        is_top_or_bottom_layer: false,
        coordinate_scale: scale,
    };
    let generated = generate(&polygons, config).map_err(|error| {
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
mod tests;
