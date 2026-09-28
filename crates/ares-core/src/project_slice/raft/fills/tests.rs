//! Live probe tests for `raft::fills`, split from `fills.rs` for the
//! <400 LOC rule.

use super::*;
use crate::geometry::Polygon;

#[test]
fn probe_flange_live() {
    let polygon = Polygon::new(vec![
        Point::new(101_059_000, 101_059_000),
        Point::new(118_941_000, 101_059_000),
        Point::new(118_941_000, 118_941_000),
        Point::new(101_059_000, 118_941_000),
    ]);
    let spec = RaftFillSpec {
        angle: std::f64::consts::FRAC_PI_2,
        spacing: 0.407_086_4,
        density: 0.9,
        sheath: false,
        width_mm: 0.42,
        preprocess: RaftFillPreprocess::Closing,
    };
    let out = raft_layer_fill(
        &[polygon],
        spec,
        Point::new(0, 0),
        crate::geometry::CoordinateScale::Normal,
    )
    .unwrap();
    eprintln!("FLANGE polylines={}", out.len());
}

/// KSR base layer at real lattice units.
#[test]
fn probe_base_connect() {
    let polygon = Polygon::new(vec![
        Point::new(-7_650_601, -7_650_601),
        Point::new(7_650_601, -7_650_601),
        Point::new(7_650_601, 7_650_601),
        Point::new(-7_650_601, 7_650_601),
    ]);
    let spec = RaftFillSpec {
        angle: 0.0,
        spacing: 0.407_086_4,
        density: 0.67,
        sheath: false,
        width_mm: 0.42,
        preprocess: RaftFillPreprocess::Closing,
    };
    let out = raft_layer_fill(
        &[polygon],
        spec,
        Point::new(0, 0),
        crate::geometry::CoordinateScale::Normal,
    )
    .unwrap();
    eprintln!(
        "BASECONNECT polylines={} lens={:?}",
        out.len(),
        out.iter().map(|p| p.points().len()).collect::<Vec<_>>()
    );
}
