//! Source-owned Brim.cpp automatic ear area shared by both Rust adapters.
use crate::geometry::{
    CoordinateScale, ExPolygon, JoinType, Point, Polygon, difference_ex, intersection_ex,
    offset_expolygons,
};
use crate::{BrimOptions, BrimPath, LayerContours, Point2, SliceError};

mod auto;
pub(crate) mod paths;

pub(crate) struct AreaParameters {
    pub(crate) gap: f32,
    pub(crate) width: f32,
    pub(crate) spacing: f32,
    pub(crate) resolution: f64,
    pub(crate) detection_length: i64,
    pub(crate) max_angle: f64,
}

pub(crate) struct PaintedParameters {
    pub(crate) gap: f32,
    /// Config brim width already quantized to even brim flow widths and
    /// scaled (`Brim.cpp:445`); the `brim_width` of the inner branch
    /// (`:546`).
    pub(crate) width: f32,
    pub(crate) spacing: f32,
    pub(crate) resolution: f64,
    /// Brim flow width in millimetres (`print.brim_flow()` spacing), the
    /// `flowWidth` argument of `make_brim_ears` (Brim.cpp:355).
    pub(crate) flow_width: f64,
    pub(crate) scale: CoordinateScale,
    /// Ear centres already mapped into the `source` (mesh-centred
    /// lslices) space, each with its `head_front_radius` in millimetres.
    pub(crate) points: Vec<([f32; 2], f32)>,
    /// Raw bottom lslices (`raw_outline`, Brim.cpp:363) for island
    /// matching during the snap.
    pub(crate) raw_outline: Vec<ExPolygon>,
    /// `Some(efc_outline)` when the gate `:56-62` is open; the
    /// lazily-built EFC-adjusted outline is the caller's single
    /// computation (Brim.cpp:364-367/:112-118).
    pub(crate) efc_outline: Option<Vec<ExPolygon>>,
}

/// `make_brim_ears` (Brim.cpp:354-412), outer-birim call site `:537`:
/// per-point discs minus the object contour offset. The EFC snapping
/// branch (:377-404) is deferred with the EFC chain; the inner call site
/// (:551) needs a holed fixture and stays deferred.
/// `Brim.cpp:383-388`: the POLY_SIDE_COUNT-sided ear circle (24,
/// libslic3r.h:72) translated to the truncating-scaled ear centre.
fn ear_disc(center_x: i64, center_y: i64, size: i64) -> Polygon {
    Polygon::new(
        (0..24)
            .map(|index| {
                let angle = 2.0 * std::f64::consts::PI * f64::from(index) / 24.0;
                Point::new(
                    center_x + (size as f64 * angle.cos()).round() as i64,
                    center_y + (size as f64 * angle.sin()).round() as i64,
                )
            })
            .collect(),
    )
}

/// `closest_point_on_expolygons` (Brim.cpp:65-80): nearest vertex over
/// every contour and hole (vertex-level `closest_point`, no projection),
/// `int64` squared deltas compared as `double` with strict `<`.
fn closest_point_on_expolygons(polygons: &[ExPolygon], from: Point) -> Option<Point> {
    let mut best: Option<(f64, Point)> = None;
    let mut consider = |point: &Point| {
        let dx = point.x() - from.x();
        let dy = point.y() - from.y();
        let distance = (dx * dx + dy * dy) as f64;
        if best.is_none_or(|(previous, _)| distance < previous) {
            best = Some((distance, *point));
        }
    };
    for polygon in polygons {
        consider_many(polygon, &mut consider);
    }
    best.map(|(_, point)| point)
}

fn consider_many<F: FnMut(&Point)>(polygon: &ExPolygon, consider: &mut F) {
    for point in polygon.contour().points() {
        consider(point);
    }
    for hole in polygon.holes() {
        for point in hole.points() {
            consider(point);
        }
    }
}

/// `find_containing_expolygon_index` (Brim.cpp:84-96): first lslices
/// island containing the point; `ExPolygon::contains` excludes the
/// holes (ExPolygon.cpp), so a centre inside a hole matches no island.
fn find_containing_expolygon_index(polygons: &[ExPolygon], from: Point) -> Option<usize> {
    polygons.iter().position(|polygon| {
        polygon.contour().contains(&from)
            && !polygon.holes().iter().any(|hole| hole.contains(&from))
    })
}

