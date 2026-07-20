use crate::geometry::chain_points::{EndPoint, priority_queue::EndPointHeap};

#[test]
fn task22m_chain_points_priority_queue_equal_keys_rise() {
    let mut points = endpoints(&[4.0, 2.0, 2.0, 3.0, 2.0]);
    let mut queue = EndPointHeap::with_capacity(points.len());
    let expected = [
        (vec![0], 0),
        (vec![1, 0], 1),
        (vec![2, 0, 1], 2),
        (vec![2, 3, 1, 0], 2),
        (vec![4, 2, 1, 0, 3], 4),
    ];

    for (point, (heap, top)) in expected.into_iter().enumerate() {
        queue.push(point, &mut points);
        assert_eq!(queue.heap(), heap);
        assert_eq!(queue.top(), top);
    }
    assert_eq!(heap_indices(&points), [3, 2, 1, 4, 0]);
}

#[test]
fn task22m_chain_points_priority_queue_pop_chooses_equal_right_child() {
    let mut points = endpoints(&[1.0, 2.0, 2.0, 9.0]);
    let mut queue = filled_queue(&mut points);
    assert_eq!(queue.heap(), [0, 1, 2, 3]);

    queue.pop(&mut points);

    assert_eq!(queue.heap(), [2, 1, 3]);
    assert_eq!(queue.top(), 2);
    assert_eq!(heap_indices(&points), [0, 1, 0, 2]);
}

#[test]
fn task22m_chain_points_priority_queue_removes_last_and_nonlast() {
    let distances = [1.0, 10.0, 2.0, 11.0, 12.0, 3.0, 4.0];
    let mut points = endpoints(&distances);
    let mut queue = filled_queue(&mut points);
    assert_eq!(queue.heap(), [0, 1, 2, 3, 4, 5, 6]);
    queue.remove(6, &mut points);
    assert_eq!(queue.heap(), [0, 1, 2, 3, 4, 5]);
    assert_eq!(points[6].heap_index_for_test(), 6);

    let mut points = endpoints(&distances);
    let mut queue = filled_queue(&mut points);
    queue.remove(3, &mut points);
    assert_eq!(queue.heap(), [0, 6, 2, 1, 4, 5]);
    assert_eq!(heap_indices(&points), [0, 3, 2, 3, 4, 5, 1]);
}

#[test]
fn task22m_chain_points_priority_queue_updates_decrease_and_increase() {
    let mut points = endpoints(&[1.0, 4.0, 2.0, 5.0, 6.0]);
    let mut queue = filled_queue(&mut points);
    points[4].set_distance_for_test(0.5);
    queue.update(4, &mut points);
    assert_eq!(queue.heap(), [4, 0, 2, 3, 1]);
    assert_eq!(heap_indices(&points), [1, 4, 2, 3, 0]);

    points[4].set_distance_for_test(10.0);
    queue.update(0, &mut points);
    assert_eq!(queue.heap(), [0, 1, 2, 3, 4]);
    assert_eq!(heap_indices(&points), [0, 1, 2, 3, 4]);
}

fn endpoints(distances: &[f64]) -> Vec<EndPoint> {
    distances
        .iter()
        .copied()
        .map(EndPoint::for_queue_test)
        .collect()
}

fn filled_queue(points: &mut [EndPoint]) -> EndPointHeap {
    let mut queue = EndPointHeap::with_capacity(points.len());
    for point in 0..points.len() {
        queue.push(point, points);
    }
    queue
}

fn heap_indices(points: &[EndPoint]) -> Vec<usize> {
    points.iter().map(EndPoint::heap_index_for_test).collect()
}
