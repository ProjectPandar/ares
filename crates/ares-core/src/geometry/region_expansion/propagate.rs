use super::{RegionExpansion, RegionExpansionParameters, WaveSeed};
use crate::geometry::clipper::{
    ClipOperation, Clipper, ClipperError, ClipperOffset, ClipperOptions, FillRule, JoinType,
    PathRole, orientation,
};
use crate::geometry::{BoundingBox, ExPolygon, Polygon, clip_clipper_expolygons_with_subject_bbox};

pub(crate) fn propagate_waves(
    seeds: &[WaveSeed],
    boundary: &[ExPolygon],
    params: &RegionExpansionParameters,
) -> Result<Vec<RegionExpansion>, ClipperError> {
    if seeds.is_empty() {
        return Ok(Vec::new());
    }

    let mut offsetter = ClipperOffset::default();
    offsetter.set_arc_tolerance(params.arc_tolerance);
    offsetter.set_shortest_edge_length(params.shortest_edge_length);

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
        let polygons = propagate_group(
            &mut offsetter,
            &paths,
            &boundary[first.boundary as usize],
            params,
        )?;
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
    offsetter: &mut ClipperOffset,
    seeds: &[Polygon],
    boundary: &ExPolygon,
    params: &RegionExpansionParameters,
) -> Result<Vec<Polygon>, ClipperError> {
    let mut bounds = BoundingBox::from_polygons(seeds)
        .expect("a supplied wave-seed group must contain nonempty paths");
    bounds.offset(params.max_inflation as i64);
    let clipping =
        clip_clipper_expolygons_with_subject_bbox(std::slice::from_ref(boundary), bounds);

    let initial = wavefront_initial(offsetter, seeds, params.initial_step)?;
    let mut polygons = wavefront_clip(&initial, &clipping)?;
    for _ in 0..params.num_other_steps {
        let stepped = wavefront_step(offsetter, &polygons, params.other_step)?;
        polygons = wavefront_clip(&stepped, &clipping)?;
    }
    Ok(polygons)
}

fn wavefront_initial(
    offsetter: &mut ClipperOffset,
    paths: &[Polygon],
    delta: f32,
) -> Result<Vec<Polygon>, ClipperError> {
    let mut output = Vec::with_capacity(paths.len());
    for path in paths {
        assert!(path.points().len() >= 2);
        let closed = path.points().first() == path.points().last();
        offsetter.clear();
        if closed {
            offsetter.add_closed_line(path, JoinType::Round);
        } else {
            offsetter.add_open_round_path(path, JoinType::Round);
        }
        output.append(&mut offsetter.execute_paths(f64::from(delta))?);
    }
    Ok(output)
}

pub(in crate::geometry) fn wavefront_counter_clockwise(polygon: &Polygon) -> bool {
    orientation(polygon)
}

#[cfg(test)]
pub(in crate::geometry) fn wavefront_step_for_test(
    polygons: &[Polygon],
    delta: f32,
    arc_tolerance: f64,
    shortest_edge_length: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    let mut offsetter = ClipperOffset::default();
    offsetter.set_arc_tolerance(arc_tolerance);
    offsetter.set_shortest_edge_length(shortest_edge_length);
    wavefront_step(&mut offsetter, polygons, delta)
}

fn wavefront_step(
    offsetter: &mut ClipperOffset,
    polygons: &[Polygon],
    delta: f32,
) -> Result<Vec<Polygon>, ClipperError> {
    let mut output = Vec::with_capacity(polygons.len());
    for polygon in polygons {
        offsetter.clear();
        offsetter.add_closed_path(polygon, JoinType::Round);
        let counter_clockwise = wavefront_counter_clockwise(polygon);
        let applied_delta = if counter_clockwise { delta } else { -delta };
        let mut expanded = offsetter.execute_paths(f64::from(applied_delta))?;
        if !counter_clockwise {
            for path in &mut expanded {
                path.reverse();
            }
        }
        output.append(&mut expanded);
    }
    Ok(output)
}

fn wavefront_clip(
    wavefront: &[Polygon],
    clipping: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper.add_closed_paths(wavefront, PathRole::Subject)?;
    clipper.add_closed_paths(clipping, PathRole::Clip)?;
    clipper.execute_paths(
        ClipOperation::Intersection,
        FillRule::Positive,
        FillRule::Positive,
    )
}