/// `closest_point_on_matching_island` (Brim.cpp:99-108): snap inside the
/// matched island's EFC intersection, else on the whole EFC outline.
fn closest_point_on_matching_island(
    raw_outline: &[ExPolygon],
    efc_outline: &[ExPolygon],
    from: Point,
) -> Result<Option<Point>, SliceError> {
    if let Some(index) = find_containing_expolygon_index(raw_outline, from) {
        let island_outline =
            intersection_ex(efc_outline, &raw_outline[index..index + 1]).map_err(geometry_error)?;
        if !island_outline.is_empty() {
            return Ok(closest_point_on_expolygons(&island_outline, from));
        }
    }
    Ok(closest_point_on_expolygons(efc_outline, from))
}

pub(crate) fn painted(
    source: &[ExPolygon],
    parameters: PaintedParameters,
) -> Result<(Vec<ExPolygon>, Vec<ExPolygon>), SliceError> {
    let PaintedParameters {
        gap,
        width,
        spacing,
        resolution,
        flow_width,
        scale,
        points,
        raw_outline,
        efc_outline,
    } = parameters;
    let factor = scale.factor();
    // :392-411: snap each ear centre (after the truncating scale_ of
    // :390-391) to the closest vertex of the EFC-adjusted outline when
    // the gate :56-62 is open; the disc size path (:375-382) is
    // unaffected, so the snapped centre feeds both the outer and the
    // inner discs (:411 translates after the branch).
    let mut centers = Vec::with_capacity(points.len());
    for &(center, _) in &points {
        let mut pt = Point::new(
            (f64::from(center[0]) / factor) as i64,
            (f64::from(center[1]) / factor) as i64,
        );
        let snapped = match efc_outline.as_ref() {
            Some(efc) if !efc.is_empty() => {
                closest_point_on_matching_island(&raw_outline, efc, pt)?
            }
            _ => None,
        };
        if let Some(closest) = snapped {
            pt = closest;
        }
        centers.push(pt);
    }
    let mut result = Vec::new();
    for poly in source {
        let contour = ExPolygon::new(poly.contour().clone(), Vec::new());
        let inner = offset_expolygons(&[contour], gap, JoinType::Round, resolution)
            .map_err(geometry_error)?;
        let mut discs = Vec::new();
        for (&(_, radius), &pt) in points.iter().zip(&centers) {
            // :375/:377-380: the painted radius quantizes to even brim
            // flow widths (the second re-quantization pass over the scaled
            // width collapses to the same millimetre quantization).
            let width_mm = (f64::from(radius) / flow_width / 2.0).floor() * flow_width * 2.0;
            // :382: size_ear = brim_width - brim_offset - scaled spacing.
            let size_ear = (scale.checked_scale(width_mm).expect("validated brim width") as f32
                - gap
                - spacing) as i64;
            discs.push(ExPolygon::new(
                ear_disc(pt.x(), pt.y(), size_ear),
                Vec::new(),
            ));
        }
        // :544: append(brim_area_object, diff_ex(outerExpoly, innerExpoly)).
        result.extend(difference_ex(&discs, &inner).map_err(geometry_error)?);
        // :546-556: the inner call site (is_outer_brim=false).
        let mut holes = Vec::new();
        for hole in poly.holes() {
            let mut reversed = hole.clone();
            reversed.reverse();
            holes.push(ExPolygon::new(reversed, Vec::new()));
        }
        if !holes.is_empty() {
            // :547 offset_ex with no joinType falls back to
            // DefaultJoinType jtMiter / DefaultMiterLimit 3
            // (ClipperUtils.hpp:19/331); only the outer arm at :533 pins
            // jtRound/SCALED_RESOLUTION. The per-hole ExPolygon list is
            // equivalent to upstream's single Polygons union as long as
            // holes never overlap (first-layer lslices holes are
            // disjoint).
            let inset = offset_expolygons(&holes, -width - gap, JoinType::Miter, 3.0)
                .map_err(geometry_error)?;
            let mut inner_discs = Vec::new();
            for (&(_, radius), &pt) in points.iter().zip(&centers) {
                // :375-376 without the :377-380 outer re-quantization:
                // brim_width = floor(scale_(radius) / flowWidth / 2) *
                // flowWidth * 2 keeps the scaled/millimetre mix; scale_ is
                // the plain double division (libslic3r.h:93), so the
                // quotient stays f64 until the float assignment.
                let scaled_radius = f64::from(radius) / factor;
                let mixed = (scaled_radius / flow_width / 2.0).floor() * flow_width * 2.0;
                let size_ear = (mixed as f32 - gap - spacing) as i64;
                inner_discs.push(ExPolygon::new(
                    ear_disc(pt.x(), pt.y(), size_ear),
                    Vec::new(),
                ));
            }
            let ring = difference_ex(&inner_discs, &inset).map_err(geometry_error)?;
            // :553-554: intersection(diff(outer, inner), holes_rev).
            result.extend(intersection_ex(&ring, &holes).map_err(geometry_error)?);
        }
    }
    // :560-570: objectIsland footprint stays the forbidden source.
    let footprint =
        offset_expolygons(source, gap, JoinType::Round, resolution).map_err(geometry_error)?;
    Ok((result, footprint))
}

