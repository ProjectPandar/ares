use crate::{
    ProcessInfillPattern,
    geometry::{
        BoundingBox, ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, Line,
        LineDistanceTree, Point, Polygon, clip_clipper_expolygons_with_subject_bbox, difference_ex,
        intersection_ex, intersection_polygons_ex, intersection_polygons_paths, offset_expolygon,
        offset_expolygons, offset_paths, opening_paths, union_ex, union_safety_offset_ex,
    },
};

use super::{filter, trace};
use crate::project_slice::group_fills::{SurfaceFill, SurfaceFillPattern};

const MITER_LIMIT: f64 = 3.0;

pub(super) fn split(
    layer_id: usize,
    fill: &SurfaceFill,
    scale: CoordinateScale,
) -> Result<(Vec<ExPolygon>, Vec<ExPolygon>), ClipperError> {
    if line_based(fill.params.pattern) {
        split_lines(layer_id, fill, scale)
    } else {
        split_non_line(fill, scale)
    }
}

fn line_based(pattern: SurfaceFillPattern) -> bool {
    match pattern {
        SurfaceFillPattern::Configured(
            ProcessInfillPattern::Rectilinear
            | ProcessInfillPattern::Monotonic
            | ProcessInfillPattern::MonotonicLine
            | ProcessInfillPattern::AlignedRectilinear,
        ) => true,
        SurfaceFillPattern::Configured(
            ProcessInfillPattern::ZigZag
            | ProcessInfillPattern::CrossZag
            | ProcessInfillPattern::LockedZag
            | ProcessInfillPattern::Line
            | ProcessInfillPattern::Grid
            | ProcessInfillPattern::Triangles
            | ProcessInfillPattern::TriHexagon
            | ProcessInfillPattern::Cubic
            | ProcessInfillPattern::AdaptiveCubic
            | ProcessInfillPattern::QuarterCubic
            | ProcessInfillPattern::SupportCubic
            | ProcessInfillPattern::Lightning
            | ProcessInfillPattern::Honeycomb
            | ProcessInfillPattern::ThreeDHoneycomb
            | ProcessInfillPattern::LateralHoneycomb
            | ProcessInfillPattern::LateralLattice
            | ProcessInfillPattern::CrossHatch
            | ProcessInfillPattern::TpmsD
            | ProcessInfillPattern::TpmsFk
            | ProcessInfillPattern::Gyroid
            | ProcessInfillPattern::Concentric
            | ProcessInfillPattern::HilbertCurve
            | ProcessInfillPattern::ArchimedeanChords
            | ProcessInfillPattern::OctagramSpiral,
        )
        | SurfaceFillPattern::ConcentricInternal => false,
    }
}

fn split_non_line(
    fill: &SurfaceFill,
    scale: CoordinateScale,
) -> Result<(Vec<ExPolygon>, Vec<ExPolygon>), ClipperError> {
    let spacing = scaled(fill.params.spacing, scale);
    let mut normal = Vec::new();
    let mut narrow = Vec::new();
    for expolygon in &fill.expolygons {
        let filled = flatten(std::slice::from_ref(expolygon));
        let opened = opening_paths(
            &filled,
            spacing as f32,
            spacing as f32,
            JoinType::Miter,
            MITER_LIMIT,
        )?;
        let inner = intersection_polygons_paths(&filled, &opened)?;
        if inner.is_empty() {
            narrow.push(expolygon.clone());
            continue;
        }
        let inner = union_ex(&inner, FillRule::NonZero)?;
        let original = std::slice::from_ref(expolygon);
        narrow.extend(difference_ex(original, &inner)?);
        normal.extend(intersection_ex(original, &inner)?);
    }
    Ok((normal, narrow))
}

