use crate::geometry::{
    Line, Point,
    medial_axis::{
        annotate::{self, VertexCategory, vertex_equal_to_point},
        diagram, edge_is_eligible,
    },
};

fn eligible_primary_edges(lines: &[Line]) -> Vec<(u32, bool)> {
    let vd = diagram::build(lines).unwrap();
    let annotations = annotate::annotate(&vd, lines).unwrap();
    (0..vd.num_edges())
        .step_by(2)
        .filter_map(|index| {
            let edge = diagram::edge_index(&vd, index);
            (vd.edge(edge).unwrap().is_primary() && vd.edge_is_finite(edge).unwrap()).then(|| {
                (
                    edge.u32(),
                    edge_is_eligible(&vd, edge, &annotations).unwrap(),
                )
            })
        })
        .collect()
}

#[test]
fn task22o13_contour_and_hole_eligible_edge_decisions_are_literal() {
    let contour = [
        Line::new(Point::new(0, 0), Point::new(20, 0)),
        Line::new(Point::new(20, 0), Point::new(20, 10)),
        Line::new(Point::new(20, 10), Point::new(0, 10)),
        Line::new(Point::new(0, 10), Point::new(0, 0)),
    ];
    assert_eq!(
        eligible_primary_edges(&contour),
        [(6, true), (10, true), (12, true), (18, true), (20, true)]
    );

    let hole = [
        Line::new(Point::new(0, 0), Point::new(40, 0)),
        Line::new(Point::new(40, 0), Point::new(40, 40)),
        Line::new(Point::new(40, 40), Point::new(0, 40)),
        Line::new(Point::new(0, 40), Point::new(0, 0)),
        Line::new(Point::new(10, 10), Point::new(10, 30)),
        Line::new(Point::new(10, 30), Point::new(30, 30)),
        Line::new(Point::new(30, 30), Point::new(30, 10)),
        Line::new(Point::new(30, 10), Point::new(10, 10)),
    ];
    assert_eq!(
        eligible_primary_edges(&hole),
        [
            (6, true),
            (10, true),
            (12, true),
            (16, true),
            (20, true),
            (24, false),
            (28, false),
            (30, true),
            (32, true),
            (34, true),
            (36, true),
            (42, false),
            (44, false),
            (50, true),
            (52, true),
            (58, true),
            (60, true),
            (62, true),
            (64, true),
            (66, true),
        ]
    );
}

#[test]
fn task22o13_ulp_boundary_flows_into_production_edge_eligibility() {
    let lines = [
        Line::new(Point::new(0, 0), Point::new(20, 0)),
        Line::new(Point::new(20, 0), Point::new(20, 10)),
        Line::new(Point::new(20, 10), Point::new(0, 10)),
        Line::new(Point::new(0, 10), Point::new(0, 0)),
    ];
    let vd = diagram::build(&lines).unwrap();
    let edge = diagram::edge_index(&vd, 12);
    let site = 12.0_f64;
    for (ulps, expected_category, expected_eligible) in [
        (64, VertexCategory::OnContour, false),
        (65, VertexCategory::Inside, true),
    ] {
        let coordinate = f64::from_bits(site.to_bits() + ulps);
        let category = if vertex_equal_to_point(site, coordinate) {
            VertexCategory::OnContour
        } else {
            VertexCategory::Inside
        };
        assert_eq!(category, expected_category);
        let mut annotations = annotate::annotate(&vd, &lines).unwrap();
        for vertex in [
            vd.edge_get_vertex0(edge).unwrap().unwrap(),
            vd.edge_get_vertex1(edge).unwrap().unwrap(),
        ] {
            annotations.vertices[vertex.usize()] = category;
        }
        assert_eq!(
            edge_is_eligible(&vd, edge, &annotations).unwrap(),
            expected_eligible
        );
    }
}
