use super::super::fill_surface;
use super::support::{ksr_params, point, raw_surface};
use crate::geometry::{CoordinateScale, Polyline};

fn expected() -> Vec<Polyline> {
    vec![
        Polyline::new(vec![
            point(9_090_037, -1_657_016),
            point(9_150_867, -3_284_198),
            point(8_090_234, -4_344_831),
            point(3_785_789, -4_504_255),
            point(9_001_497, 711_454),
            point(8_852_127, 4_707_105),
            point(5_579_544, 1_434_522),
            point(5_283_781, -1_720_289),
            point(2_653_456, -1_491_565),
            point(1_415_434, -2_729_588),
            point(1_294_965, 95_406),
            point(1_202_059, 94_520),
            point(1_242_628, 1_242_628),
            point(5_833_824, 5_833_824),
            point(4_207_642, 5_750_429),
        ]),
        Polyline::new(vec![
            point(-3_745_718, -5_303_844),
            point(-5_372_585, -5_372_585),
            point(-1_246_040, -1_246_040),
            point(-1_201_960, -84_520),
            point(-1_291_766, -85_375),
            point(-1_455_630, 2_689_392),
            point(-8_222_593, -4_077_571),
            point(-8_522_956, -232_913),
            point(-3_211_456, 5_078_588),
            point(-7_176_259, 5_258_807),
            point(-7_965_371, 4_469_694),
            point(-8_233_064, 2_863_530),
        ]),
    ]
}

#[test]
fn task22o45_normal_repeat_positive_matches_pinned_orca() {
    let surface = raw_surface();
    let mut params = ksr_params();
    params.z = f64::from_bits(0x4019_9999_9999_999d);

    let actual = fill_surface(&surface, params, CoordinateScale::Normal).unwrap();

    assert_eq!(actual, expected());
}
