use super::{JoinType, offset_paths};
use crate::geometry::{ClipperError, Polygon};

pub(crate) fn opening_paths(
    paths: &[Polygon],
    delta1: f32,
    delta2: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    let shrunk = offset_paths(paths, -delta1, join_type, miter_limit)?;
    offset_paths(&shrunk, delta2, join_type, miter_limit)
}

#[cfg(test)]
pub(crate) fn opening_paths_with_interstage(
    paths: &[Polygon],
    deltas: [f32; 2],
    join_type: JoinType,
    miter_limit: f64,
    interstage: impl FnOnce(&[Polygon]) -> Result<(), ClipperError>,
) -> Result<Vec<Polygon>, ClipperError> {
    let [delta1, delta2] = deltas;
    let shrunk = offset_paths(paths, -delta1, join_type, miter_limit)?;
    interstage(&shrunk)?;
    offset_paths(&shrunk, delta2, join_type, miter_limit)
}

#[cfg(test)]
pub(in crate::geometry) fn opening_path_configurations_for_test(
    delta1: f32,
    delta2: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> [(f64, f64); 2] {
    [
        super::offset_configuration_for_test(-delta1, join_type, miter_limit),
        super::offset_configuration_for_test(delta2, join_type, miter_limit),
    ]
}
