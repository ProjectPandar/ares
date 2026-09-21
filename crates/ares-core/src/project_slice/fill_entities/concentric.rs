use crate::{
    SliceError,
    arachne::wall_toolpaths::{RawWallToolPathConfig, generate},
    geometry::{
        CoordinateScale, ExPolygon, JoinType, Point, ThickPolyline,
        intersection_ex_with_safety_offset, offset_expolygon,
    },
    project_slice::{
        group_fills::SurfaceFill,
        perimeters::{
            classic::{
                gap_extrusion::variable_width, materialize::ExtrusionRole as MaterializedRole,
                shortest_path::reorder_thick_polylines,
            },
            flow::with_spacing,
        },
    },
};

mod generate;

use super::{FillExtrusionCollection, FillExtrusionEntity, LayerFillEntities, geometry_error};
use generate::{generate_standard_thick_polylines, generate_thick_polylines};

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    minimum_nozzle_diameter: f64,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    if let Ok(path) = std::env::var("ARES_DUMP_CI") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(file, "CALL");
        }
    }
    let spacing = scale
        .checked_scale(f64::from(fill.params.flow.spacing))
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
            let generated = generate_thick_polylines(
                domain,
                spacing,
                scale
                    .checked_scale(f64::from(fill.params.flow.height))
                    .unwrap(),
                minimum_nozzle_diameter,
                scale,
            )?;
            if let Ok(path) = std::env::var("ARES_DUMP_PRECLIP")
                && !generated.is_empty()
            {
                use std::io::Write;
                if let Ok(mut out) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                {
                    for polyline in &generated {
                        let _ = writeln!(
                            out,
                            "PRECLIP n={} w0={}",
                            polyline.points.len(),
                            polyline.width.first().copied().unwrap_or_default()
                        );
                    }
                }
            }
            polylines.extend(generated);
            finalize_polylines(
                &mut polylines,
                first_polyline,
                fill.params.loop_clipping as f64,
                scale,
            );
        }
        let entities = variable_width::convert_with_role(
            {
                if let Ok(path) = std::env::var("ARES_DUMP_MEDIAL")
                    && !polylines.is_empty()
                {
                    use std::io::Write;
                    if let Ok(mut out) = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(path)
                    {
                        for polyline in &polylines {
                            let widths = polyline
                                .width
                                .iter()
                                .map(|width| format!("{width}"))
                                .collect::<Vec<_>>()
                                .join(",");
                            let _ = writeln!(out, "CONC1 W{widths}");
                        }
                    }
                }
                &polylines
            },
            with_spacing(fill.params.flow, fill.params.spacing as f32),
            scale,
            MaterializedRole::SolidInfill,
        )?
        .entities
        .into_iter()
        .map(FillExtrusionEntity::VariableWidth)
        .collect::<Vec<_>>();
        if !entities.is_empty() {
            output.collections.push(FillExtrusionCollection {
                entities,
                no_sort: true,
                simplify_reversed: false,
            });
        }
    }
    Ok(())
}

