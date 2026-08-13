use crate::{OrcaBool, OrcaFloat, ProcessInfillPattern, geometry::CoordinateScale};

use super::{
    determine_candidate_bridge_angle, polygon, polyline, region, snapshot_polygons,
    snapshot_polylines,
};

#[test]
fn task22o60_real_o51_then_o49_composition_preserves_inputs_and_exact_bits() {
    let area = vec![polygon(&[(0, 0), (1, 0)])];
    let anchors = vec![polyline(&[(8, 9)])];
    let boundaries = vec![polyline(&[(10, 20), (30, 40)])];
    let mut region = region();
    region.sparse_infill_pattern = ProcessInfillPattern::HilbertCurve;
    region.internal_bridge_angle = OrcaFloat(17.3);
    region.relative_bridge_angle = OrcaBool(true);
    region.align_infill_direction_to_model = OrcaBool(true);
    let rotation = f64::from_bits(0x7ff8_0000_0000_0084);
    let area_before = snapshot_polygons(&area);
    let anchors_before = snapshot_polylines(&anchors);
    let boundaries_before = snapshot_polylines(&boundaries);
    let region_before = region.clone();
    let area_ptr = area[0].points().as_ptr();
    let anchor_ptr = anchors[0].points().as_ptr();
    let boundary_ptr = boundaries[0].points().as_ptr();

    let first = determine_candidate_bridge_angle(
        &area,
        &anchors,
        &boundaries,
        &region,
        rotation,
        CoordinateScale::Normal,
    );
    let second = determine_candidate_bridge_angle(
        &area,
        &anchors,
        &boundaries,
        &region,
        rotation,
        CoordinateScale::Normal,
    );

    assert_eq!(first.to_bits(), 0x3ff1_69d7_5577_8e43);
    assert_eq!(second.to_bits(), first.to_bits());
    assert_eq!(snapshot_polygons(&area), area_before);
    assert_eq!(snapshot_polylines(&anchors), anchors_before);
    assert_eq!(snapshot_polylines(&boundaries), boundaries_before);
    assert_eq!(region, region_before);
    assert_eq!(area[0].points().as_ptr(), area_ptr);
    assert_eq!(anchors[0].points().as_ptr(), anchor_ptr);
    assert_eq!(boundaries[0].points().as_ptr(), boundary_ptr);
}

#[test]
fn task22o60_fallback_neutral_pattern_ignores_sparse_pattern_adjustment() {
    let area = vec![polygon(&[(0, 0), (1, 0)])];
    let anchors = Vec::new();
    let boundaries = vec![polyline(&[(8, 9)])];
    let mut region = region();
    region.sparse_infill_pattern = ProcessInfillPattern::HilbertCurve;

    let output = determine_candidate_bridge_angle(
        &area,
        &anchors,
        &boundaries,
        &region,
        0.0,
        CoordinateScale::LargeBed,
    );

    assert_eq!(output.to_bits(), 0x3f50_624d_d2f1_a9fc);
}
