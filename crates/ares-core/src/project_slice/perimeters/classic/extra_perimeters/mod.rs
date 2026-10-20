//! `PerimeterGenerator.cpp:885-1084`, extra overhang perimeter geometry.

mod apply;
mod ordering;
pub(in crate::project_slice::perimeters) use apply::Context;
pub(super) use apply::context;

use crate::SliceError;
use crate::geometry::medial_axis::MedialAxisError;
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, Line,
    LineDistanceTree, Point, Polygon, Polyline, detect_bridging_direction,
    difference_open_polylines, difference_polygons_paths, intersection_open_polylines,
    intersection_polygons_paths, medial_axis, offset_paths, offset_paths_tree, union_ex,
    union_polygons_paths, union_polygons_paths_pair,
};
use crate::project_slice::perimeters::types::Flow;

pub(in crate::project_slice::perimeters) struct ExtraPerimeters {
    pub(in crate::project_slice::perimeters) paths: Vec<Polyline>,
    pub(super) has_regions: bool,
    pub(in crate::project_slice::perimeters) filled: Vec<Polygon>,
}

impl Context<'_> {
    pub(in crate::project_slice::perimeters) fn generate(
        &self,
        infill: &[ExPolygon],
    ) -> Result<ExtraPerimeters, SliceError> {
        generate(
            infill,
            self.lower,
            self.perimeter_count,
            self.flow,
            self.scale,
        )
        .map_err(|error| match error {
            Error::Geometry(error) => {
                SliceError::InvalidInput(format!("extra overhang perimeter geometry: {error:?}"))
            }
            Error::Medial(error) => {
                SliceError::InvalidInput(format!("extra overhang perimeter medial axis: {error:?}"))
            }
        })
    }
}

enum Error {
    Geometry(ClipperError),
    Medial(MedialAxisError),
}

impl From<ClipperError> for Error {
    fn from(error: ClipperError) -> Self {
        Self::Geometry(error)
    }
}

fn generate(
    infill: &[ExPolygon],
    lower: &[Polygon],
    perimeter_count: i32,
    flow: Flow,
    scale: CoordinateScale,
) -> Result<ExtraPerimeters, Error> {
    let mut result = ExtraPerimeters {
        paths: Vec::new(),
        filled: Vec::new(),
        has_regions: false,
    };
    let Some(bounds) = BoundingBox::from_expolygons(infill) else {
        return Ok(result);
    };
    let epsilon = scale.scaled_delta(1e-4) as i64;
    let bounds = BoundingBox::new(
        Point::new(bounds.min().x() - epsilon, bounds.min().y() - epsilon),
        Point::new(bounds.max().x() + epsilon, bounds.max().y() + epsilon),
    );
    let lower = crate::geometry::clip_clipper_polygons_with_subject_bbox(lower, bounds);
    let infill = polygons(infill);
    let overhangs = difference_polygons_paths(&infill, &lower)?;
    if overhangs.is_empty() {
        return Ok(result);
    }
    let spacing = flow.scaled_spacing(scale) as f64;
    let width = flow.scaled_width(scale) as f64;
    let anchor_size = (scale.scaled_delta(3.0) as i64)
        .min(flow.scaled_spacing(scale) * i64::from(perimeter_count + 1));
    let anchors = intersection_polygons_paths(&infill, &lower)?;
    let expanded = offset_paths(
        &overhangs,
        (anchor_size as f64 + 0.1 * width) as f32,
        JoinType::Square,
        0.0,
    )?;
    let inset_anchors = difference_polygons_paths(&anchors, &expanded)?;
    let inset = difference_polygons_paths(&infill, &inset_anchors)?;
    let lower_lines = lines(&lower);
    let lower_tree = LineDistanceTree::new(&lower_lines);
    let mut unfilled = Vec::new();
    for overhang in union_ex(&inset, FillRule::NonZero)? {
        let cover = polygons(&[overhang]);
        let expanded_cover = offset(&cover, 1.1 * spacing)?;
        let shrunken_cover = offset(&cover, -0.1 * spacing)?;
        let real = intersection_polygons_paths(&cover, &overhangs)?;
        if real.is_empty() {
            unfilled.extend(cover);
            continue;
        }
        result.has_regions = true;
        let anchoring = intersection_polygons_paths(&expanded_cover, &inset_anchors)?;
        let joined = union_polygons_paths_pair(&offset(&cover, 0.1 * spacing)?, &anchoring)?;
        let mut perimeter = offset(&joined, -0.6 * spacing)?;
        let anchor_points = anchoring
            .iter()
            .flat_map(Polygon::points)
            .copied()
            .collect::<Vec<_>>();
        let hull = Polygon::new(crate::geometry::convex_hull(&anchor_points));
        let unbridgeable = area(&difference_polygons_paths(&real, &[hull])?);
        let unsupported = unsupported_distance(&real, &anchors, scale)?;
        let real_length: f64 = lines(&real).into_iter().map(Line::length).sum();
        if unbridgeable < 0.2 * area(&real) && unsupported < real_length * 0.2 {
            unfilled.extend(cover);
            continue;
        }
        let mut paths = Vec::new();
        let mut continuation = 2;
        let mut changed = true;
        while continuation >= 0 && changed {
            let previous = perimeter;
            let clipped = intersection_open_polylines(&closed(&previous), &shrunken_cover)?;
            let joined = union_polygons_paths_pair(&previous, &anchoring)?;
            perimeter = intersection_polygons_paths(&offset(&joined, -spacing)?, &expanded_cover)?;
            changed = previous != perimeter;
            if !perimeter.is_empty() {
                paths.extend(ordering::reconnect(clipped, spacing));
                continuation -=
                    i32::from(intersection_polygons_paths(&perimeter, &real)?.is_empty());
                continue;
            }
            let shrunken =
                intersection_polygons_paths(&offset(&previous, -0.3 * spacing)?, &expanded_cover)?;
            if !shrunken.is_empty() {
                paths.extend(ordering::reconnect(clipped, spacing));
            }
            let gaps = if shrunken.is_empty() {
                offset_paths_tree(&previous, (0.5 * spacing) as f32, JoinType::Miter, 3.0)?
                    .into_expolygons()
            } else {
                // `to_expolygons` wraps each polygon without unioning or regrouping holes.
                shrunken
                    .into_iter()
                    .map(|polygon| ExPolygon::new(polygon, Vec::new()))
                    .collect()
            };
            let mut fills = Vec::new();
            for gap in gaps {
                let medial =
                    medial_axis(&gap, 0.75 * width, 3.0 * spacing, scale).map_err(Error::Medial)?;
                fills.extend(medial.into_iter().map(|path| Polyline::new(path.points)));
            }
            if !fills.is_empty() {
                let clipped = intersection_open_polylines(&fills, &shrunken_cover)?;
                paths.extend(ordering::reconnect(clipped, spacing));
            }
            break;
        }
        unfilled.extend(union_polygons_paths_pair(
            &offset(&perimeter, 0.5 * spacing)?,
            &anchoring,
        )?);
        paths.retain(|path| !path.points().is_empty());
        let Some(first) = paths.first() else { continue };
        let closed_and_anchored = first.front() == first.back()
            && !intersection_open_polylines(std::slice::from_ref(first), &lower)?.is_empty();
        if !closed_and_anchored {
            paths.reverse();
        } else {
            ordering::start_at_anchor(&mut paths[0], &lower_tree);
        }
        let (mut anchored, others): (Vec<_>, Vec<_>) = paths.into_iter().partition(|path| {
            signed_distance(&lower_tree, path.front().unwrap()) <= 0.0
                || signed_distance(&lower_tree, path.back().unwrap()) <= 0.0
        });
        let first_unanchored = anchored.len();
        anchored.extend(others);
        result
            .paths
            .extend(ordering::sort(anchored, first_unanchored, spacing));
    }
    result.filled = difference_polygons_paths(&inset, &union_polygons_paths(&unfilled)?)?;
    Ok(result)
}