#[expect(
    clippy::excessive_nesting,
    reason = "the source line split pairs ordered wall intersections per scanline"
)]
fn split_lines(
    layer_id: usize,
    fill: &SurfaceFill,
    scale: CoordinateScale,
) -> Result<(Vec<ExPolygon>, Vec<ExPolygon>), ClipperError> {
    let spacing = scaled(fill.params.spacing, scale);
    let mut base_angle = f64::from(fill.params.angle + std::f32::consts::FRAC_PI_2);
    if fill.params.pattern
        != SurfaceFillPattern::Configured(ProcessInfillPattern::AlignedRectilinear)
        && (layer_id / usize::from(fill.representative.thickness_layers)) & 1 != 0
    {
        base_angle += f64::from(std::f32::consts::FRAC_PI_2);
    }
    let aligning_angle = std::f64::consts::PI - base_angle;
    let mut reconstructed = Vec::new();
    for expolygon in &fill.expolygons {
        let rotated = rotate_polygons(&flatten(std::slice::from_ref(expolygon)), aligning_angle);
        let bounds = BoundingBox::from_polygons(&rotated)
            .expect("trusted grouped ExPolygon contains a nonempty contour");
        let opened = opening_paths(
            &rotated,
            (2 * spacing) as f32,
            (3 * spacing) as f32,
            JoinType::Miter,
            MITER_LIMIT,
        )?;
        let mut inner = intersection_polygons_paths(&rotated, &opened)?;
        let shrink = spacing as f64 * 0.5 - fill.params.overlap / scale.factor();
        inner = offset_paths(&inner, -(shrink as f32), JoinType::Miter, MITER_LIMIT)?;
        let walls = inner.iter().flat_map(Polygon::lines).collect::<Vec<_>>();
        let tree = LineDistanceTree::new(&walls);
        let width = bounds.max().x() - bounds.min().x();
        let count = (width + spacing - 1) / spacing;
        let mut sections = Vec::with_capacity(count as usize);
        for index in 0..count {
            let x = (bounds.min().x() as f64 + index as f64 * spacing as f64) as i64;
            let vertical = Line::new(
                Point::new(x, bounds.min().y()),
                Point::new(x, bounds.max().y()),
            );
            let intersections = tree.intersections_sorted(vertical);
            let mut section = Vec::new();
            for pair in intersections.windows(2) {
                let first = pair[0].0;
                let second = pair[1].0;
                let midpoint = Point::new(
                    midpoint(first.x(), second.x()),
                    midpoint(first.y(), second.y()),
                );
                if tree.outside(midpoint) < 0 && (first.y() - second.y()).abs() > spacing {
                    section.push(Line::new(first, second));
                }
            }
            sections.push(section);
        }
        let maximum_short = scaled(4.0, scale);
        let sections = filter::apply(sections, maximum_short);
        reconstructed.extend(trace::reconstruct(&sections, spacing));
    }
    reconstructed = rotate_polygons(&reconstructed, -aligning_angle);
    if let Ok(path) = std::env::var("ARES_DUMP_SPLIT") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "RECON n={}", reconstructed.len());
        }
    }

    let mut normal = union_safety_offset_ex(&reconstructed)?;
    let mut narrow = difference_ex(&fill.expolygons, &normal)?;
    if let Ok(path) = std::env::var("ARES_DUMP_SPLIT") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(
                file,
                "SPLITOUT normal={} narrow={}",
                normal.len(),
                narrow.len()
            );
        }
    }
    let mut index = 0;
    while index < narrow.len() {
        let shrunk = offset_expolygon(
            &narrow[index],
            -(spacing as f64 * 0.5) as f32,
            JoinType::Miter,
            MITER_LIMIT,
        )?;
        if shrunk.is_empty() {
            let expanded = offset_expolygon(
                &narrow[index],
                (spacing as f64 * 0.3) as f32,
                JoinType::Miter,
                MITER_LIMIT,
            )?;
            let Some(bounds) = BoundingBox::from_expolygons(&expanded) else {
                index += 1;
                continue;
            };
            let clipped = clip_clipper_expolygons_with_subject_bbox(&normal, bounds);
            if !intersection_polygons_ex(&clipped, &expanded)?.is_empty() {
                normal.push(narrow.remove(index));
                continue;
            }
        }
        index += 1;
    }
    if narrow.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let expanded_normal = offset_expolygons(
        &normal,
        (spacing as f64 * 0.5) as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )?;
    let normal = intersection_ex(&expanded_normal, &fill.expolygons)?;
    Ok((normal, narrow))
}

fn scaled(value: f64, scale: CoordinateScale) -> i64 {
    (value / scale.factor()) as i64
}

fn midpoint(left: i64, right: i64) -> i64 {
    ((i128::from(left) + i128::from(right)) / 2) as i64
}

fn flatten(expolygons: &[ExPolygon]) -> Vec<Polygon> {
    let mut polygons = Vec::new();
    for expolygon in expolygons {
        polygons.push(expolygon.contour().clone());
        polygons.extend(expolygon.holes().iter().cloned());
    }
    polygons
}

fn rotate_polygons(polygons: &[Polygon], angle: f64) -> Vec<Polygon> {
    let cosine = angle.cos();
    let sine = angle.sin();
    polygons
        .iter()
        .map(|polygon| {
            Polygon::new(
                polygon
                    .points()
                    .iter()
                    .map(|point| {
                        let x = point.x() as f64;
                        let y = point.y() as f64;
                        Point::new(
                            (cosine * x - sine * y).round() as i64,
                            (cosine * y + sine * x).round() as i64,
                        )
                    })
                    .collect(),
            )
        })
        .collect()
}
