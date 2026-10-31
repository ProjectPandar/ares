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
