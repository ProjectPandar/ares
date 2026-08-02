use super::{ClipperOffset, JoinType};
use crate::geometry::Polygon;
use crate::geometry::clipper::bounds::negative_outer;
use crate::geometry::clipper::predicates::area;
use crate::geometry::clipper::{
    ClipOperation, Clipper, ClipperError, ClipperOptions, FillRule, PathRole, PolyTree,
};

const SHORTEST_EDGE_FACTOR: f64 = 0.005;

impl ClipperOffset {
    pub(crate) fn execute_paths(&mut self, delta: f64) -> Result<Vec<Polygon>, ClipperError> {
        let generated = self.generate_raw(delta);
        if delta > 0.0 {
            union_paths(&generated, FillRule::Positive)
        } else {
            negative_paths(&generated)
        }
    }

    pub(crate) fn execute_polytree(&mut self, delta: f64) -> Result<PolyTree, ClipperError> {
        let generated = self.generate_raw(delta);
        if delta > 0.0 {
            union_tree(&generated, FillRule::Positive)
        } else {
            negative_tree(&generated)
        }
    }
}

pub(crate) fn raw_offset_paths(
    paths: &[Polygon],
    delta: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    let mut output = Vec::with_capacity(paths.len());
    for path in paths {
        let mut offset = configured_offset(delta, join_type, miter_limit);
        offset.add_closed_path(path, join_type);
        let counter_clockwise = area(path.points()) >= 0.0;
        let applied_delta = if counter_clockwise { delta } else { -delta };
        let mut path_output = offset.execute_paths(f64::from(applied_delta))?;
        if !counter_clockwise {
            for path in &mut path_output {
                path.reverse();
            }
        }
        output.append(&mut path_output);
    }
    Ok(output)
}

pub(crate) fn offset_paths(
    paths: &[Polygon],
    delta: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    if delta > 0.0 {
        expand_paths(paths, delta, join_type, miter_limit)
    } else {
        shrink_paths(paths, -delta, join_type, miter_limit)
    }
}

pub(crate) fn offset_paths_tree(
    paths: &[Polygon],
    delta: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> Result<PolyTree, ClipperError> {
    if delta > 0.0 {
        let raw = raw_offset_paths(paths, delta, join_type, miter_limit)?;
        union_tree(&raw, FillRule::NonZero)
    } else {
        shrink_paths_tree(paths, -delta, join_type, miter_limit)
    }
}

pub(super) fn configured_offset(
    delta: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> ClipperOffset {
    let mut offset = ClipperOffset::default();
    if join_type == JoinType::Round {
        offset.set_arc_tolerance(miter_limit);
    } else {
        offset.set_miter_limit(miter_limit);
    }
    offset.set_shortest_edge_length((f64::from(delta) * SHORTEST_EDGE_FACTOR).abs());
    offset
}

pub(super) fn union_paths(
    paths: &[Polygon],
    fill_rule: FillRule,
) -> Result<Vec<Polygon>, ClipperError> {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper.add_closed_paths(paths, PathRole::Subject)?;
    clipper.execute_paths(ClipOperation::Union, fill_rule, fill_rule)
}

pub(super) fn union_tree(paths: &[Polygon], fill_rule: FillRule) -> Result<PolyTree, ClipperError> {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper.add_closed_paths(paths, PathRole::Subject)?;
    Ok(clipper.execute_polytree(ClipOperation::Union, fill_rule, fill_rule))
}

pub(super) fn difference_paths(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper.add_closed_paths(subject, PathRole::Subject)?;
    clipper.add_closed_paths(clip, PathRole::Clip)?;
    clipper.execute_paths(
        ClipOperation::Difference,
        FillRule::NonZero,
        FillRule::NonZero,
    )
}

fn expand_paths(
    paths: &[Polygon],
    delta: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    let raw = raw_offset_paths(paths, delta, join_type, miter_limit)?;
    union_paths(&raw, FillRule::NonZero)
}

fn shrink_paths(
    paths: &[Polygon],
    delta: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    let raw = raw_offset_paths(paths, -delta, join_type, miter_limit)?;
    if raw.is_empty() {
        Ok(Vec::new())
    } else {
        negative_paths(&raw)
    }
}

fn shrink_paths_tree(
    paths: &[Polygon],
    delta: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> Result<PolyTree, ClipperError> {
    let raw = raw_offset_paths(paths, -delta, join_type, miter_limit)?;
    if raw.is_empty() {
        Ok(PolyTree::empty())
    } else {
        negative_tree(&raw)
    }
}

fn negative_paths(paths: &[Polygon]) -> Result<Vec<Polygon>, ClipperError> {
    let mut clipper = negative_clipper(paths)?;
    let mut output =
        clipper.execute_paths(ClipOperation::Union, FillRule::Negative, FillRule::Negative)?;
    if !output.is_empty() {
        output.remove(0);
    }
    Ok(output)
}

fn negative_tree(paths: &[Polygon]) -> Result<PolyTree, ClipperError> {
    let mut clipper = negative_clipper(paths)?;
    let mut output =
        clipper.execute_polytree(ClipOperation::Union, FillRule::Negative, FillRule::Negative);
    output.remove_outermost_polygon();
    Ok(output)
}

fn negative_clipper(paths: &[Polygon]) -> Result<Clipper, ClipperError> {
    let mut clipper = Clipper::new(ClipperOptions {
        reverse_solution: true,
        preserve_collinear: false,
        strictly_simple: false,
    });
    clipper.add_closed_paths(paths, PathRole::Subject)?;
    let outer = negative_outer(clipper.bounds());
    clipper.add_closed_path(&outer, PathRole::Subject)?;
    Ok(clipper)
}
