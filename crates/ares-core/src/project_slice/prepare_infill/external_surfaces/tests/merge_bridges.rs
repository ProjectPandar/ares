use super::helpers::{expolygon, surface_snapshots};
use crate::{
    geometry::{ClipperError, ExPolygon},
    project_slice::{
        prepare_infill::external_surfaces::{Bridge, merge_bridges::merge_bridges},
        region_slices::RegionSurfaceKind,
    },
};

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    expolygon(
        &[
            (min_x, min_y),
            (max_x, min_y),
            (max_x, max_y),
            (min_x, max_y),
        ],
        Vec::new(),
    )
}

fn bridge(expolygon: ExPolygon, group_id: u32, angle: Option<f64>) -> Bridge {
    Bridge {
        expolygon,
        group_id,
        angle,
    }
}

#[test]
fn task22o40_empty_input_does_not_reach_closing() {
    assert!(merge_bridges(Vec::new(), &[], f32::NAN).unwrap().is_empty());
}

fn disconnected_group() -> Vec<Bridge> {
    vec![
        bridge(rectangle(0, 0, 20, 20), 0, Some(0.125)),
        bridge(rectangle(100, 0, 120, 20), 0, None),
    ]
}

#[test]
fn task22o40_disconnected_group_output_is_deterministic() {
    let first = merge_bridges(disconnected_group(), &[], 2.0).unwrap();
    let second = merge_bridges(disconnected_group(), &[], 2.0).unwrap();

    assert_eq!(first.len(), 2);
    assert_eq!(surface_snapshots(&first), surface_snapshots(&second));
    assert!(first.iter().all(|surface| {
        let (kind, _, _, _, angle, _) = surface.as_parts();
        kind == RegionSurfaceKind::BottomBridge && angle == 0.125
    }));
}

#[test]
fn task22o40_propagates_clipper_coordinate_errors() {
    let invalid = rectangle(i64::MAX - 10, 0, i64::MAX, 10);

    assert!(matches!(
        merge_bridges(vec![bridge(invalid, 0, Some(0.5))], &[], 2.0),
        Err(ClipperError::CoordinateOutOfRange)
    ));
}
