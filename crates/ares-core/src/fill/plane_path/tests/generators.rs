use super::super::{PlanePathPattern, generate, output::InfillPolylineOutput};
use crate::geometry::Point;

#[test]
fn hilbert_uses_source_state_table_and_offset_order() {
    let mut output = InfillPolylineOutput::plain(1.0);

    generate::generate(
        PlanePathPattern::HilbertCurve,
        -2,
        3,
        1,
        6,
        0.0,
        &mut output,
    )
    .unwrap();

    assert_eq!(
        output.result(),
        [
            (0, 0),
            (0, 1),
            (1, 1),
            (1, 0),
            (2, 0),
            (3, 0),
            (3, 1),
            (2, 1),
            (2, 2),
            (3, 2),
            (3, 3),
            (2, 3),
            (1, 3),
            (1, 2),
            (0, 2),
            (0, 3),
        ]
        .map(|(x, y)| Point::new(x - 2, y + 3))
    );
}

#[test]
fn archimedean_uses_source_chord_error_increment() {
    let mut output = InfillPolylineOutput::plain(100.0);

    generate::generate(
        PlanePathPattern::ArchimedeanChords,
        -2,
        -2,
        2,
        2,
        0.1,
        &mut output,
    )
    .unwrap();

    assert_eq!(
        &output.result()[..3],
        [Point::new(0, 0), Point::new(100, 0), Point::new(71, 90)]
    );
}

#[test]
fn octagram_retains_the_sixteen_point_ring_order() {
    let mut output = InfillPolylineOutput::plain(10.0);

    generate::generate(
        PlanePathPattern::OctagramSpiral,
        -2,
        -2,
        2,
        2,
        0.0,
        &mut output,
    )
    .unwrap();

    assert_eq!(
        &output.result()[..17],
        [
            (0, 0),
            (14, 0),
            (24, 10),
            (10, 10),
            (10, 24),
            (0, 14),
            (-10, 24),
            (-10, 10),
            (-24, 10),
            (-14, 0),
            (-24, -10),
            (-10, -10),
            (-10, -24),
            (0, -14),
            (10, -24),
            (10, -10),
            (38, -10),
        ]
        .map(|(x, y)| Point::new(x, y))
    );
}
