use super::super::{
    apply::sort_arcs,
    graph::{EndpointHit, sort_endpoint_hits},
    types::Arc,
};

const EXPECTED: [usize; 35] = [
    33, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 0, 34,
];

#[test]
fn task22o44_arc_adapter_uses_fixed_msvc_equivalent_key_order() {
    let mut arcs = (0..35)
        .map(|identity| Arc {
            intersection_index: identity,
            length: match identity {
                33 => 10.0,
                34 => 30.0,
                _ => 20.0,
            },
        })
        .collect::<Vec<_>>();

    sort_arcs(&mut arcs);

    assert_eq!(
        arcs.into_iter()
            .map(|arc| arc.intersection_index)
            .collect::<Vec<_>>(),
        EXPECTED
    );
}

#[test]
fn task22o44_endpoint_adapter_uses_fixed_msvc_equivalent_key_order() {
    let mut hits = (0..35)
        .map(|identity| EndpointHit {
            contour_index: 7,
            segment_index: 11,
            t: match identity {
                33 => 0.25,
                34 => 0.75,
                _ => 0.5,
            },
            endpoint_index: identity,
        })
        .collect::<Vec<_>>();

    sort_endpoint_hits(&mut hits);

    assert_eq!(
        hits.into_iter()
            .map(|hit| hit.endpoint_index)
            .collect::<Vec<_>>(),
        EXPECTED
    );
}
