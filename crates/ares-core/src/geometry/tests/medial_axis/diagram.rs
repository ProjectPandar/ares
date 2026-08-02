use boostvoronoi::prelude::SourceCategory;

use crate::geometry::{Line, Point, medial_axis::diagram};

#[test]
fn task22o13_single_segment_topology_is_literal_and_source_indexed() {
    let output = diagram::build(&[Line::new(Point::new(0, 0), Point::new(10, 0))]).unwrap();
    assert_eq!(
        (
            output.num_cells(),
            output.num_vertices(),
            output.num_edges()
        ),
        (3, 0, 4)
    );
    assert_eq!(
        output
            .cells()
            .iter()
            .map(|cell| (
                cell.id().u32(),
                cell.source_index().u32(),
                cell.source_category()
            ))
            .collect::<Vec<_>>(),
        vec![
            (0, 0, SourceCategory::SegmentStart),
            (1, 0, SourceCategory::Segment),
            (2, 0, SourceCategory::SegmentEnd),
        ]
    );
    let edges = (0..output.num_edges())
        .map(|index| {
            let edge = diagram::edge_index(&output, index);
            let value = output.edge(edge).unwrap();
            (
                edge.u32(),
                output.edge_get_twin(edge).unwrap().u32(),
                output.edge_get_cell(edge).unwrap().u32(),
                output
                    .edge_get_vertex0(edge)
                    .unwrap()
                    .map(|vertex| vertex.u32()),
                output
                    .edge_get_vertex1(edge)
                    .unwrap()
                    .map(|vertex| vertex.u32()),
                value.is_primary(),
                output.edge_is_finite(edge).unwrap(),
                output.edge_get_next(edge).unwrap().u32(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        edges,
        vec![
            (0, 1, 0, None, None, false, false, 0),
            (1, 0, 1, None, None, false, false, 2),
            (2, 3, 1, None, None, false, false, 1),
            (3, 2, 2, None, None, false, false, 3),
        ]
    );
}

#[test]
fn task22o13_finite_multisite_topology_is_fully_literal() {
    let lines = vec![
        Line::new(Point::new(0, 0), Point::new(20, 0)),
        Line::new(Point::new(20, 0), Point::new(20, 10)),
        Line::new(Point::new(20, 10), Point::new(0, 10)),
        Line::new(Point::new(0, 10), Point::new(0, 0)),
    ];
    let output = diagram::build(&lines).unwrap();
    let cells = output
        .cells()
        .iter()
        .map(|cell| {
            format!(
                "cell={} source={} category={:?} point={} segment={} incident={:?}",
                cell.id().u32(),
                cell.source_index().u32(),
                cell.source_category(),
                cell.contains_point(),
                cell.contains_segment(),
                cell.get_incident_edge().map(|edge| edge.u32())
            )
        })
        .collect::<Vec<_>>();
    let vertices = output
        .vertices()
        .iter()
        .enumerate()
        .map(|(id, vertex)| format!("vertex={id} x={} y={}", vertex.x(), vertex.y()))
        .collect::<Vec<_>>();
    let edges = (0..output.num_edges())
        .map(|index| {
            let edge = diagram::edge_index(&output, index);
            let value = output.edge(edge).unwrap();
            format!(
                "edge={} twin={} cell={} v0={:?} v1={:?} primary={} secondary={} finite={} next={} rot={:?}",
                edge.u32(),
                output.edge_get_twin(edge).unwrap().u32(),
                output.edge_get_cell(edge).unwrap().u32(),
                output.edge_get_vertex0(edge).unwrap().map(|vertex| vertex.u32()),
                output.edge_get_vertex1(edge).unwrap().map(|vertex| vertex.u32()),
                value.is_primary(),
                value.is_secondary(),
                output.edge_is_finite(edge).unwrap(),
                output.edge_get_next(edge).unwrap().u32(),
                output.edge_rot_next(edge).map(|edge| edge.u32()),
            )
        })
        .collect::<Vec<_>>();
    let incident_cycles = output
        .cells()
        .iter()
        .map(|cell| {
            let Some(first) = cell.get_incident_edge() else {
                return Vec::new();
            };
            let mut cycle = Vec::new();
            let mut edge = first;
            loop {
                cycle.push(edge.u32());
                edge = output.edge_get_next(edge).unwrap();
                if edge == first {
                    break;
                }
            }
            cycle
        })
        .collect::<Vec<_>>();
    assert_eq!(
        cells,
        [
            "cell=0 source=0 category=SegmentStart point=true segment=false incident=Some(0)",
            "cell=1 source=3 category=Segment point=false segment=true incident=Some(1)",
            "cell=2 source=2 category=SegmentEnd point=true segment=false incident=Some(3)",
            "cell=3 source=0 category=Segment point=false segment=true incident=Some(5)",
            "cell=4 source=2 category=Segment point=false segment=true incident=Some(9)",
            "cell=5 source=0 category=SegmentEnd point=true segment=false incident=Some(15)",
            "cell=6 source=1 category=Segment point=false segment=true incident=Some(17)",
            "cell=7 source=1 category=SegmentEnd point=true segment=false incident=Some(23)",
        ]
    );
    assert_eq!(
        vertices,
        [
            "vertex=0 x=0 y=0",
            "vertex=1 x=0 y=10",
            "vertex=2 x=5 y=5",
            "vertex=3 x=20 y=0",
            "vertex=4 x=15 y=5",
            "vertex=5 x=20 y=10",
        ]
    );
    assert_eq!(
        edges,
        [
            "edge=0 twin=1 cell=0 v0=Some(0) v1=None primary=false secondary=true finite=false next=4 rot=Some(5)",
            "edge=1 twin=0 cell=1 v0=None v1=Some(0) primary=false secondary=true finite=false next=7 rot=Some(3)",
            "edge=2 twin=3 cell=1 v0=Some(1) v1=None primary=false secondary=true finite=false next=1 rot=Some(11)",
            "edge=3 twin=2 cell=2 v0=None v1=Some(1) primary=false secondary=true finite=false next=8 rot=Some(9)",
            "edge=4 twin=5 cell=0 v0=None v1=Some(0) primary=false secondary=true finite=false next=0 rot=Some(1)",
            "edge=5 twin=4 cell=3 v0=Some(0) v1=None primary=false secondary=true finite=false next=14 rot=Some(7)",
            "edge=6 twin=7 cell=3 v0=Some(2) v1=Some(0) primary=true secondary=false finite=true next=5 rot=Some(13)",
            "edge=7 twin=6 cell=1 v0=Some(0) v1=Some(2) primary=true secondary=false finite=true next=10 rot=Some(0)",
            "edge=8 twin=9 cell=2 v0=Some(1) v1=None primary=false secondary=true finite=false next=3 rot=Some(2)",
            "edge=9 twin=8 cell=4 v0=None v1=Some(1) primary=false secondary=true finite=false next=11 rot=Some(24)",
            "edge=10 twin=11 cell=1 v0=Some(2) v1=Some(1) primary=true secondary=false finite=true next=2 rot=Some(6)",
            "edge=11 twin=10 cell=4 v0=Some(1) v1=Some(2) primary=true secondary=false finite=true next=13 rot=Some(8)",
            "edge=12 twin=13 cell=3 v0=Some(4) v1=Some(2) primary=true secondary=false finite=true next=6 rot=Some(18)",
            "edge=13 twin=12 cell=4 v0=Some(2) v1=Some(4) primary=true secondary=false finite=true next=21 rot=Some(10)",
            "edge=14 twin=15 cell=3 v0=None v1=Some(3) primary=false secondary=true finite=false next=19 rot=Some(4)",
            "edge=15 twin=14 cell=5 v0=Some(3) v1=None primary=false secondary=true finite=false next=16 rot=Some(17)",
            "edge=16 twin=17 cell=5 v0=None v1=Some(3) primary=false secondary=true finite=false next=15 rot=Some(14)",
            "edge=17 twin=16 cell=6 v0=Some(3) v1=None primary=false secondary=true finite=false next=22 rot=Some(19)",
            "edge=18 twin=19 cell=6 v0=Some(4) v1=Some(3) primary=true secondary=false finite=true next=17 rot=Some(21)",
            "edge=19 twin=18 cell=3 v0=Some(3) v1=Some(4) primary=true secondary=false finite=true next=12 rot=Some(15)",
            "edge=20 twin=21 cell=6 v0=Some(5) v1=Some(4) primary=true secondary=false finite=true next=18 rot=Some(23)",
            "edge=21 twin=20 cell=4 v0=Some(4) v1=Some(5) primary=true secondary=false finite=true next=25 rot=Some(12)",
            "edge=22 twin=23 cell=6 v0=None v1=Some(5) primary=false secondary=true finite=false next=20 rot=Some(16)",
            "edge=23 twin=22 cell=7 v0=Some(5) v1=None primary=false secondary=true finite=false next=24 rot=Some(25)",
            "edge=24 twin=25 cell=7 v0=None v1=Some(5) primary=false secondary=true finite=false next=23 rot=Some(22)",
            "edge=25 twin=24 cell=4 v0=Some(5) v1=None primary=false secondary=true finite=false next=9 rot=Some(20)",
        ]
    );
    assert_eq!(
        incident_cycles,
        vec![
            vec![0, 4],
            vec![1, 7, 10, 2],
            vec![3, 8],
            vec![5, 14, 19, 12, 6],
            vec![9, 11, 13, 21, 25],
            vec![15, 16],
            vec![17, 22, 20, 18],
            vec![23, 24],
        ]
    );
}

#[test]
fn task22o13_disconnected_segments_pin_point_point_face_and_rotation_order() {
    let output = diagram::build(&[
        Line::new(Point::new(0, 0), Point::new(10, 0)),
        Line::new(Point::new(30, 10), Point::new(40, 10)),
    ])
    .unwrap();
    let point_point_edges = (0..output.num_edges())
        .filter_map(|index| {
            let edge = diagram::edge_index(&output, index);
            let twin = output.edge_get_twin(edge).unwrap();
            let left = output.cell(output.edge_get_cell(edge).unwrap()).unwrap();
            let right = output.cell(output.edge_get_cell(twin).unwrap()).unwrap();
            (left.contains_point() && right.contains_point()).then(|| {
                (
                    edge.u32(),
                    twin.u32(),
                    output.edge_get_next(edge).unwrap().u32(),
                    output.edge_rot_next(edge).map(|next| next.u32()),
                )
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(
        point_point_edges,
        [
            (4, 5, 3, Some(15)),
            (5, 4, 6, Some(9)),
            (12, 13, 8, Some(7)),
            (13, 12, 0, Some(1)),
            (16, 17, 14, Some(2)),
            (17, 16, 11, Some(10)),
        ]
    );
}
