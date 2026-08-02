use super::operations::execute;
use super::{coordinates, polygon, polyline};
use crate::geometry::clipper::ClipOperation;

#[test]
fn task22o6_large_valid_open_clipping_preserves_f64_scanline_rounding() {
    let n = 1_i64 << 54;
    let subject = [polyline(&[(n - 20, n + 5), (n + 30, n + 5)])];
    let clip = [polygon(&[
        (n, n),
        (n + 10, n),
        (n + 10, n + 10),
        (n, n + 10),
    ])];

    assert_eq!(
        coordinates(&execute(ClipOperation::Intersection, &subject, &clip)),
        vec![vec![(n + 10, n + 5), (n, n + 5)]]
    );
}
