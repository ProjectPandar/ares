//! Concentric thick-polyline generation (`FillConcentric.cpp`): wraps the
//! Arachne wall toolpath generator with the two source concentric variants
//! (standard vs. internal) sharing explicit geometry and mode inputs.

use crate::{
    SliceError,
    arachne::wall_toolpaths::{RawWallToolPathConfig, generate},
    geometry::{CoordinateScale, ExPolygon, JoinType, ThickPolyline, offset_expolygon},
};

use super::geometry_error;

pub(super) fn generate_thick_polylines(
    expolygon: ExPolygon,
    spacing: i64,
    layer_height: i64,
    minimum_nozzle_diameter: f64,
    scale: CoordinateScale,
) -> Result<Vec<ThickPolyline>, SliceError> {
    generate_with_mode(
        expolygon,
        spacing,
        layer_height,
        minimum_nozzle_diameter,
        scale,
        false,
    )
}

pub(super) fn generate_standard_thick_polylines(
    expolygon: ExPolygon,
    spacing: i64,
    layer_height: i64,
    minimum_nozzle_diameter: f64,
    scale: CoordinateScale,
) -> Result<Vec<ThickPolyline>, SliceError> {
    generate_with_mode(
        expolygon,
        spacing,
        layer_height,
        minimum_nozzle_diameter,
        scale,
        true,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "the two source concentric variants share explicit geometry and mode inputs"
)]
fn generate_with_mode(
    expolygon: ExPolygon,
    spacing: i64,
    layer_height: i64,
    minimum_nozzle_diameter: f64,
    scale: CoordinateScale,
    standard: bool,
) -> Result<Vec<ThickPolyline>, SliceError> {
    if let Ok(path) = std::env::var("ARES_DUMP_CI2") {
        use std::io::Write;
        if let Ok(mut out) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let (contour, holes) = (&expolygon.contour(), expolygon.holes());
            let _ = write!(out, "ADOM n={}", contour.points().len());
            for point in contour.points() {
                let _ = write!(out, " {},{}", point.x(), point.y());
            }
            let _ = writeln!(out, " holes={}", holes.len());
        }
    }
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
    let polygons = if standard {
        let source = ExPolygon::new(contour, holes);
        offset_expolygon(&source, (0.5 * spacing as f64) as f32, JoinType::Miter, 3.0)
            .map_err(geometry_error)?
            .into_iter()
            .flat_map(|expolygon| {
                let (contour, holes) = expolygon.into_parts();
                std::iter::once(contour).chain(holes)
            })
            .collect()
    } else {
        let mut polygons = Vec::with_capacity(1 + holes.len());
        polygons.push(contour);
        polygons.extend(holes);
        polygons
    };
    let config = RawWallToolPathConfig {
        outer_spacing: spacing,
        inner_spacing: spacing,
        inset_count: loops_count,
        outer_wall_inset: 0,
        layer_height,
        min_bead_width: scale.checked_scale(0.85 * minimum_nozzle_diameter).unwrap(),
        min_feature_size: scale.checked_scale(0.25 * minimum_nozzle_diameter).unwrap(),
        transition_length: scale
            .checked_scale(if standard {
                minimum_nozzle_diameter
            } else {
                0.4
            })
            .unwrap(),
        transitioning_angle: f64::from((std::f32::consts::PI * 10.0_f32) / 180.0_f32),
        transition_filter_deviation: scale.checked_scale(0.25 * minimum_nozzle_diameter).unwrap(),
        wall_distribution_count: 1,
        // FillConcentric.cpp leaves this POD field unset. Repeated standalone
        // Linux 2.4.2 artifacts retain the short standard-fill branch at the
        // common wall-path factor; ConcentricInternal retains all odd branches.
        min_length_factor: if standard { 0.5 } else { 0.0 },
        wall_maximum_resolution: scale.checked_scale(0.5).unwrap(),
        wall_maximum_deviation: scale.checked_scale(0.025).unwrap(),
        is_top_or_bottom_layer: false,
        coordinate_scale: scale,
    };
    let generated = generate(&polygons, config).map_err(|error| {
        SliceError::InvalidInput(format!("concentric Arachne generation failed: {error:?}"))
    })?;
    if let Ok(path) = std::env::var("ARES_DUMP_CI") {
        use std::io::Write;
        if let Ok(mut out) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            for group in &generated.toolpaths {
                for line in group {
                    if line.junctions.len() < 2 {
                        continue;
                    }
                    let widths = line
                        .junctions
                        .iter()
                        .map(|junction| junction.width.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    let first = &line.junctions[0].point;
                    let idxs = line
                        .junctions
                        .iter()
                        .map(|junction| junction.perimeter_index.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    let _ = writeln!(
                        out,
                        "ARESCI closed={} odd={} n={} w=[{}] idx=[{}] p0={},{}",
                        line.is_closed,
                        line.is_odd,
                        line.junctions.len(),
                        widths,
                        idxs,
                        first.x(),
                        first.y()
                    );
                }
            }
        }
    }
    Ok(generated
        .toolpaths
        .into_iter()
        .flatten()
        .filter(|line| line.junctions.len() >= 2)
        .map(|line| line.to_thick_polyline())
        .collect())
}
