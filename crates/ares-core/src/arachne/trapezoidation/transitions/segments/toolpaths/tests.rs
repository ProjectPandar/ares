use crate::{
    arachne::{extrusion_line::ExtrusionJunction, skeletal::SkeletalGraph},
    geometry::{CoordinateScale, Point},
};

use super::SegmentConditions;

use super::super::super::{
    SkeletalTrapezoidation,
    test_support::{config, strategy},
};

#[test]
fn task22o184_creates_and_extends_compatible_toolpath_segment() {
    let scale = CoordinateScale::Normal;
    let strategy = strategy(scale);
    let mut trapezoidation = SkeletalTrapezoidation {
        graph: SkeletalGraph::default(),
        beading_strategy: strategy.as_ref(),
        config: config(scale),
        vd_edge_to_he_edge: Default::default(),
        vd_node_to_he_node: Default::default(),
        transition_storage: Vec::new(),
        transition_end_storage: Vec::new(),
        beading_storage: Vec::new(),
        extrusion_junction_storage: Vec::new(),
        generated_toolpaths: Vec::new(),
    };
    let first = ExtrusionJunction::new(Point::new(0, 0), 100, 1);
    let middle = ExtrusionJunction::new(Point::new(100, 0), 100, 1);
    let nearby_middle = ExtrusionJunction::new(Point::new(105, 0), 105, 1);
    let last = ExtrusionJunction::new(Point::new(200, 0), 105, 1);
    let conditions = SegmentConditions {
        is_odd: false,
        force_new_path: false,
        from_is_three_way: false,
        to_is_three_way: false,
    };

    trapezoidation.add_toolpath_segment(first, middle, conditions);
    trapezoidation.add_toolpath_segment(nearby_middle, last, conditions);

    assert_eq!(trapezoidation.generated_toolpaths.len(), 2);
    assert_eq!(trapezoidation.generated_toolpaths[1].len(), 1);
    assert_eq!(
        trapezoidation.generated_toolpaths[1][0].junctions,
        vec![first, middle, last]
    );
}
