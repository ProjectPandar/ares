//! Single-wall top-surface regeneration (`PerimeterGenerator.cpp:2751–2877`).

use crate::{
    SliceError,
    arachne::{ExtrusionLine, wall_toolpaths},
    geometry::{
        BoundingBox, ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, Line, Polygon,
        Polyline, clip_clipper_expolygons_with_subject_bbox, difference_ex, difference_ex_polygons,
        difference_ex_polygons_with_safety_offset, difference_open_polylines, intersection_ex,
        offset_expolygons, offset_open_paths, offset2_ex, union_ex, union_expolygons,
        union_polygons_paths,
    },
    project_slice::region_slices::RegionSurface,
};

use super::{
    config::{ArachneRecordConfig, ArachneWallParams},
    walls::{ArachneSpacings, GeneratedSurfaceWalls, MITER_LIMIT},
};

pub(in crate::project_slice) struct TopSurfaceInputs<'a> {
    pub(in crate::project_slice) upper_slices: &'a [ExPolygon],
    pub(in crate::project_slice) upper_same_region: Option<&'a [RegionSurface]>,
    pub(in crate::project_slice) lower_slices: Option<&'a [ExPolygon]>,
    pub(in crate::project_slice) interface_shells: bool,
    pub(in crate::project_slice) min_width_top_surface_mm: f64,
    pub(in crate::project_slice) perimeter_width: i64,
}

pub(in crate::project_slice) struct Regeneration<'a> {
    pub(in crate::project_slice) spacings: &'a ArachneSpacings,
    pub(in crate::project_slice) last: &'a [ExPolygon],
    pub(in crate::project_slice) wall_0_inset: i64,
    pub(in crate::project_slice) layer_height: f64,
    pub(in crate::project_slice) params: ArachneWallParams,
    pub(in crate::project_slice) scale: CoordinateScale,
    pub(in crate::project_slice) inner_loop_number: i32,
}

pub(in crate::project_slice) fn regenerate(
    mut toolpaths: Vec<Vec<ExtrusionLine>>,
    mut infill_contour: Vec<ExPolygon>,
    ts: &TopSurfaceInputs<'_>,
    config: &ArachneRecordConfig,
    regeneration: Regeneration<'_>,
) -> Result<GeneratedSurfaceWalls, SliceError> {
    let r = regeneration;
    let spacing = r.spacings.ext_perimeter_spacing;
    let tolerance = spacing / 10;
    let mut full = None;
    // A one-inset solve may widen its two outer beads enough to displace an
    // inner bead where the object continues upward (:2811–2832).
    if toolpaths
        .iter()
        .flatten()
        .flat_map(|line| &line.junctions)
        .any(|j| j.width > spacing + 2 * tolerance)
    {
        let nominal = offset(r.last, -(spacing + r.wall_0_inset) as f32)?;
        let (top, uncovered) = top_area(&nominal, ts, &r)?;
        if uncovered {
            let top_zone = offset(&top, spacing as f32)?;
            let reference = r.generate_full(config)?;
            if let Some(outer) = reference
                .toolpaths
                .first()
                .filter(|outer| !outer.is_empty())
                && length_off_reference(&toolpaths, outer, &top_zone, tolerance)
                    .map_err(geometry_error)?
                    > ts.perimeter_width as f64
            {
                infill_contour = difference_ex_polygons_with_safety_offset(
                    &nominal,
                    &walls_footprint(outer).map_err(geometry_error)?,
                )
                .map_err(geometry_error)?;
                toolpaths = vec![outer.clone()];
            }
            full = Some(reference);
        }
    }

    let (top_expolygons, uncovered) = top_area(&infill_contour, ts, &r)?;
    if !uncovered {
        let generated = match full {
            Some(generated) => generated,
            None => r.generate_full(config)?,
        };
        return Ok(GeneratedSurfaceWalls {
            toolpaths: generated.toolpaths,
            infill_contour: union_ex(&generated.inner_contour, FillRule::NonZero)
                .map_err(geometry_error)?,
            top_expolygons,
        });
    }

    let not_top = difference_ex(&infill_contour, &top_expolygons).map_err(geometry_error)?;
    let not_top = offset(&not_top, r.wall_0_inset as f32)?;
    let mut inner = r.generate(
        &not_top,
        config,
        (r.spacings.perimeter_spacing, r.inner_loop_number + 1, 0),
    )?;
    if !toolpaths.is_empty() {
        for line in inner.toolpaths.iter_mut().flatten() {
            line.inset_index += 1;
        }
    }
    toolpaths.extend(inner.toolpaths);
    let infill_contour = union_expolygons(
        &top_expolygons,
        &union_ex(&inner.inner_contour, FillRule::NonZero).map_err(geometry_error)?,
    )
    .map_err(geometry_error)?;
    Ok(GeneratedSurfaceWalls {
        toolpaths,
        infill_contour,
        top_expolygons,
    })
}

