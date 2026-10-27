//! Unit tests for the curled-perimeter geometry (values cross-checked
//! against the MK4S oracle forensics: layer-1 corner curl ≈ 0.222 and the
//! layer-2 artificial distance ≈ 0.050).

use super::CurledLine;

// Failure modes: wrong previous-layer rollover; wrong equal-distance
// predecessor selection at the degenerate corner; wrong height decay.
#[test]
fn sermoon_layer_two_corner_height_matches_oracle() {
    use crate::geometry::CoordinateScale;
    use crate::project_slice::gcode_emit::motion::{CurlEstimate, estimate_object};
    let layers = [4.345001, 4.645001, 4.645001]
        .into_iter()
        .map(|half_side| CurlEstimate {
            entities: vec![(
                vec![
                    (half_side, half_side),
                    (-half_side, half_side),
                    (-half_side, -half_side),
                    (half_side, -half_side),
                    (half_side, half_side),
                ],
                0.61,
            )],
            height: 0.2,
            lower_boundary: None,
        })
        .collect::<Vec<_>>();
    let output = estimate_object(CoordinateScale::Normal, &layers);
    let corner = output[2][0];
    assert_eq!((corner.x0, corner.y0), (corner.x1, corner.y1));
    // curlp oracle kv5pcc97ack7c1fpmvni44bm5ffp4hm2, byte-identical
    // case-6qvP0t output: CURL layer=2 corner h=0.166997.
    assert!(
        (corner.curled_height - 0.166997).abs() < 1e-6,
        "layer-2 corner height {} != oracle 0.166997",
        corner.curled_height,
    );
}

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
        x0: 0,
        y0: 0,
        x1: 9_550_000,
        y1: 0,
        curled_height: 0.2225,
    }];
    let distance = artificial_distance(
        &curled,
        [(0.0, 0.0), (9.55, 0.0)],
        0.45,
        0.2,
        crate::geometry::CoordinateScale::Normal,
    );
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
        x0: 5_000_000,
        y0: -1_000_000,
        x1: 5_000_000,
        y1: 1_000_000,
        curled_height: 0.2225,
    }];
    let distance = artificial_distance(
        &curled,
        [(0.0, 0.0), (9.55, 0.0)],
        0.45,
        0.2,
        crate::geometry::CoordinateScale::Normal,
    );
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

/// Whole-file-excerpt oracle for `artificial_distance`
/// (`ExtrusionProcessor.hpp:382-420`) — Sermoon V1 cube10, stock
/// matrix case-6qvP0t. The fixture `oracle_artd_sermoon_v1.txt`
/// (sha256-provenanced header inside) carries upstream's stored
/// `CurledLine` endpoints, the per-segment `all_lines_in_radius` hit
/// count `nr`, and upstream's resulting `art` — the diverging
/// segments on the emitted gcode (orca art 0.0743/0.0750 vs ares'
/// ~0.081 crossing the 13%-of-width band edge).
#[test]
fn artificial_distance_matches_the_sermoon_v1_oracle_records() {
    let mut sets: Vec<(String, Vec<CurledLine>)> = Vec::new();
    let mut cases: Vec<(usize, [f64; 4], usize, f32)> = Vec::new();
    for raw in include_str!("oracle_artd_sermoon_v1.txt").lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        match fields[0] {
            "SET" => sets.push((fields[1].to_string(), Vec::new())),
            "CL" => sets.last_mut().unwrap().1.push(CurledLine {
                x0: fields[1].parse().unwrap(),
                y0: fields[2].parse().unwrap(),
                x1: fields[3].parse().unwrap(),
                y1: fields[4].parse().unwrap(),
                curled_height: fields[5].parse().unwrap(),
            }),
            "CASE" => cases.push((
                sets.len() - 1,
                [
                    fields[1].parse().unwrap(),
                    fields[2].parse().unwrap(),
                    fields[3].parse().unwrap(),
                    fields[4].parse().unwrap(),
                ],
                fields[5].parse().unwrap(),
                fields[6].parse().unwrap(),
            )),
            other => panic!("unknown fixture directive {other}"),
        }
    }
    for (set_index, segment, _oracle_nr, oracle_art) in cases {
        let (set_name, curled) = &sets[set_index];
        let art = artificial_distance(
            curled,
            [(segment[0], segment[1]), (segment[2], segment[3])],
            0.61,
            0.2,
            crate::geometry::CoordinateScale::Normal,
        );
        // f32-vs-dump-4-digit tolerance: upstream prints `art` at 4
        // decimals; 5e-4 covers the printed rounding but not the
        // ~0.007 band-edge miss this test exists to expose.
        assert!(
            (art - oracle_art).abs() < 5e-4,
            "set {set_name} seg ({},{})-({},{}): oracle art={oracle_art} nr={_oracle_nr} ares art={art}",
            segment[0],
            segment[1],
            segment[2],
            segment[3],
        );
    }
}
