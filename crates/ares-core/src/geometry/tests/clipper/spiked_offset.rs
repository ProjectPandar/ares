//! Repro: the narrow-split reconstruction polygon (case-twCWO9, MK4S)
//! offsets to a 60-point self-intersecting contour with back-step
//! slivers near the corner caps. Native ClipperLib
//! `ClipperOffset::Execute` resolves the union with `pftPositive` to a
//! single 20-gon (verified with a standalone build of OrcaSlicer
//! `deps_src/clipper/clipper.cpp`: pieces=1, area 6.5853488e13); the
//! ares engine must match.

use super::helpers::polygon;
use crate::geometry::clipper::union_safety_offset_ex;

/// The un-rotated reconstruction polygon (layer 1, case-twCWO9): the
/// corner-cap spikes survive the 45° round-trip and fragment the
/// safety-offset union in the ares engine (5 pieces vs the oracle's 1
/// expolygon per layer — the MK4S narrow-split divergence).
const UNROTATED: &[(i64, i64)] = &[
    (-5304358, -729850),
    (-5100819, -729850),
    (-4693740, -1136929),
    (-4286661, -1544008),
    (-3879582, -1951087),
    (-3472503, -2358165),
    (-3065424, -2765245),
    (-2658345, -3172324),
    (-2251266, -3579403),
    (-1844187, -3986482),
    (-1437108, -4393561),
    (-1030029, -4800640),
    (-622950, -5207719),
    (-215871, -5614798),
    (191208, -5639467),
    (598287, -5232388),
    (1005366, -4825309),
    (1412445, -4418230),
    (1819524, -4011151),
    (2226603, -3604072),
    (2633682, -3196993),
    (3040761, -2789914),
    (3447840, -2382835),
    (3854919, -1975756),
    (4261998, -1568677),
    (4465537, -1568677),
    (4465537, 1568677),
    (4261998, 1568677),
    (3854919, 1975757),
    (3447840, 2382836),
    (3040761, 2789915),
    (2633682, 3196994),
    (2226603, 3604073),
    (1819524, 4011152),
    (1412445, 4418231),
    (1005366, 4825310),
    (598287, 5232389),
    (191208, 5639468),
    (-215871, 5614797),
    (-622950, 5207718),
    (-1030029, 4800639),
    (-1437108, 4393560),
    (-1844187, 3986481),
    (-2251266, 3579402),
    (-2658345, 3172323),
    (-3065424, 2765244),
    (-3472503, 2358165),
    (-3879582, 1951086),
    (-4286661, 1544007),
    (-4693740, 1136928),
    (-5100819, 729849),
    (-5304358, 729849),
];

#[test]
fn mk4s_serpentine_offset_positive_union_is_single_piece() {
    let polygon = polygon(UNROTATED);
    let pieces = union_safety_offset_ex(&[polygon]).expect("union");
    assert_eq!(
        pieces.len(),
        1,
        "native ClipperLib resolves this contour to a single polygon; got {}",
        pieces.len()
    );
}