impl Regeneration<'_> {
    fn generate_full(
        &self,
        config: &ArachneRecordConfig,
    ) -> Result<wall_toolpaths::GeneratedWallToolPaths, SliceError> {
        self.generate(
            self.last,
            config,
            (
                self.spacings.ext_perimeter_spacing,
                self.inner_loop_number + 2,
                self.wall_0_inset,
            ),
        )
    }

    fn generate(
        &self,
        outline: &[ExPolygon],
        config: &ArachneRecordConfig,
        (outer_spacing, count, outer_wall_inset): (i64, i32, i64),
    ) -> Result<wall_toolpaths::GeneratedWallToolPaths, SliceError> {
        let p = self.params;
        wall_toolpaths::generate(
            &to_polygons(outline),
            wall_toolpaths::RawWallToolPathConfig {
                outer_spacing,
                inner_spacing: self.spacings.perimeter_spacing,
                inset_count: usize::try_from(count)
                    .map_err(|_| SliceError::InvalidInput("wall_loops underflow".to_owned()))?,
                outer_wall_inset,
                layer_height: self.layer_height,
                min_bead_width: p.min_bead_width,
                min_feature_size: p.min_feature_size,
                transition_length: p.wall_transition_length,
                transitioning_angle: f64::from(
                    std::f32::consts::PI * p.wall_transition_angle as f32 / 180.0_f32,
                ),
                transition_filter_deviation: p.wall_transition_filter_deviation,
                wall_distribution_count: p.wall_distribution_count,
                min_length_factor: p.min_length_factor,
                wall_maximum_resolution: p.wall_maximum_resolution,
                wall_maximum_deviation: p.wall_maximum_deviation,
                is_top_or_bottom_layer: config.is_bottom_layer || config.is_topmost_layer,
                coordinate_scale: self.scale,
            },
        )
        .map_err(|error| {
            SliceError::InvalidInput(format!("Arachne wall generation failed: {error:?}"))
        })
    }
}

