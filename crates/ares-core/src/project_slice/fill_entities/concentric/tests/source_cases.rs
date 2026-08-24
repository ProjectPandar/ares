use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon, ThickPolyline};

use super::super::{finalize_polylines, generate_thick_polylines};

#[test]
fn concentric_arachne_preserves_source_short_variable_width_branch() {
    let domain = ExPolygon::new(
        Polygon::new(vec![
            Point::new(9265885, -34077090),
            Point::new(8974594, -33859458),
            Point::new(8664879, -33575086),
            Point::new(8381005, -33276688),
            Point::new(8094758, -32901119),
            Point::new(7887461, -32458923),
            Point::new(7722588, -31895290),
            Point::new(7796085, -31366343),
            Point::new(7935247, -30931244),
            Point::new(8180301, -30554561),
            Point::new(8572968, -30134408),
            Point::new(8742056, -29993727),
            Point::new(9113480, -29769610),
            Point::new(9466468, -29644972),
            Point::new(9835317, -29595924),
            Point::new(10147669, -29612227),
            Point::new(10453641, -29678656),
            Point::new(10720157, -29779920),
            Point::new(10799925, -29822782),
            Point::new(11234978, -29389843),
            Point::new(10462993, -28315615),
            Point::new(12942323, -25836286),
            Point::new(12790363, -25649162),
            Point::new(13226947, -25294615),
            Point::new(12911424, -24575263),
            Point::new(12805013, -24155078),
            Point::new(12194988, -24155078),
            Point::new(12088575, -24575265),
            Point::new(11766728, -25309035),
            Point::new(11328458, -25979827),
            Point::new(10785796, -26569320),
            Point::new(10153483, -27061471),
            Point::new(9448808, -27442825),
            Point::new(8690951, -27702996),
            Point::new(7900633, -27834873),
            Point::new(7099364, -27834873),
            Point::new(6309051, -27702995),
            Point::new(5957127, -27582181),
            Point::new(6023501, -27839695),
            Point::new(5749796, -27910242),
            Point::new(5755955, -27938823),
            Point::new(5519227, -27989840),
            Point::new(5645958, -28156631),
            Point::new(5835681, -28620451),
            Point::new(5882096, -28888950),
            Point::new(5898489, -29160653),
            Point::new(5864494, -29429570),
            Point::new(5799468, -29702544),
            Point::new(5666967, -29986557),
            Point::new(5522498, -30232762),
            Point::new(5347101, -30453186),
            Point::new(5236360, -30565450),
            Point::new(8692652, -34021741),
            Point::new(8562934, -34151459),
            Point::new(9241172, -34151459),
        ]),
        Vec::new(),
    );
    let mut output =
        generate_thick_polylines(domain, 377_079, 200_000, 0.4, CoordinateScale::Normal).unwrap();
    finalize_polylines(&mut output, 0, 40_000.0);

    assert!(output.contains(&ThickPolyline {
        points: vec![
            Point::new(10_494_353, -27_499_416),
            Point::new(10_527_208, -27_470_402),
        ],
        width: vec![355_770.0, 345_378.0],
        endpoints: (false, false),
    }));
}
