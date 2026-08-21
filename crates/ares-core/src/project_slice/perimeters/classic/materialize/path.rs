use crate::{
    SliceError,
    geometry::{
        BoundingBox, ClipperError, CoordinateScale, Polygon, Polyline,
        clip_clipper_polygons_with_subject_bbox, diff_pl, intersection_pl,
    },
};

use super::{
    super::traversal::{
        ClassicTraversalRecord, PendingExtrusionRole, PendingPathBranch,
        PostClassicTraversalPrintObject, TraversalSeed,
    },
    types::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
};

const EPSILON_MM: f64 = 1e-4;

pub(super) fn materialize_seed(
    object: &PostClassicTraversalPrintObject,
    record_index: usize,
    record: &ClassicTraversalRecord,
    seed: &TraversalSeed,
    scale: CoordinateScale,
) -> Result<Vec<ExtrusionPath>, SliceError> {
    match record.branch {
        PendingPathBranch::OrdinaryUnsplit { .. } => {
            Ok(materialize_ordinary(seed, record.layer_height))
        }
        PendingPathBranch::OverhangClipping { .. } => {
            materialize_overhang(object, record_index, record, seed, scale)
        }
    }
}

fn materialize_overhang(
    object: &PostClassicTraversalPrintObject,
    record_index: usize,
    record: &ClassicTraversalRecord,
    seed: &TraversalSeed,
    scale: CoordinateScale,
) -> Result<Vec<ExtrusionPath>, SliceError> {
    let lower = object
        .lower_series(record_index, seed.route)
        .last()
        .expect("a traversal route has a source-prepared lower polygon series");
    materialize_overhang_from_lower(record, seed, scale, lower)
}

pub(in crate::project_slice) fn materialize_overhang_from_lower(
    record: &ClassicTraversalRecord,
    seed: &TraversalSeed,
    scale: CoordinateScale,
    lower: &[Polygon],
) -> Result<Vec<ExtrusionPath>, SliceError> {
    let mut bounds = BoundingBox::from_polygon(&seed.polygon)
        .expect("a traversal seed polygon is source-validated nonempty");
    bounds.offset(scale.checked_scale(EPSILON_MM).ok_or_else(|| {
        SliceError::InvalidInput(
            "classic perimeter scaled epsilon is outside the supported coordinate range".into(),
        )
    })?);
    let filtered = clip_clipper_polygons_with_subject_bbox(lower, bounds);
    let subject = std::slice::from_ref(&seed.polygon);
    let inside = intersection_pl(subject, &filtered).map_err(map_clipper_error)?;
    let remain = diff_pl(subject, &filtered).map_err(map_clipper_error)?;
    let mut paths = Vec::with_capacity(inside.len() + remain.len());
    paths.extend(
        inside
            .into_iter()
            .filter(Polyline::is_valid)
            .map(|polyline| {
                path(
                    polyline,
                    role(seed.extrusion_role),
                    seed.mm3_per_mm,
                    seed.width,
                    record.layer_height as f32,
                )
            }),
    );
    paths.extend(
        remain
            .into_iter()
            .filter(Polyline::is_valid)
            .map(|polyline| {
                path(
                    polyline,
                    ExtrusionRole::OverhangPerimeter,
                    record.overhang_flow.mm3_per_mm,
                    record.overhang_flow.width,
                    record.overhang_flow.height,
                )
            }),
    );
    Ok(paths)
}

pub(super) fn materialize_ordinary(seed: &TraversalSeed, layer_height: f64) -> Vec<ExtrusionPath> {
    vec![path(
        seed.polygon.split_at_first_point(),
        role(seed.extrusion_role),
        seed.mm3_per_mm,
        seed.width,
        layer_height as f32,
    )]
}

fn role(role: PendingExtrusionRole) -> ExtrusionRole {
    match role {
        PendingExtrusionRole::ExternalPerimeter => ExtrusionRole::ExternalPerimeter,
        PendingExtrusionRole::Perimeter => ExtrusionRole::Perimeter,
    }
}

fn path(
    polyline: Polyline,
    role: ExtrusionRole,
    mm3_per_mm: f64,
    width: f32,
    height: f32,
) -> ExtrusionPath {
    ExtrusionPath {
        polyline: Polyline3 {
            points: polyline
                .into_points()
                .into_iter()
                .map(|point| Point3 {
                    x: point.x(),
                    y: point.y(),
                    z: 0,
                })
                .collect(),
            fitting: Vec::new(),
        },
        role,
        can_reverse: true,
        mm3_per_mm,
        width,
        height,
    }
}

fn map_clipper_error(error: ClipperError) -> SliceError {
    match error {
        ClipperError::CoordinateOutOfRange => SliceError::InvalidInput(
            "classic perimeter raw path coordinate is outside the supported Clipper range".into(),
        ),
        ClipperError::OpenPathMustBeSubject | ClipperError::OpenPathsRequirePolyTree => {
            unreachable!("closed polygon wrappers cannot produce open-path API errors")
        }
    }
}

#[cfg(test)]
pub(super) fn scaled_epsilon(scale: CoordinateScale) -> i64 {
    scale.checked_scale(EPSILON_MM).unwrap()
}
