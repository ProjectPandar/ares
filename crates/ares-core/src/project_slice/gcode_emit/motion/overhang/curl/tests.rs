//! Unit tests for the curled-perimeter geometry (values cross-checked
//! against the MK4S oracle forensics: layer-1 corner curl ≈ 0.222 and the
//! layer-2 artificial distance ≈ 0.050).

use super::CurledLine;
use super::geometry::{artificial_distance, curvatures, estimate_curled_up_height};

#[test]
fn curl_height_swelling_and_convex_tension_match_the_oracle_value() {
    // distance 0.225 (in the malformation band), corner curvature
    // ≈ (π/2)/3, layer height 0.2, width 0.45, no prior curl.
    let height = estimate_curled_up_height(0.225, 0.524, 0.2, 0.45, 0.0);
    assert!((height - 0.2225).abs() < 0.001, "got {height}");
}

#[test]
fn curl_height_without_curvature_stays_below_the_tolerance() {
    let height = estimate_curled_up_height(0.225, 0.0, 0.2, 0.45, 0.0);
    assert!((height - 0.00625).abs() < 1e-6, "got {height}");
}

#[test]
fn curl_height_decays_from_the_previous_line_outside_the_band() {
    // Distance 0 lies inside `3·width` but below the band: only the
    // decay branch applies.
    assert_eq!(estimate_curled_up_height(0.0, 0.524, 0.2, 0.45, 0.3), 0.15);
}

#[test]
fn artificial_distance_uses_the_coincident_curled_line() {
    // A curled line coincident with the slowed segment passes the
    // projected-length box check and yields width·(h/(height·10)).
    let curled = [CurledLine {
        x0: 0.0,
        y0: 0.0,
        x1: 9.55,
        y1: 0.0,
        curled_height: 0.2225,
    }];
    let distance = artificial_distance(&curled, (0.0, 0.0), (9.55, 0.0), 0.45, 0.2);
    assert!(
        (distance - 0.45 * 0.2225 / 2.0).abs() < 1e-4,
        "got {distance}"
    );
}

#[test]
fn artificial_distance_rejects_perpendicular_influence() {
    // A curled line crossing the segment perpendicular spans ~0 of the
    // direction, so the 40% projected-length gate rejects it.
    let curled = [CurledLine {
        x0: 5.0,
        y0: -1.0,
        x1: 5.0,
        y1: 1.0,
        curled_height: 0.2225,
    }];
    let distance = artificial_distance(&curled, (0.0, 0.0), (9.55, 0.0), 0.45, 0.2);
    assert_eq!(distance, 0.0);
}

#[test]
fn square_corners_carry_the_windowed_curvature() {
    // The exact-corner curvature over the 3mm window is (π/2)/3; the
    // seam-start corner has no backward tangent and stays zero.
    let square = [
        (4.775, 4.775),
        (-4.775, 4.775),
        (-4.775, -4.775),
        (4.775, -4.775),
        (4.775, 4.735),
    ];
    let curvature = curvatures(&square);
    // The seam-start corner has a zero backward tangent; `atan2(+0, −0)`
    // (IEEE signed zeros from multiplying the zero vector by negative
    // components) yields π there — faithful to upstream atan2 and
    // reproduced by the end-to-end MK4S oracle comparison.
    let seam_start = std::f32::consts::PI / 3.0;
    assert!((curvature[0] - seam_start).abs() < 1e-3);
    assert_eq!(curvature[4], 0.0);
    let corner = std::f64::consts::FRAC_PI_2 as f32 / 3.0;
    assert!((curvature[1].abs() - corner).abs() < 1e-3);
    assert!((curvature[2].abs() - corner).abs() < 1e-3);
    assert!(curvature[3].abs() > 0.01);
}
