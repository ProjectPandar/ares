use super::super::fill_surface;
use super::support::{ksr_params, large_bed_surface, point};
use crate::geometry::{CoordinateScale, Polyline};

fn transform_forward_expected() -> Vec<Polyline> {
    vec![
        Polyline::new(vec![
            point(350_032, 221_008),
            point(510_819, 195_281),
            point(518_125, 202_588),
            point(725_376, 211_913),
            point(824_338, 310_875),
            point(833_663, 518_125),
            point(881_645, 566_108),
            point(880_430, 598_615),
            point(479_070, 578_032),
            point(419_163, 518_125),
            point(409_838, 310_875),
            point(324_117, 225_154),
            point(270_335, 233_759),
            point(269_638, 210_057),
            point(127_066, 203_642),
            point(120_205, 9_452),
            point(129_497, 9_541),
            point(138_874, -210_327),
            point(310_875, -202_588),
            point(356_389, -157_074),
            point(518_608, -171_180),
        ]),
        Polyline::new(vec![
            point(665_412, -439_801),
            point(828_134, -433_775),
            point(833_663, -310_875),
            point(911_520, -233_019),
            point(896_582, 166_544),
            point(833_663, 103_625),
            point(824_338, -103_625),
            point(725_376, -202_587),
            point(518_125, -211_913),
            point(419_163, -310_875),
            point(412_941, -449_152),
            point(250_221, -455_177),
        ]),
        Polyline::new(vec![
            point(-572_781, 519_296),
            point(-410_118, 511_903),
            point(-419_163, 310_875),
            point(-518_125, 211_913),
            point(-725_376, 202_587),
            point(-824_338, 103_625),
            point(-833_663, -103_625),
            point(-845_125, -115_087),
            point(-832_741, -273_597),
        ]),
        Polyline::new(vec![
            point(-271_243, -526_017),
            point(-433_929, -532_892),
            point(-419_163, -518_125),
            point(-409_838, -310_875),
            point(-310_875, -211_913),
            point(-127_604, -203_666),
            point(-120_195, -8_453),
            point(-129_177, -8_539),
            point(-142_093, 210_182),
            point(-310_875, 202_588),
            point(-409_838, 103_625),
            point(-419_163, -103_625),
            point(-518_125, -202_588),
            point(-725_376, -211_913),
            point(-824_338, -310_875),
            point(-826_345, -355_471),
            point(-813_661, -517_808),
        ]),
    ]
}

fn transform_backward_expected() -> Vec<Polyline> {
    vec![
        Polyline::new(vec![
            point(887_677, 404_743),
            point(893_761, 242_025),
            point(824_911, 310_875),
            point(833_091, 518_125),
            point(758_837, 592_379),
            point(364_556, 572_160),
            point(418_590, 518_125),
            point(410_410, 310_875),
            point(518_125, 203_160),
            point(725_376, 211_340),
            point(833_091, 103_625),
            point(824_911, -103_625),
            point(909_858, -188_573),
            point(918_899, -430_413),
            point(828_240, -433_770),
            point(833_091, -310_875),
            point(725_376, -203_160),
            point(518_125, -211_340),
            point(474_095, -167_309),
            point(259_089, -148_613),
            point(269_628, 209_712),
            point(127_082, 204_086),
            point(120_205, 9_452),
            point(129_497, 9_541),
            point(138_857, -209_950),
            point(310_875, -203_160),
            point(418_590, -310_875),
            point(413_133, -449_144),
            point(250_412, -455_170),
        ]),
        Polyline::new(vec![
            point(-842_927, -143_204),
            point(-830_245, -305_541),
            point(-824_911, -310_875),
            point(-826_561, -352_696),
            point(-811_238, -548_835),
            point(-405_044, -531_671),
            point(-418_590, -518_125),
            point(-410_410, -310_875),
            point(-518_125, -203_160),
            point(-725_376, -211_340),
            point(-833_091, -103_625),
            point(-824_911, 103_625),
            point(-849_640, 128_354),
            point(-821_201, 298_985),
            point(-725_376, 203_160),
            point(-518_125, 211_340),
            point(-410_410, 103_625),
            point(-418_590, -103_625),
            point(-310_875, -211_340),
            point(-127_621, -204_107),
            point(-120_195, -8_453),
            point(-129_177, -8_539),
            point(-142_072, 209_823),
            point(-310_875, 203_160),
            point(-418_590, 310_875),
            point(-410_655, 511_927),
            point(-573_318, 519_320),
        ]),
    ]
}

#[test]
fn task22o45_large_bed_transform_forward_matches_pinned_orca() {
    let surface = large_bed_surface();
    let mut params = ksr_params();
    params.z = f64::from_bits(0x4020_6666_6666_6668);

    let actual = fill_surface(&surface, params, CoordinateScale::LargeBed).unwrap();

    assert_eq!(actual, transform_forward_expected());
}

#[test]
fn task22o45_large_bed_transform_backward_matches_pinned_orca() {
    let surface = large_bed_surface();
    let mut params = ksr_params();
    params.z = f64::from_bits(0x4008_0000_0000_0001);

    let actual = fill_surface(&surface, params, CoordinateScale::LargeBed).unwrap();

    assert_eq!(actual, transform_backward_expected());
}
