use super::{
    IntersectionKind, MonotonicRegion, SegmentedLine,
    regions::{overlap_left, overlap_right, vertical_run_top},
};

pub(crate) fn connect_region_neighbors(regions: &mut [MonotonicRegion], lines: &[SegmentedLine]) {
    for region in &mut *regions {
        region.left_neighbors.clear();
        region.right_neighbors.clear();
    }

    let mut starts = regions
        .iter()
        .enumerate()
        .map(|(index, region)| ((region.left.line, region.left.low), index))
        .collect::<Vec<_>>();
    let mut ends = regions
        .iter()
        .enumerate()
        .map(|(index, region)| ((region.right.line, region.right.low), index))
        .collect::<Vec<_>>();
    starts.sort_unstable_by_key(|entry| entry.0);
    ends.sort_unstable_by_key(|entry| entry.0);

    let mut links = Vec::new();
    for (index, region) in regions.iter().enumerate() {
        if let Some(left_line) = region.left.line.checked_sub(1)
            && let Some(overlap) = overlap_left(
                &lines[region.left.line],
                &lines[left_line],
                (region.left.low, region.left.high),
            )
        {
            for_each_overlapping_region(&ends, left_line, &lines[left_line], overlap, |neighbor| {
                links.push((neighbor, index))
            });
        }

        let right_line = region.right.line + 1;
        if right_line < lines.len()
            && let Some(overlap) = overlap_right(
                &lines[region.right.line],
                &lines[right_line],
                (region.right.low, region.right.high),
            )
        {
            for_each_overlapping_region(
                &starts,
                right_line,
                &lines[right_line],
                overlap,
                |neighbor| links.push((index, neighbor)),
            );
        }
    }

    links.sort_unstable();
    links.dedup();
    for (left, right) in links {
        regions[left].right_neighbors.push(right);
        regions[right].left_neighbors.push(left);
    }
}

fn for_each_overlapping_region(
    map: &[((usize, usize), usize)],
    line_index: usize,
    line: &SegmentedLine,
    (mut begin, end): (usize, usize),
    mut visit: impl FnMut(usize),
) {
    loop {
        visit(mapped_region(map, (line_index, begin)));
        let top = vertical_run_top(line, begin);
        if top == end {
            break;
        }
        begin = next_inner_low(line, top);
    }
}

fn mapped_region(map: &[((usize, usize), usize)], boundary: (usize, usize)) -> usize {
    let position = map
        .binary_search_by_key(&boundary, |entry| entry.0)
        .expect("overlap begins at a region boundary");
    map[position].1
}

fn next_inner_low(line: &SegmentedLine, top: usize) -> usize {
    top + 1
        + line.intersections[top + 1..]
            .iter()
            .position(|intersection| intersection.kind == IntersectionKind::InnerLow)
            .expect("overlap contains another vertical run")
}
