use super::super::fill_surface;
use super::support::{ksr_params, large_bed_surface, point};
use crate::geometry::{CoordinateScale, Polyline};

fn repeat_negative_expected() -> Vec<Polyline> {
    vec![
        Polyline::new(vec![
            point(883_494, 516_641),
            point(889_578, 353_923),
            point(656_376, 587_125),
            point(262_095, 566_905),
            point(905_675, -76_674),
            point(899_591, 86_044),
        ]),
        Polyline::new(vec![
            point(345_143, 221_790),
            point(270_335, 233_759),
            point(267_775, 146_725),
            point(129_927, 284_574),
            point(120_205, 9_452),
            point(129_497, 9_541),
            point(135_691, -135_691),
            point(447_859, -447_859),
            point(847_556, -433_056),
            point(533_361, -118_861),
            point(528_377, -172_029),
            point(419_357, -162_550),
        ]),
        Polyline::new(vec![
            point(-833_799, -260_047),
            point(-821_117, -422_384),
            point(-699_392, -544_109),
            point(-301_696, -527_305),
            point(-856_244, 27_243),
            point(-859_514, 69_105),
            point(-805_813, 391_312),
            point(-130_647, -283_853),
            point(-120_195, -8_453),
            point(-129_177, -8_539),
            point(-137_820, 137_820),
            point(-516_749, 516_749),
            point(-354_086, 509_355),
        ]),
    ]
}

fn repeat_positive_expected() -> Vec<Polyline> {
    vec![
        Polyline::new(vec![
            point(909_002, -165_697),
            point(915_086, -328_415),
            point(809_019, -434_482),
            point(378_576, -450_424),
            point(900_149, 71_148),
            point(885_211, 470_711),
            point(557_955, 143_454),
            point(528_377, -172_029),
            point(265_343, -149_157),
            point(141_544, -272_956),
            point(129_497, 9_541),
            point(120_205, 9_452),
            point(124_261, 124_261),
            point(583_382, 583_382),
            point(420_764, 575_042),
        ]),
        Polyline::new(vec![
            point(-374_571, -530_384),
            point(-537_258, -537_258),
            point(-124_604, -124_604),
            point(-120_195, -8_453),
            point(-129_177, -8_539),
            point(-145_563, 268_937),
            point(-822_260, -407_760),
            point(-852_296, -23_295),
            point(-321_142, 507_858),
            point(-717_621, 525_880),
            point(-796_537, 446_964),
            point(-823_306, 286_348),
        ]),
    ]
}

#[test]
fn task22o45_large_bed_repeat_negative_matches_pinned_orca() {
    let surface = large_bed_surface();
    let before = surface.clone();

    let actual = fill_surface(&surface, ksr_params(), CoordinateScale::LargeBed).unwrap();

    assert_eq!(actual, repeat_negative_expected());
    assert_eq!(surface, before);
}

#[test]
fn task22o45_large_bed_repeat_positive_matches_pinned_orca() {
    let surface = large_bed_surface();
    let mut params = ksr_params();
    params.z = f64::from_bits(0x4019_9999_9999_999d);

    let actual = fill_surface(&surface, params, CoordinateScale::LargeBed).unwrap();

    assert_eq!(actual, repeat_positive_expected());
}
