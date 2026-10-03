//! Geometry substeps of `SupportMaterial.cpp::detect_overhangs`.
//! The owning generator retains each layer's sharp tails and cantilevers for
//! the subsequent serial propagation and cluster filtering.

use super::layer_input::{LayerAnnotations, LayerInput, RegionInput, to_polygons};
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polygon, difference_ex,
    difference_ex_polygons, difference_polygons_paths, intersection_ex, intersection_pl,
    intersection_polygons_paths, offset_expolygon, offset_expolygons, offset_paths,
    point_in_polygon, union_ex,
};
use crate::{FloatOrPercent, ObjectOptions, ProcessSupportType};

// These upstream globals are initialized before a printer can switch
// SCALING_FACTOR from SCALING_FACTOR_INTERNAL to its large-bed value.
pub(super) const LENGTH_THRESH_WELL_SUPPORTED: f64 = 6_000_000.0;
const AREA_THRESH_WELL_SUPPORTED: f64 = LENGTH_THRESH_WELL_SUPPORTED * LENGTH_THRESH_WELL_SUPPORTED;

#[derive(Default)]
pub(super) struct SharpTails {
    pub(super) polygons: Vec<ExPolygon>,
    pub(super) heights: Vec<f32>,
}

pub(super) struct OverhangDetection<'a> {
    pub(super) layer_index: usize,
    pub(super) layer: &'a LayerInput<'a>,
    pub(super) lower: Option<&'a LayerInput<'a>>,
    pub(super) object: &'a ObjectOptions,
    pub(super) annotations: &'a LayerAnnotations<'a>,
    pub(super) scale: CoordinateScale,
}

pub(super) struct RegionOverhangs {
    pub(super) polygons: Vec<Polygon>,
    pub(super) lower_layer_offset: f32,
}

pub(super) struct DetectedOverhangs {
    pub(super) polygons: Vec<ExPolygon>,
    pub(super) cantilevers: Vec<ExPolygon>,
}

pub(super) fn detect_overhangs(
    input: &OverhangDetection<'_>,
    sharp_tails: &mut SharpTails,
) -> Result<DetectedOverhangs, ClipperError> {
    let mut lower_offset = 0.0;
    let mut polygons = Vec::new();
    if input.layer_index == 0 {
        polygons = first_layer_overhangs(input.layer, sharp_tails);
    } else if !input.layer.regions.is_empty() {
        let lower = input.lower.unwrap();
        let filtered =
            filtered_lower_slices(lower.slices, input.layer.regions[0].external_width_scaled)?;
        for region in input.layer.regions {
            let detected = detect_region_overhangs(input, region, &filtered, sharp_tails)?;
            lower_offset = detected.lower_layer_offset;
            let mut region_polygons = detected.polygons;
            if region_polygons.is_empty() {
                continue;
            }
            if !input.annotations.blockers.is_empty() {
                region_polygons =
                    difference_polygons_paths(&region_polygons, input.annotations.blockers)?;
                sharp_tails.polygons =
                    difference_ex_polygons(&sharp_tails.polygons, input.annotations.blockers)?;
            }
            if input.object.bridge_no_support.0 {
                super::remove_bridges::remove_bridges_from_contacts(
                    &region.bridges,
                    lower.slices,
                    region.external_width_scaled,
                    &mut region_polygons,
                    input.scale,
                )?;
            }
            if region_polygons.is_empty()
                || offset_paths(
                    &region_polygons,
                    (-0.1 * f64::from(region.external_width_scaled)) as f32,
                    JoinType::Miter,
                    3.0,
                )?
                .is_empty()
            {
                continue;
            }
            let expansion = input.object.support_expansion.0 / input.scale.factor();
            if expansion != 0.0 {
                region_polygons =
                    offset_paths(&region_polygons, expansion as f32, JoinType::Square, 0.0)?;
            }
            polygons.extend(region_polygons);
        }
    }
    let polygons = union_ex(&polygons, crate::geometry::FillRule::NonZero)?;
    let cantilevers = if let Some(lower) = input.lower {
        classify_cantilevers(
            &polygons,
            lower.slices,
            input.layer.regions[0].external_width_scaled,
            lower_offset,
            input.scale,
        )?
    } else {
        Vec::new()
    };
    Ok(DetectedOverhangs {
        polygons,
        cantilevers,
    })
}

pub(super) fn first_layer_overhangs(
    layer: &LayerInput<'_>,
    sharp_tails: &mut SharpTails,
) -> Vec<Polygon> {
    for slice in layer.slices {
        let bounds = BoundingBox::from_expolygon(slice).unwrap();
        let width = (bounds.max().x() - bounds.min().x()) as f64;
        let height = (bounds.max().y() - bounds.min().y()) as f64;
        if !(width > LENGTH_THRESH_WELL_SUPPORTED && height > LENGTH_THRESH_WELL_SUPPORTED) {
            sharp_tails.polygons.push(slice.clone());
            sharp_tails.heights.push(layer.height as f32);
        }
    }
    to_polygons(layer.slices)
}

pub(super) fn filtered_lower_slices(
    lower: &[ExPolygon],
    first_region_width: f32,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut output = Vec::new();
    for polygon in lower {
        if !offset_expolygon(polygon, -first_region_width / 2.0, JoinType::Miter, 3.0)?.is_empty() {
            output.push(polygon.clone());
        }
    }
    Ok(output)
}

