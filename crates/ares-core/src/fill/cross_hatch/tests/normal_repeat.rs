use super::super::fill_surface;
use super::support::{ksr_params, point, raw_surface};
use crate::geometry::{CoordinateScale, Polyline};

fn expected() -> Vec<Polyline> {
    vec![
        Polyline::new(vec![
            point(8_834_954, 5_166_463),
            point(8_895_784, 3_539_281),
            point(6_563_806, 5_871_259),
            point(2_620_980, 5_669_063),
            point(9_056_756, -766_712),
            point(8_995_926, 860_470),
        ]),
        Polyline::new(vec![
            point(3_451_455, 2_217_896),
            point(2_703_360, 2_337_591),
            point(2_677_762, 1_467_259),
            point(1_299_276, 2_845_746),
            point(1_202_059, 94_520),
            point(1_294_965, 95_406),
            point(1_356_897, -1_356_897),
            point(4_478_595, -4_478_595),
            point(8_475_581, -4_330_559),
            point(5_333_626, -1_188_604),
            point(5_283_781, -1_720_289),
            point(4_193_592, -1_625_490),
        ]),
        Polyline::new(vec![
            point(-8_337_985, -2_600_532),
            point(-8_211_161, -4_223_905),
            point(-6_993_971, -5_441_094),
            point(-3_016_990, -5_273_053),
            point(-8_562_433, 272_389),
            point(-8_595_142, 691_064),
            point(-8_058_135, 3_913_112),
            point(-1_306_477, -2_838_545),
            point(-1_201_960, -84_520),
            point(-1_291_766, -85_375),
            point(-1_378_198, 1_378_198),
            point(-5_167_500, 5_167_499),
            point(-3_540_861, 5_093_561),
        ]),
    ]
}

fn transform_forward_expected() -> Vec<Polyline> {
    vec![
        Polyline::new(vec![
            point(3_164_588, 5_696_939),
            point(4_790_769, 5_780_334),
            point(4_191_712, 5_181_277),
            point(4_098_331, 3_108_767),
            point(3_241_115, 2_251_550),
            point(2_703_360, 2_337_591),
            point(2_696_390, 2_100_621),
            point(1_270_676, 2_036_383),
            point(1_202_059, 94_520),
            point(1_294_965, 95_406),
            point(1_388_727, -2_103_319),
            point(3_108_766, -2_025_821),
            point(3_563_856, -1_570_730),
            point(5_186_053, -1_711_791),
        ]),
        Polyline::new(vec![
            point(3_500_395, 2_210_065),
            point(5_108_264, 1_952_807),
            point(5_181_278, 2_025_821),
            point(7_253_788, 2_119_201),
            point(8_243_353, 3_108_767),
            point(8_336_734, 5_181_277),
            point(8_816_467, 5_661_011),
            point(8_965_837, 1_665_359),
            point(8_336_734, 1_036_255),
            point(8_243_353, -1_036_255),
            point(7_253_788, -2_025_820),
            point(5_181_278, -2_119_201),
            point(4_191_712, -3_108_767),
            point(4_129_409, -4_491_528),
            point(8_281_360, -4_337_753),
            point(8_336_734, -3_108_766),
            point(9_115_207, -2_330_294),
            point(9_054_377, -703_112),
        ]),
        Polyline::new(vec![
            point(-5_727_775, 5_192_966),
            point(-4_101_137, 5_119_028),
            point(-4_191_712, 3_108_767),
            point(-5_181_278, 2_119_201),
            point(-7_253_788, 2_025_820),
            point(-8_243_353, 1_036_255),
            point(-8_336_734, -1_036_255),
            point(-8_451_249, -1_150_771),
            point(-8_327_401, -2_736_018),
        ]),
        Polyline::new(vec![
            point(-2_712_496, -5_260_186),
            point(-4_339_363, -5_328_928),
            point(-4_191_712, -5_181_277),
            point(-4_098_331, -3_108_767),
            point(-3_108_766, -2_119_201),
            point(-1_276_043, -2_036_625),
            point(-1_201_960, -84_520),
            point(-1_291_766, -85_375),
            point(-1_420_935, 2_101_869),
            point(-3_108_766, 2_025_821),
            point(-4_098_332, 1_036_255),
            point(-4_191_712, -1_036_255),
            point(-5_181_278, -2_025_821),
            point(-7_253_788, -2_119_201),
            point(-8_243_353, -3_108_767),
            point(-8_263_444, -3_554_672),
            point(-8_136_618, -5_178_043),
        ]),
    ]
}