pub(super) fn append_configured(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    minimum_nozzle_diameter: f64,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let spacing = scale
        .checked_scale(fill.params.spacing)
        .ok_or_else(|| SliceError::InvalidInput("concentric spacing is out of range".into()))?;
    let params = fill.params;
    let kind = fill.representative.kind;
    for expolygon in fill.expolygons {
        let generation_domains = offset_expolygon(
            &expolygon,
            ((params.overlap - 0.5 * params.spacing) / scale.factor()) as f32,
            JoinType::Miter,
            3.0,
        )
        .map_err(geometry_error)?;
        let mut polylines = Vec::new();
        for domain in generation_domains {
            polylines.extend(generate_standard_thick_polylines(
                domain,
                spacing,
                scale.checked_scale(f64::from(params.flow.height)).unwrap(),
                minimum_nozzle_diameter,
                scale,
            )?);
        }
        finalize_standard_polylines(&mut polylines, params.loop_clipping as f64);
        if let Ok(path) = std::env::var("ARES_DUMP_MEDIAL")
            && !polylines.is_empty()
        {
            use std::io::Write;
            if let Ok(mut out) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                for polyline in &polylines {
                    let points = polyline
                        .points
                        .iter()
                        .map(|point| format!("({},{})", point.x(), point.y()))
                        .collect::<Vec<_>>()
                        .join("");
                    let widths = polyline
                        .width
                        .iter()
                        .map(|width| format!("{width}"))
                        .collect::<Vec<_>>()
                        .join(",");
                    let _ = writeln!(out, "CONC P{points} W{widths}");
                }
            }
        }
        let materialized_role = match params.extrusion_role {
            crate::ExtrusionRole::TopSolidInfill => MaterializedRole::TopSolidInfill,
            crate::ExtrusionRole::BottomSurface => MaterializedRole::BottomSurface,
            _ => MaterializedRole::SolidInfill,
        };
        let mut entities = variable_width::convert_with_role(
            &polylines,
            with_spacing(params.flow, params.spacing as f32),
            scale,
            materialized_role,
        )?
        .entities
        .into_iter()
        .map(FillExtrusionEntity::VariableWidth)
        .collect::<Vec<_>>();
        super::gap_residual::append_residual(super::gap_residual::ResidualInput {
            output_entities: &mut entities,
            no_overlap_expolygons: &fill.no_overlap_expolygons,
            params,
            kind,
            expolygon: &expolygon,
            scale,
        })?;
        if !entities.is_empty() {
            output.collections.push(FillExtrusionCollection {
                entities,
                no_sort: true,
                simplify_reversed: false,
            });
        }
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
    scale: CoordinateScale,
) {
    for polyline in &mut polylines[first_polyline..] {
        if polyline.points.first() == polyline.points.last()
            && polyline.width.first() == polyline.width.last()
        {
            if let Ok(path) = std::env::var("ARES_DUMP_CI") {
                use std::io::Write;
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                {
                    let _ = write!(file, "E closed=1 n={}:", polyline.points.len());
                    for point in &polyline.points {
                        let _ = write!(
                            file,
                            " ({:.6},{:.6})",
                            scale.unscale(point.x()),
                            scale.unscale(point.y())
                        );
                    }
                    let _ = writeln!(file);
                }
            }
            if let Ok(path) = std::env::var("ARES_DUMP_CI") {
                use std::io::Write;
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                {
                    let _ = write!(
                        file,
                        "W n={} first={:?} last={:?} match={}",
                        polyline.width.len(),
                        polyline.width.first(),
                        polyline.width.last(),
                        polyline.width.first() == polyline.width.last()
                    );
                    let _ = writeln!(file);
                }
            }
            polyline.points.pop();
            let start = nearest_to_origin(&polyline.points);
            polyline.points.push(polyline.points[0]);
            polyline.start_at_index(start);
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

fn finalize_standard_polylines(polylines: &mut Vec<ThickPolyline>, loop_clipping: f64) {
    let mut last = Point::new(0, 0);
    for polyline in &mut *polylines {
        if polyline.points.first() == polyline.points.last()
            && polyline.width.first() == polyline.width.last()
        {
            polyline.start_at_index(nearest_to(&polyline.points, last));
        }
        if let Some(point) = polyline.points.last() {
            last = *point;
        }
    }
    for polyline in &mut *polylines {
        polyline.clip_end(loop_clipping);
    }
    polylines.retain(|polyline| polyline.points.len() >= 2);
    reorder_thick_polylines(polylines);
}

fn nearest_to_origin(points: &[Point]) -> usize {
    points
        .iter()
        .enumerate()
        .fold((0, i128::MAX), |nearest, (index, point)| {
            let distance = squared_distance(*point, Point::new(0, 0));
            if distance <= nearest.1 {
                (index, distance)
            } else {
                nearest
            }
        })
        .0
}

fn nearest_to(points: &[Point], target: Point) -> usize {
    points
        .iter()
        .enumerate()
        .min_by_key(|(_, point)| squared_distance(**point, target))
        .map_or(0, |(index, _)| index)
}

fn squared_distance(left: Point, right: Point) -> i128 {
    let dx = i128::from(left.x() - right.x());
    let dy = i128::from(left.y() - right.y());
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests;
