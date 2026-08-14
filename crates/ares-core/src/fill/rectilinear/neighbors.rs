use super::{LinkType, MonotonicRegion, SegmentedLine};

pub(crate) fn connect_region_neighbors(regions: &mut [MonotonicRegion], lines: &[SegmentedLine]) {
    for region in &mut *regions {
        region.left_neighbors.clear();
        region.right_neighbors.clear();
    }

    let mut links = Vec::new();
    for (left_index, left) in regions.iter().enumerate() {
        for (right_index, right) in regions.iter().enumerate() {
            if left.right.line + 1 != right.left.line {
                continue;
            }
            if boundaries_overlap(left, right, lines) {
                links.push((left_index, right_index));
            }
        }
    }

    for (left, right) in links {
        regions[left].right_neighbors.push(right);
        regions[right].left_neighbors.push(left);
    }
    for region in regions {
        region.left_neighbors.sort_unstable();
        region.left_neighbors.dedup();
        region.right_neighbors.sort_unstable();
        region.right_neighbors.dedup();
    }
}

fn boundaries_overlap(
    left: &MonotonicRegion,
    right: &MonotonicRegion,
    lines: &[SegmentedLine],
) -> bool {
    let left_line = &lines[left.right.line];
    let right_line = &lines[right.left.line];
    let forward = (left.right.low..=left.right.high).any(|index| {
        left_line.intersections[index]
            .next
            .is_some_and(|(target, kind, _)| {
                kind == LinkType::Horizontal && (right.left.low..=right.left.high).contains(&target)
            })
    });
    let backward = (right.left.low..=right.left.high).any(|index| {
        right_line.intersections[index]
            .previous
            .is_some_and(|(target, kind, _)| {
                kind == LinkType::Horizontal && (left.right.low..=left.right.high).contains(&target)
            })
    });
    forward || backward
}
