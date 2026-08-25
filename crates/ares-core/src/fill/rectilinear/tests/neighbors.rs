use super::{
    IntersectionKind, LinkQuality, LinkType, SegmentedLine, connect_region_neighbors, intersection,
    region,
};

#[test]
fn task22o88_region_overlap_connects_every_vertical_run() {
    let mut lines = vec![
        SegmentedLine {
            x: 0,
            intersections: vec![
                intersection(0, IntersectionKind::InnerLow),
                intersection(90, IntersectionKind::InnerHigh),
            ],
        },
        SegmentedLine {
            x: 10,
            intersections: vec![
                intersection(0, IntersectionKind::InnerLow),
                intersection(10, IntersectionKind::InnerHigh),
                intersection(20, IntersectionKind::OuterHigh),
                intersection(30, IntersectionKind::OuterLow),
                intersection(40, IntersectionKind::InnerLow),
                intersection(50, IntersectionKind::InnerHigh),
                intersection(60, IntersectionKind::OuterHigh),
                intersection(70, IntersectionKind::OuterLow),
                intersection(80, IntersectionKind::InnerLow),
                intersection(90, IntersectionKind::InnerHigh),
            ],
        },
    ];
    lines[0].intersections[0].next = Some((0, LinkType::Horizontal, LinkQuality::Valid));
    lines[0].intersections[1].next = Some((9, LinkType::Horizontal, LinkQuality::Valid));
    lines[1].intersections[0].previous = Some((0, LinkType::Horizontal, LinkQuality::Valid));
    lines[1].intersections[9].previous = Some((1, LinkType::Horizontal, LinkQuality::Valid));
    let mut regions = vec![
        region(0, 0, 0, 1),
        region(1, 1, 0, 1),
        region(1, 1, 4, 5),
        region(1, 1, 8, 9),
    ];

    connect_region_neighbors(&mut regions, &lines);

    assert_eq!(regions[0].right_neighbors, vec![1, 2, 3]);
    assert_eq!(regions[1].left_neighbors, vec![0]);
    assert_eq!(regions[2].left_neighbors, vec![0]);
    assert_eq!(regions[3].left_neighbors, vec![0]);
}
