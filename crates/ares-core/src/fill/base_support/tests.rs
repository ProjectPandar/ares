use super::*;

fn contour_params(contour: &[Point]) -> Vec<f64> {
    let mut params = Vec::with_capacity(contour.len() + 1);
    let mut total = 0.0;
    params.push(0.0);
    for index in 0..contour.len() {
        let next = (index + 1) % contour.len();
        let (dx, dy) = (
            contour[next].x() - contour[index].x(),
            contour[next].y() - contour[index].y(),
        );
        total += ((dx * dx + dy * dy) as f64).sqrt();
        params.push(total);
    }
    params
}

fn rectangle() -> Vec<Point> {
    vec![
        Point::new(0, 0),
        Point::new(10_000, 0),
        Point::new(10_000, 10_000),
        Point::new(0, 10_000),
    ]
}

/// A trace inside the band [4000, 6000]: the interpolated band-edge
/// split points are removed by the vertical-connector pairing,
/// leaving the collinear boundary run
/// (`FillBase.cpp:1952-2140`).
#[test]
fn band_trace_keeps_boundary_run() {
    let contour = rectangle();
    let params = contour_params(&contour);
    let mut out = Vec::new();

    emit_loops_in_band(
        4_000, 6_000, &contour, &params, 5_000.0, 25_000.0, 10.0, &mut out,
    );

    assert_eq!(out.len(), 1);
    assert_eq!(
        out[0].points(),
        &[
            Point::new(5_000, 0),
            Point::new(10_000, 0),
            Point::new(10_000, 10_000),
            Point::new(5_000, 10_000),
        ]
    );
}

/// Arch pieces below `min_length` are dropped (`:1962`).
#[test]
fn short_arches_are_dropped() {
    let contour = rectangle();
    let params = contour_params(&contour);
    let mut out = Vec::new();

    emit_loops_in_band(
        4_000, 6_000, &contour, &params, 5_000.0, 25_000.0, 30_000.0, &mut out,
    );

    assert!(out.is_empty(), "the arch is below min_length");
}
