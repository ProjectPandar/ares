//! `TreeSupport3D.cpp::triangulate_fan/triangulate_strip`.

use super::{IndexedTriangleSet, Strip, vector::distance_squared};

pub(super) fn triangulate_fan<const FLIP: bool>(
    mesh: &mut IndexedTriangleSet,
    fan: usize,
    (begin, end): Strip,
) {
    mesh.indices.reserve(end - begin);
    let mut previous = end - 1;
    for current in begin..end {
        mesh.indices.push(if FLIP {
            [fan as u32, previous as u32, current as u32]
        } else {
            [fan as u32, current as u32, previous as u32]
        });
        previous = current;
    }
}

pub(super) fn triangulate_strip(
    mesh: &mut IndexedTriangleSet,
    (begin1, end1): Strip,
    (begin2, end2): Strip,
) {
    let mut count1 = end1 - begin1;
    let mut count2 = end2 - begin2;
    mesh.indices.reserve(count1 + count2);
    let mut second = begin2;
    let mut closest = f32::MAX;
    for index in begin2..end2 {
        let distance = distance_squared(mesh.vertices[index], mesh.vertices[begin1]);
        if distance < closest {
            closest = distance;
            second = index;
        }
    }
    let mut first = begin1;
    while count1 > 0 || count2 > 0 {
        let next_first = if first + 1 == end1 { begin1 } else { first + 1 };
        let next_second = if second + 1 == end2 {
            begin2
        } else {
            second + 1
        };
        let take_first = if count1 == 0 {
            false
        } else if count2 == 0 {
            true
        } else {
            distance_squared(mesh.vertices[next_first], mesh.vertices[second])
                < distance_squared(mesh.vertices[next_second], mesh.vertices[first])
        };
        if take_first {
            mesh.indices
                .push([first as u32, next_first as u32, second as u32]);
            count1 -= 1;
            first = next_first;
        } else {
            mesh.indices
                .push([first as u32, next_second as u32, second as u32]);
            count2 -= 1;
            second = next_second;
        }
    }
}