fn offset(polygons: &[Polygon], delta: f64) -> Result<Vec<Polygon>, ClipperError> {
    offset_paths(polygons, delta as f32, JoinType::Miter, 3.0)
}

fn polygons(expolygons: &[ExPolygon]) -> Vec<Polygon> {
    expolygons
        .iter()
        .flat_map(|ex| std::iter::once(ex.contour()).chain(ex.holes()))
        .cloned()
        .collect()
}

fn lines(polygons: &[Polygon]) -> Vec<Line> {
    polygons.iter().flat_map(Polygon::lines).collect()
}

fn closed(polygons: &[Polygon]) -> Vec<Polyline> {
    polygons
        .iter()
        .map(|polygon| {
            let mut points = polygon.points().to_vec();
            points.push(points[0]);
            Polyline::new(points)
        })
        .collect()
}

fn area(polygons: &[Polygon]) -> f64 {
    polygons.iter().map(Polygon::area).sum()
}

fn signed_distance(tree: &LineDistanceTree<'_>, point: Point) -> f64 {
    tree.nearest(point).map_or(f64::MAX, |nearest| {
        nearest.squared_distance.sqrt() * f64::from(tree.outside(point))
    })
}

/// Authorized local adapter: `BridgeDetector.hpp:122-126`. Shared direction
/// selection remains owned by `geometry::detect_bridging_direction`.
fn unsupported_distance(
    cover: &[Polygon],
    anchors: &[Polygon],
    scale: CoordinateScale,
) -> Result<f64, ClipperError> {
    let overhang = difference_polygons_paths(cover, anchors)?;
    let floating = difference_open_polylines(
        &closed(&overhang),
        &offset(anchors, scale.scaled_delta(1e-4))?,
    )?;
    let edges = floating
        .iter()
        .flat_map(|p| p.points().windows(2).map(|p| Line::new(p[0], p[1])))
        .collect::<Vec<_>>();
    Ok(detect_bridging_direction(&edges, &overhang, scale).1)
}