// `get_top_expolygons` (:2762–2795); exposure is recorded before excluding
// unsupported or too narrow regions.
fn top_area(
    contour: &[ExPolygon],
    ts: &TopSurfaceInputs<'_>,
    r: &Regeneration<'_>,
) -> Result<(Vec<ExPolygon>, bool), SliceError> {
    let Some(mut bounds) = BoundingBox::from_expolygons(contour) else {
        return Ok((Vec::new(), false));
    };
    bounds.offset(r.scale.scaled_delta(1e-4).round() as i64);
    let upper_clipped = if ts.interface_shells
        && let Some(surfaces) = ts.upper_same_region
    {
        clip_clipper_expolygons_with_subject_bbox(
            &surfaces
                .iter()
                .map(|surface| surface.as_parts().1.clone())
                .collect::<Vec<_>>(),
            bounds,
        )
    } else {
        clip_clipper_expolygons_with_subject_bbox(ts.upper_slices, bounds)
    };
    let mut top = difference_ex_polygons(contour, &upper_clipped).map_err(geometry_error)?;
    if top.is_empty() {
        return Ok((top, false));
    }
    if let Some(lower) = ts.lower_slices {
        let lower_clipped = clip_clipper_expolygons_with_subject_bbox(lower, bounds);
        let unsupported = difference_ex_polygons(&top, &lower_clipped).map_err(geometry_error)?;
        let bridges = offset(
            &unsupported,
            r.spacings.ext_perimeter_spacing.max(ts.perimeter_width) as f32,
        )?;
        top = difference_ex(&top, &bridges).map_err(geometry_error)?;
    }
    let min_width = f32::max(
        r.spacings.ext_perimeter_spacing as f32 / 4.0 + r.scale.scaled_delta(0.00001) as f32,
        r.scale.scaled_delta(ts.min_width_top_surface_mm) as f32 / 4.0,
    );
    top = offset2_ex(
        &top,
        -min_width,
        min_width + (ts.perimeter_width as f64 * 0.85) as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(geometry_error)?;
    Ok((
        intersection_ex(&top, contour).map_err(geometry_error)?,
        true,
    ))
}

// `length_off_reference` (:779–803). ClipperUtils offsets polylines with
// square joins and butt ends by default.
fn length_off_reference(
    walls: &[Vec<ExtrusionLine>],
    reference: &[ExtrusionLine],
    excluded: &[ExPolygon],
    tolerance: i64,
) -> Result<f64, ClipperError> {
    let reference = centerlines(reference).map(Polygon::new).collect::<Vec<_>>();
    let subject = centerlines(walls.iter().flatten())
        .map(Polyline::new)
        .collect::<Vec<_>>();
    let envelope = offset_open_paths(&reference, tolerance as f32, JoinType::Square, 0.0)?;
    let mut off = difference_open_polylines(&subject, &envelope)?;
    if !excluded.is_empty() {
        off = difference_open_polylines(&off, &to_polygons(excluded))?;
    }
    Ok(off
        .iter()
        .map(|line| {
            line.points()
                .windows(2)
                .map(|p| Line::new(p[0], p[1]).length())
                .sum::<f64>()
        })
        .sum())
}

fn centerlines<'a>(
    lines: impl IntoIterator<Item = &'a ExtrusionLine>,
) -> impl Iterator<Item = Vec<crate::geometry::Point>> {
    lines
        .into_iter()
        .filter(|line| line.junctions.len() >= 2)
        .map(|line| line.junctions.iter().map(|j| j.point).collect())
}

// `walls_footprint` (:805–818) uses each segment's greater endpoint width.
fn walls_footprint(walls: &[ExtrusionLine]) -> Result<Vec<Polygon>, ClipperError> {
    let mut footprints = Vec::new();
    for pair in walls.iter().flat_map(|line| line.junctions.windows(2)) {
        let width = pair[0].width.max(pair[1].width);
        if width > 0 {
            let segment = Polygon::new(vec![pair[0].point, pair[1].point]);
            footprints.extend(offset_open_paths(
                &[segment],
                width as f32 / 2.0,
                JoinType::Square,
                0.0,
            )?);
        }
    }
    union_polygons_paths(&footprints)
}

fn to_polygons(expolygons: &[ExPolygon]) -> Vec<Polygon> {
    expolygons
        .iter()
        .flat_map(|p| std::iter::once(p.contour()).chain(p.holes()))
        .cloned()
        .collect()
}

fn offset(polygons: &[ExPolygon], delta: f32) -> Result<Vec<ExPolygon>, SliceError> {
    offset_expolygons(polygons, delta, JoinType::Miter, MITER_LIMIT).map_err(geometry_error)
}

fn geometry_error(error: ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("Arachne top-surface geometry failed: {error:?}"))
}