fn transform_backward_expected() -> Vec<Polyline> {
    vec![
        Polyline::new(vec![
            point(8_876_787, 4_047_447),
            point(8_937_616, 2_420_265),
            point(8_249_115, 3_108_767),
            point(8_330_972, 5_181_277),
            point(7_588_444, 5_923_805),
            point(3_645_619, 5_721_608),
            point(4_185_950, 5_181_277),
            point(4_104_093, 3_108_767),
            point(5_181_278, 2_031_582),
            point(7_253_788, 2_113_440),
            point(8_330_972, 1_036_255),
            point(8_249_115, -1_036_255),
            point(9_098_588, -1_885_728),
            point(9_188_996, -4_304_136),
            point(8_282_433, -4_337_713),
            point(8_330_972, -3_108_766),
            point(7_253_788, -2_031_582),
            point(5_181_278, -2_113_440),
            point(4_740_922, -1_673_084),
            point(2_590_898, -1_486_126),
            point(2_696_289, 2_097_149),
            point(1_270_834, 2_040_847),
            point(1_202_059, 94_520),
            point(1_294_965, 95_406),
            point(1_388_565, -2_099_524),
            point(3_108_766, -2_031_582),
            point(4_185_951, -3_108_767),
            point(4_131_339, -4_491_457),
            point(2_504_135, -4_551_723),
        ]),
        Polyline::new(vec![
            point(-8_429_272, -1_432_063),
            point(-8_302_447, -3_055_435),
            point(-8_249_115, -3_108_767),
            point(-8_265_624, -3_526_766),
            point(-8_112_376, -5_488_351),
            point(-4_050_505, -5_316_723),
            point(-4_185_950, -5_181_277),
            point(-4_104_093, -3_108_767),
            point(-5_181_278, -2_031_582),
            point(-7_253_788, -2_113_440),
            point(-8_330_972, -1_036_255),
            point(-8_249_115, 1_036_255),
            point(-8_496_396, 1_283_537),
            point(-8_212_018, 2_989_812),
            point(-7_253_788, 2_031_582),
            point(-5_181_278, 2_113_440),
            point(-4_104_093, 1_036_255),
            point(-4_185_950, -1_036_255),
            point(-3_108_766, -2_113_440),
            point(-1_276_212, -2_041_060),
            point(-1_201_960, -84_520),
            point(-1_291_766, -85_375),
            point(-1_420_720, 2_098_254),
            point(-3_108_766, 2_031_582),
            point(-4_185_951, 3_108_767),
            point(-4_106_542, 5_119_274),
            point(-5_733_180, 5_193_212),
        ]),
    ]
}

#[test]
fn task22o45_normal_repeat_negative_matches_pinned_orca() {
    let surface = raw_surface();
    let before = surface.clone();

    let actual = fill_surface(&surface, ksr_params(), CoordinateScale::Normal).unwrap();

    assert_eq!(actual, expected());
    assert_eq!(surface, before);
}

#[test]
fn task22o45_normal_transform_forward_matches_pinned_orca() {
    let surface = raw_surface();
    let mut params = ksr_params();
    params.z = f64::from_bits(0x4020_6666_6666_6668);

    let actual = fill_surface(&surface, params, CoordinateScale::Normal).unwrap();

    assert_eq!(actual, transform_forward_expected());
}

#[test]
fn task22o45_normal_transform_backward_matches_pinned_orca() {
    let surface = raw_surface();
    let mut params = ksr_params();
    params.z = f64::from_bits(0x4008_0000_0000_0001);

    let actual = fill_surface(&surface, params, CoordinateScale::Normal).unwrap();

    assert_eq!(actual, transform_backward_expected());
}
