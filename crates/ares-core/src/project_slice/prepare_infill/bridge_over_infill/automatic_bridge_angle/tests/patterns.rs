use crate::{ProcessInfillPattern, geometry::CoordinateScale};

use super::{line, polygon};
use crate::project_slice::prepare_infill::bridge_over_infill::automatic_bridge_angle::determine_automatic_bridge_angle;

#[test]
fn task22o51_empty_fallback_precedes_pattern_adjustments() {
    for (pattern, expected) in [
        (ProcessInfillPattern::Line, 0x3f50_624d_d2f1_a9fc),
        (ProcessInfillPattern::HilbertCurve, 0x3fe9_2a2c_7b2d_a5ed),
        (ProcessInfillPattern::OctagramSpiral, 0x3fc9_42bf_efea_106c),
    ] {
        let first = determine_automatic_bridge_angle(&[], &[], pattern, CoordinateScale::Normal);
        let second = determine_automatic_bridge_angle(&[], &[], pattern, CoordinateScale::Normal);
        assert_eq!(first.to_bits(), expected);
        assert_eq!(second.to_bits(), expected);
    }
}

#[test]
fn task22o51_every_ordinary_typed_pattern_preserves_the_same_angle() {
    let ordinary = [
        ProcessInfillPattern::Monotonic,
        ProcessInfillPattern::MonotonicLine,
        ProcessInfillPattern::Rectilinear,
        ProcessInfillPattern::AlignedRectilinear,
        ProcessInfillPattern::ZigZag,
        ProcessInfillPattern::CrossZag,
        ProcessInfillPattern::LockedZag,
        ProcessInfillPattern::Line,
        ProcessInfillPattern::Grid,
        ProcessInfillPattern::Triangles,
        ProcessInfillPattern::TriHexagon,
        ProcessInfillPattern::Cubic,
        ProcessInfillPattern::AdaptiveCubic,
        ProcessInfillPattern::QuarterCubic,
        ProcessInfillPattern::SupportCubic,
        ProcessInfillPattern::Lightning,
        ProcessInfillPattern::Honeycomb,
        ProcessInfillPattern::ThreeDHoneycomb,
        ProcessInfillPattern::LateralHoneycomb,
        ProcessInfillPattern::LateralLattice,
        ProcessInfillPattern::CrossHatch,
        ProcessInfillPattern::TpmsD,
        ProcessInfillPattern::TpmsFk,
        ProcessInfillPattern::Gyroid,
        ProcessInfillPattern::Concentric,
        ProcessInfillPattern::ArchimedeanChords,
    ];
    for pattern in ordinary {
        assert_eq!(
            determine_automatic_bridge_angle(&[], &[], pattern, CoordinateScale::LargeBed)
                .to_bits(),
            0x3f50_624d_d2f1_a9fc
        );
    }
}

#[test]
fn task22o51_orientation_fold_boundary_and_post_adjustment_do_not_normalize() {
    let area = [polygon(&[(0, 0), (2_000_001, 0)])];
    let pi_anchor = [line(10, 0, 0, 0)];
    for (pattern, expected) in [
        (ProcessInfillPattern::Line, 0x4012_d97c_7f33_21d2),
        (ProcessInfillPattern::HilbertCurve, 0x4015_fdbb_e9bb_a775),
        (ProcessInfillPattern::OctagramSpiral, 0x4013_a28c_59d5_433b),
    ] {
        assert_eq!(
            determine_automatic_bridge_angle(&area, &pi_anchor, pattern, CoordinateScale::Normal,)
                .to_bits(),
            expected
        );
    }

    let greater_than_pi = [line(0, 0, -3, -4)];
    assert_eq!(
        determine_automatic_bridge_angle(
            &area,
            &greater_than_pi,
            ProcessInfillPattern::Line,
            CoordinateScale::Normal,
        )
        .to_bits(),
        0x4003_fc17_6b7a_8560
    );
}
