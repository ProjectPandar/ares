use super::{RegionExpansion, RegionExpansionEx, RegionExpansionParameters, WaveSeed, wave_seeds};
use crate::geometry::clipper::{
    ClipperError, FillRule, JoinType, intersection_polygons_paths_with_fill, offset_lines,
    raw_offset_paths, union_ex,
};
use crate::geometry::clipper2::EndType;
use crate::geometry::{
    BoundingBox, CoordinateScale, ExPolygon, Polygon, clip_clipper_expolygons_with_subject_bbox,
};

pub(crate) fn propagate_waves_ex(
    seeds: &[WaveSeed],
    boundary: &[ExPolygon],
    params: &RegionExpansionParameters,
) -> Result<Vec<RegionExpansionEx>, ClipperError> {
    let expanded = propagate_waves(seeds, boundary, params)?;
    debug_assert!(seeds.is_sorted_by_key(|seed| (seed.boundary, seed.src)));

    let mut expanded = expanded.into_iter().peekable();
    let mut output = Vec::new();
    while let Some(first) = expanded.next() {
        let src_id = first.src_id;
        let boundary_id = first.boundary_id;
        let mut polygons = vec![first.polygon];
        while expanded
            .peek()
            .is_some_and(|next| next.boundary_id == boundary_id && next.src_id == src_id)
        {
            polygons.push(
                expanded
                    .next()
                    .expect("the matching record was peeked")
                    .polygon,
            );
        }
        if polygons.len() == 1 {
            output.push(RegionExpansionEx {
                expolygon: ExPolygon::new(polygons.pop().expect("singleton group"), Vec::new()),
                src_id,
                boundary_id,
            });
        } else {
            output.extend(
                union_ex(&polygons, FillRule::NonZero)?
                    .into_iter()
                    .map(|expolygon| RegionExpansionEx {
                        expolygon,
                        src_id,
                        boundary_id,
                    }),
            );
        }
    }
    Ok(output)
}

#[expect(
    clippy::too_many_arguments,
    reason = "the source scalar overload keeps the upstream argument order"
)]
pub(crate) fn propagate_waves_ex_from_sources_with_steps(
    src: &[ExPolygon],
    boundary: &[ExPolygon],
    full_expansion: f32,
    expansion_step: f32,
    max_nr_expansion_steps: usize,
    scale: CoordinateScale,
) -> Result<Vec<RegionExpansionEx>, ClipperError> {
    let params = RegionExpansionParameters::build(
        full_expansion,
        expansion_step,
        max_nr_expansion_steps,
        scale,
    );
    let seeds = wave_seeds(src, boundary, params.tiny_expansion, true, scale)?;
    propagate_waves_ex(&seeds, boundary, &params)
}

pub(crate) fn propagate_waves_from_sources(
    src: &[ExPolygon],
    boundary: &[ExPolygon],
    params: &RegionExpansionParameters,
    scale: CoordinateScale,
) -> Result<Vec<RegionExpansion>, ClipperError> {
    let seeds = wave_seeds(src, boundary, params.tiny_expansion, true, scale)?;
    propagate_waves(&seeds, boundary, params)
}

#[expect(
    clippy::too_many_arguments,
    reason = "the source scalar overload keeps the upstream argument order"
)]
pub(crate) fn propagate_waves_from_sources_with_steps(
    src: &[ExPolygon],
    boundary: &[ExPolygon],
    expansion: f32,
    expansion_step: f32,
    max_nr_steps: usize,
    scale: CoordinateScale,
) -> Result<Vec<RegionExpansion>, ClipperError> {
    let params = RegionExpansionParameters::build(expansion, expansion_step, max_nr_steps, scale);
    propagate_waves_from_sources(src, boundary, &params, scale)
}

#[expect(
    clippy::too_many_arguments,
    reason = "the source helper keeps the upstream argument order"
)]
pub(crate) fn expand_expolygons(
    src: &[ExPolygon],
    boundary: &[ExPolygon],
    expansion: f32,
    expansion_step: f32,
    max_nr_steps: usize,
    scale: CoordinateScale,
) -> Result<Vec<Vec<Polygon>>, ClipperError> {
    let mut output = vec![Vec::new(); src.len()];
    for expansion in propagate_waves_from_sources_with_steps(
        src,
        boundary,
        expansion,
        expansion_step,
        max_nr_steps,
        scale,
    )? {
        output[expansion.src_id as usize].push(expansion.polygon);
    }
    Ok(output)
}

pub(crate) fn propagate_waves(
    seeds: &[WaveSeed],
    boundary: &[ExPolygon],
    params: &RegionExpansionParameters,
) -> Result<Vec<RegionExpansion>, ClipperError> {
    if seeds.is_empty() {
        return Ok(Vec::new());
    }

    let mut output = Vec::new();
    let mut group_start = 0;
    while group_start < seeds.len() {
        let first = &seeds[group_start];
        let mut group_end = group_start + 1;
        while group_end < seeds.len()
            && seeds[group_end].boundary == first.boundary
            && seeds[group_end].src == first.src
        {
            group_end += 1;
        }
        let paths = seeds[group_start..group_end]
            .iter()
            .map(|seed| seed.path.clone())
            .collect::<Vec<_>>();
        let polygons = propagate_group(&paths, &boundary[first.boundary as usize], params)?;
        output.extend(polygons.into_iter().map(|polygon| RegionExpansion {
            polygon,
            src_id: first.src,
            boundary_id: first.boundary,
        }));
        group_start = group_end;
    }
    Ok(output)
}

fn propagate_group(
    seeds: &[Polygon],
    boundary: &ExPolygon,
    params: &RegionExpansionParameters,
) -> Result<Vec<Polygon>, ClipperError> {
    let mut bounds = BoundingBox::from_polygons(seeds)
        .expect("a supplied wave-seed group must contain nonempty paths");
    bounds.offset(params.max_inflation as i64);
    let clipping =
        clip_clipper_expolygons_with_subject_bbox(std::slice::from_ref(boundary), bounds);

    let initial = wavefront_initial(seeds, params.initial_step, params.arc_tolerance)?;
    let mut polygons = wavefront_clip(&initial, &clipping)?;
    for _ in 0..params.num_other_steps {
        let stepped = wavefront_step(&polygons, params.other_step, params.arc_tolerance)?;
        polygons = wavefront_clip(&stepped, &clipping)?;
    }
    Ok(polygons)
}

fn wavefront_initial(
    paths: &[Polygon],
    delta: f32,
    arc_tolerance: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    let mut output = Vec::with_capacity(paths.len());
    for path in paths {
        assert!(path.points().len() >= 2);
        let end = if path.points().first() == path.points().last() {
            EndType::Joined
        } else {
            EndType::Round
        };
        output.extend(offset_lines(
            std::slice::from_ref(path),
            delta,
            JoinType::Round,
            arc_tolerance,
            (end, 0.0),
        )?);
    }
    Ok(output)
}

// RegionExpansion.cpp:wavefront_step: the single-polygon overload grows CCW
// contours and shrinks CW holes; there is no caller-side sign/reverse repair.
fn wavefront_step(
    polygons: &[Polygon],
    delta: f32,
    arc_tolerance: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    raw_offset_paths(polygons, delta, JoinType::Round, arc_tolerance)
}

fn wavefront_clip(
    wavefront: &[Polygon],
    clipping: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    intersection_polygons_paths_with_fill(wavefront, clipping, FillRule::Positive)
}
