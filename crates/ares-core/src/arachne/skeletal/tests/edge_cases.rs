use crate::arachne::skeletal::SkeletalGraph;

use super::{edge_pair, node};

#[test]
fn task22o100_recursive_equal_distance_walk_finds_later_upward_edge() {
    let mut graph = SkeletalGraph::default();
    let left = node(&mut graph, 0, 0, 5);
    let equal = node(&mut graph, 10, 0, 5);
    let high = node(&mut graph, 10, 10, 7);
    let (edge, twin) = edge_pair(&mut graph, left, equal);
    let (up, up_twin) = edge_pair(&mut graph, equal, high);
    graph.edge_mut(edge).next = Some(up);
    graph.edge_mut(up_twin).next = Some(twin);

    assert!(graph.edge_can_go_up(edge, false));
    assert!(!graph.edge_can_go_up(edge, true));
    assert_eq!(graph.edge_dist_to_go_up(edge), Some(10));
    assert!(graph.edge_is_upward(edge));
}

#[test]
fn task22o100_one_short_side_does_not_collapse_a_partial_cell() {
    let mut graph = SkeletalGraph::default();
    let a = node(&mut graph, 0, 0, 0);
    let b = node(&mut graph, 0, 10, 2);
    let c = node(&mut graph, 100, 10, 2);
    let d = node(&mut graph, 1, 0, 0);
    let (start, start_twin) = edge_pair(&mut graph, a, b);
    let (end, end_twin) = edge_pair(&mut graph, c, d);
    graph.edge_mut(start).next = Some(end);
    graph.edge_mut(end).prev = Some(start);

    graph.collapse_small_edges(2);

    for edge in [start, start_twin, end, end_twin] {
        assert!(graph.contains_edge(edge));
    }
    for node in [a, b, c, d] {
        assert!(graph.contains_node(node));
    }
}

#[test]
fn task22o100_middle_then_sides_collapse_the_complete_small_cell() {
    let mut graph = SkeletalGraph::default();
    let a = node(&mut graph, 0, 0, 0);
    let b = node(&mut graph, 0, 10, 2);
    let c = node(&mut graph, 1, 10, 2);
    let d = node(&mut graph, 1, 0, 0);
    let (start, start_twin) = edge_pair(&mut graph, a, b);
    let (middle, middle_twin) = edge_pair(&mut graph, b, c);
    let (end, end_twin) = edge_pair(&mut graph, c, d);
    graph.edge_mut(start).next = Some(middle);
    graph.edge_mut(middle).prev = Some(start);
    graph.edge_mut(middle).next = Some(end);
    graph.edge_mut(end).prev = Some(middle);
    graph.edge_mut(end_twin).next = Some(middle_twin);
    graph.edge_mut(middle_twin).prev = Some(end_twin);
    graph.edge_mut(middle_twin).next = Some(start_twin);
    graph.edge_mut(start_twin).prev = Some(middle_twin);
    graph.edge_mut(end_twin).prev = Some(end_twin);

    graph.collapse_small_edges(2);

    for edge in [start, middle, middle_twin, end] {
        assert!(!graph.contains_edge(edge));
    }
    assert_eq!(graph.edge(start_twin).twin, Some(end_twin));
    assert_eq!(graph.edge(end_twin).twin, Some(start_twin));
    assert!(!graph.contains_node(a));
    assert!(!graph.contains_node(c));
}

#[test]
fn task22o100_middle_collapse_stops_after_source_1001_rewires() {
    let mut graph = SkeletalGraph::default();
    let a = node(&mut graph, 0, 0, 0);
    let b = node(&mut graph, 10, 0, 2);
    let c = node(&mut graph, 11, 0, 2);
    let d = node(&mut graph, 10_000, 0, 0);
    let (start, start_twin) = edge_pair(&mut graph, a, b);
    let (middle, middle_twin) = edge_pair(&mut graph, b, c);
    let (end, end_twin) = edge_pair(&mut graph, c, d);
    graph.edge_mut(start).next = Some(middle);
    graph.edge_mut(middle).prev = Some(start);
    graph.edge_mut(middle).next = Some(end);
    graph.edge_mut(end).prev = Some(middle);
    graph.edge_mut(middle_twin).next = Some(start_twin);
    graph.edge_mut(start_twin).prev = Some(middle_twin);

    let mut chain = Vec::new();
    for index in 0..1_002 {
        let target = node(&mut graph, 20_000 + index, 1, 0);
        let pair = edge_pair(&mut graph, c, target);
        graph.edge_mut(pair.0).prev = Some(pair.0);
        graph.edge_mut(pair.1).prev = Some(pair.1);
        chain.push(pair);
    }
    graph.edge_mut(end_twin).prev = Some(end_twin);
    graph.edge_mut(end_twin).next = Some(chain[0].0);
    for pair in chain.windows(2) {
        graph.edge_mut(pair[0].1).next = Some(pair[1].0);
    }
    graph.edge_mut(chain.last().unwrap().1).next = Some(middle_twin);
    graph.edge_mut(middle_twin).prev = Some(chain.last().unwrap().1);

    graph.collapse_small_edges(2);

    assert_eq!(graph.edge(chain[999].0).from, Some(b));
    assert_eq!(graph.edge(chain[999].1).to, Some(b));
    assert_eq!(graph.edge(chain[1_000].0).from, Some(c));
    assert_eq!(graph.edge(chain[1_000].1).to, Some(c));
}