pub(super) fn detect_region_overhangs(
    input: &OverhangDetection<'_>,
    region: &RegionInput<'_>,
    lower_filtered: &[ExPolygon],
    sharp_tails: &mut SharpTails,
) -> Result<RegionOverhangs, ClipperError> {
    let lower_layer = input.lower.unwrap();
    let threshold_angle = if input.object.support_threshold_angle.0 > 0 {
        f64::from(input.object.support_threshold_angle.0) + 1.0
    } else {
        0.0
    }
    .min(89.0);
    let threshold_angle = std::f64::consts::PI * threshold_angle / 180.0;
    let width = region.external_width_scaled;
    let lower_layer_offset = if input.layer_index < input.object.enforce_support_layers.0 as usize {
        0.0
    } else if threshold_angle > 0.0 {
        (lower_layer.height / threshold_angle.tan() / input.scale.factor()) as f32
    } else {
        let width_mm = f64::from(width) * input.scale.factor();
        let overlap_mm = match input.object.support_threshold_overlap {
            FloatOrPercent::Float(value) => value,
            FloatOrPercent::Percent(value) => width_mm * value.0 / 100.0,
        };
        width - (overlap_mm / input.scale.factor()) as f32
    };
    let lower = to_polygons(lower_layer.slices);
    let current = to_polygons(region.slices);
    let mut polygons = Vec::new();
    if lower_layer_offset == 0.0 {
        polygons = difference_polygons_paths(&current, &lower)?;
        if let Some(covered) = input.annotations.buildplate_covered {
            polygons = difference_polygons_paths(&polygons, covered)?;
        }
    } else if input.object.support_type == ProcessSupportType::NormalAuto {
        let expanded_lower = offset_paths(&lower, lower_layer_offset, JoinType::Square, 0.0)?;
        polygons = difference_polygons_paths(&current, &expanded_lower)?;
        if let Some(covered) = input.annotations.buildplate_covered
            && !covered.is_empty()
        {
            polygons = difference_polygons_paths(&polygons, covered)?;
        }
        if !polygons.is_empty() {
            let expanded = offset_paths(&polygons, lower_layer_offset, JoinType::Square, 0.0)?;
            let clipped = intersection_polygons_paths(&expanded, &current)?;
            polygons = difference_polygons_paths(&clipped, &lower)?;
        }
        for polygon in region.raw_slices {
            if offset_expolygon(polygon, -0.5 * width, JoinType::Miter, 3.0)?.is_empty() {
                continue;
            }
            let expanded = offset_expolygon(polygon, 0.5 * width, JoinType::Miter, 3.0)?;
            if overlaps(&expanded, lower_filtered)?
                || polygon.area() >= AREA_THRESH_WELL_SUPPORTED
                || offset_expolygon(
                    polygon,
                    (-0.1 * f64::from(width)) as f32,
                    JoinType::Miter,
                    3.0,
                )?
                .is_empty()
            {
                continue;
            }
            let overhang = difference_ex(std::slice::from_ref(polygon), lower_filtered)?;
            sharp_tails.polygons.push(polygon.clone());
            sharp_tails.heights.push(input.layer.height as f32);
            // The literal is a double in the source; conversion to float
            // occurs at the offset_ex argument, not before multiplication.
            let expanded = offset_expolygons(
                &overhang,
                (0.05 * f64::from(width)) as f32,
                JoinType::Miter,
                3.0,
            )?;
            polygons.extend(to_polygons(&expanded));
        }
    }
    Ok(RegionOverhangs {
        polygons,
        lower_layer_offset,
    })
}

pub(super) fn classify_cantilevers(
    overhangs: &[ExPolygon],
    lower_slices: &[ExPolygon],
    first_region_width: f32,
    last_lower_layer_offset: f32,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let expanded_lower = offset_expolygons(
        lower_slices,
        (f64::from(first_region_width.max(last_lower_layer_offset)) + 0.1 / scale.factor()) as f32,
        JoinType::Miter,
        3.0,
    )?;
    let mut cantilevers = Vec::new();
    for polygon in overhangs {
        let boundary = intersection_ex(std::slice::from_ref(polygon), &expanded_lower)?;
        let boundary = to_polygons(&boundary);
        if boundary.is_empty() {
            continue;
        }
        let mut maximum = 0.0_f64;
        for point in polygon.contour().points() {
            // Native MultiPoint::distance_to measures nearest vertex,
            // not nearest segment or the rounded point projection.
            let mut minimum = f64::MAX;
            for boundary in &boundary {
                let nearest = nearest_vertex_distance(boundary, *point);
                minimum = minimum.min(nearest);
            }
            maximum = maximum.max(minimum);
        }
        if maximum > 3.0 / scale.factor() {
            cantilevers.push(polygon.clone());
        }
    }
    Ok(cantilevers)
}

fn nearest_vertex_distance(polygon: &Polygon, point: Point) -> f64 {
    polygon
        .points()
        .iter()
        .map(|candidate| {
            let x = (candidate.x() - point.x()) as f64;
            let y = (candidate.y() - point.y()) as f64;
            (x * x + y * y).sqrt()
        })
        .fold(f64::MAX, f64::min)
}

/// `ExPolygon.cpp::overlaps`: contour intersection plus containment.
/// An area intersection is not equivalent for touching boundaries.
pub(super) fn overlaps(left: &[ExPolygon], right: &[ExPolygon]) -> Result<bool, ClipperError> {
    for left in left {
        for right in right {
            if left.contour().points().is_empty() || right.contour().points().is_empty() {
                continue;
            }
            let edges = intersection_pl(
                &to_polygons(std::slice::from_ref(right)),
                &to_polygons(std::slice::from_ref(left)),
            )?;
            if !edges.is_empty() || contains(right, left.contour().points()[0]) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub(super) fn contains(polygon: &ExPolygon, point: Point) -> bool {
    point_in_polygon(point, polygon.contour().points()) != 0
        && !polygon
            .holes()
            .iter()
            .any(|hole| point_in_polygon(point, hole.points()) == 1)
}