pub(crate) fn area(
    source: &[ExPolygon],
    parameters: AreaParameters,
) -> Result<(Vec<ExPolygon>, Vec<ExPolygon>), SliceError> {
    let AreaParameters {
        gap,
        width,
        spacing,
        resolution,
        detection_length,
        max_angle,
    } = parameters;
    let mut result = Vec::new();
    for poly in source {
        let contour = ExPolygon::new(poly.contour().clone(), Vec::new());
        let inner = offset_expolygons(&[contour], gap, JoinType::Round, resolution)
            .map_err(geometry_error)?;
        let radius = (width - gap - spacing) as i64;
        let outer = auto::make_auto(&inner, radius, detection_length, max_angle, true);
        result.extend(difference_ex(&outer, &inner).map_err(geometry_error)?);
        let holes: Vec<_> = poly
            .holes()
            .iter()
            .map(|hole| {
                let mut reversed = hole.clone();
                reversed.reverse();
                ExPolygon::new(reversed, Vec::new())
            })
            .collect();
        let inset = offset_expolygons(&holes, -width - gap, JoinType::Round, resolution)
            .map_err(geometry_error)?;
        let ear_source =
            offset_expolygons(&holes, -gap, JoinType::Round, resolution).map_err(geometry_error)?;
        let ears = auto::make_auto(&ear_source, radius, detection_length, max_angle, false);
        let inner_ears = difference_ex(&ears, &inset).map_err(geometry_error)?;
        result.extend(intersection_ex(&inner_ears, &holes).map_err(geometry_error)?);
    }
    let footprint =
        offset_expolygons(source, gap, JoinType::Round, resolution).map_err(geometry_error)?;
    Ok((result, footprint))
}

pub(super) fn brim_ear_paths(
    layer: &LayerContours,
    options: BrimOptions,
    effective_line_width: f64,
) -> Result<Vec<BrimPath>, SliceError> {
    let scale = CoordinateScale::Normal;
    let mut source = Vec::new();
    for contour in layer
        .contours()
        .iter()
        .filter(|c| layer.is_outer_contour(c))
    {
        let polygon = scaled_contour(contour, scale)?;
        let holes = layer
            .contours()
            .iter()
            .filter(|c| !layer.is_outer_contour(c))
            .map(|c| scaled_contour(c, scale))
            .collect::<Result<Vec<_>, _>>()?;
        let mut holes = holes
            .into_iter()
            .filter(|h| polygon.contains(&h.points()[0]))
            .collect::<Vec<_>>();
        for hole in &mut holes {
            if hole.is_counter_clockwise() {
                hole.reverse();
            }
        }
        let mut polygon = polygon;
        if !polygon.is_counter_clockwise() {
            polygon.reverse();
        }
        source.push(ExPolygon::new(polygon, holes));
    }
    let spacing = scaled(scale, effective_line_width)? as f32;
    let spacing_mm = f64::from(spacing) * scale.factor();
    let width = (options.width_mm() / spacing_mm / 2.0).floor() * spacing_mm * 2.0;
    let gap = scaled(scale, options.object_gap_mm())? as f32;
    let resolution = scale.scaled_delta(0.0125);
    let (area, footprint) = area(
        &source,
        AreaParameters {
            gap,
            width: scaled(scale, width)? as f32,
            spacing,
            resolution,
            detection_length: scaled(scale, options.brim_ears_detection_length_mm())?,
            max_angle: options.brim_ears_max_angle_degrees(),
        },
    )?;
    let area = difference_ex(&area, &footprint).map_err(geometry_error)?;
    paths::from_area(area, spacing, resolution)?
        .into_iter()
        .map(|points| {
            BrimPath::new(
                points
                    .into_iter()
                    .map(|p| Point2::new(scale.unscale(p.x()), scale.unscale(p.y())))
                    .collect(),
            )
        })
        .collect()
}

fn scaled_contour(contour: &crate::Contour, scale: CoordinateScale) -> Result<Polygon, SliceError> {
    Ok(Polygon::new(
        contour
            .points()
            .iter()
            .map(|p| Ok(Point::new(scaled(scale, p.x())?, scaled(scale, p.y())?)))
            .collect::<Result<Vec<_>, SliceError>>()?,
    ))
}
fn scaled(scale: CoordinateScale, value: f64) -> Result<i64, SliceError> {
    scale
        .checked_scale(value)
        .ok_or_else(|| SliceError::InvalidInput("brim coordinate is out of range".into()))
}
fn geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput("brim generation failed".into())
}
