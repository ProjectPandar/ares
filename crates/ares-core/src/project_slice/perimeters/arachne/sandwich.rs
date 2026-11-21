//! Inner-outer-inner sandwich reordering for the arachne generator
//! (`PerimeterGenerator.cpp:2999-3094`): after the topological sort,
//! bring inset-0 contours to the front, reorder by proximity, then
//! resequence each island as [deep internals (innermost first),
//! external, first internal].

use super::traverse::PerimeterGeneratorArachneExtrusion;
use crate::arachne::{ExtrusionJunction, ExtrusionLine};

pub(in crate::project_slice) fn apply_inner_outer_inner(
    mut ordered: Vec<PerimeterGeneratorArachneExtrusion>,
    threshold_external: i64,
    threshold_internal: i64,
) -> Vec<PerimeterGeneratorArachneExtrusion> {
    // `bringContoursToFront` (`PerimeterGenerator.cpp:2634-2638`): a
    // stable partition moving external-perimeter contours to the front.
    ordered.sort_by_key(|item| !(item.is_contour && item.extrusion.inset_index == 0));

    // `reorderPerimetersByProximity` (`:3028`, `:2523-2623`).
    let mut result =
        reorder_perimeters_by_proximity(&ordered, threshold_external, threshold_internal);
    // `reordered_extrusions = ordered_extrusions` — the sandwich reads the
    // copy and writes the resequenced blocks back into the base list.
    let reordered = result.clone();

    // Sandwich scan (`:3035-3092`): `position`, `arr_i`, `arr_j` are the
    // upstream loop indices; `outer`/`first_internal`/`second_internal`
    // use the -1 sentinel, so the scan arithmetic runs in i64.
    let size = reordered.len() as i64;
    let mut position: i64 = 0;
    while position < size {
        let mut outer: i64 = -1;
        let mut first_internal: i64 = -1;
        let mut second_internal: i64 = -1;
        let mut current_perimeter: i64 = -1;
        let mut max_internal: i64 = size - 1;
        let mut arr_i: i64 = position;
        while arr_i < size {
            let inset = reordered[arr_i as usize].extrusion.inset_index;
            match inset {
                0 if outer == -1 => {
                    outer = arr_i;
                }
                1 if first_internal == -1 && arr_i > outer && outer != -1 => {
                    first_internal = arr_i;
                }
                2 if second_internal == -1 && arr_i > first_internal && outer != -1 => {
                    second_internal = arr_i;
                }
                _ => {}
            }
            // A new external perimeter after the first internal starts a
            // new island (`:3060-3065`): step back one perimeter and bound
            // the resequencing block there.
            if outer > -1
                && first_internal > -1
                && reordered[arr_i as usize].extrusion.inset_index == 0
            {
                arr_i -= 1;
                max_internal = arr_i;
                break;
            }
            arr_i += 1;
        }
        if !(outer > -1 && first_internal > -1 && second_internal > -1) {
            position = arr_i + 1;
            continue;
        }
        let mut inner_outer: Vec<Option<PerimeterGeneratorArachneExtrusion>> =
            vec![None; (max_internal - position + 1) as usize];
        // Inside out: the deep internals from `max_internal` down to
        // `second_internal` fill the front of the block (`:3074-3079`).
        let mut arr_j: i64 = max_internal;
        while arr_j >= position {
            if arr_j >= second_internal {
                inner_outer[(max_internal - arr_j) as usize] =
                    Some(reordered[arr_j as usize].clone());
                current_perimeter += 1;
            }
            arr_j -= 1;
        }
        // Outside in: the external and first-internal walls keep their
        // order after the deep internals (`:3082-3085`).
        arr_j = position;
        while arr_j < second_internal {
            current_perimeter += 1;
            inner_outer[current_perimeter as usize] = Some(reordered[arr_j as usize].clone());
            arr_j += 1;
        }
        arr_j = position;
        while arr_j <= max_internal {
            result[arr_j as usize] = inner_outer[(arr_j - position) as usize]
                .clone()
                .expect("sandwich block fully assigned");
            arr_j += 1;
        }
        position = arr_i + 1;
    }
    result
}

/// `reorderPerimetersByProximity` (`PerimeterGenerator.cpp:2523-2623`).
fn reorder_perimeters_by_proximity(
    entities: &[PerimeterGeneratorArachneExtrusion],
    threshold_external: i64,
    threshold_internal: i64,
) -> Vec<PerimeterGeneratorArachneExtrusion> {
    let mut reordered = Vec::with_capacity(entities.len());
    let mut included = vec![false; entities.len()];

    for reference_index in 0..entities.len() {
        if entities[reference_index].extrusion.inset_index != 0 || included[reference_index] {
            continue;
        }
        let (outer_indices, mut first_level_touching) =
            outer_group(entities, &included, reference_index, threshold_external);
        if !first_level_touching.is_empty() {
            let max_position = largest_perimeter_position(entities, &first_level_touching);
            let last = first_level_touching.len() - 1;
            first_level_touching.swap(max_position, last);
        }
        append_unincluded(entities, &outer_indices, &mut included, &mut reordered);
        append_unincluded(
            entities,
            &first_level_touching,
            &mut included,
            &mut reordered,
        );

        let mut current_inset_index = 2;
        while !first_level_touching.is_empty() {
            let mut current_level_touching = find_all_touching_perimeters(
                entities,
                &first_level_touching,
                threshold_internal,
                current_inset_index,
            );
            current_level_touching.retain(|&index| !included[index]);
            if current_level_touching.is_empty() {
                break;
            }
            let max_position = largest_perimeter_position(entities, &current_level_touching);
            current_level_touching.swap(max_position, 0);
            append_unincluded(
                entities,
                &current_level_touching,
                &mut included,
                &mut reordered,
            );
            first_level_touching = current_level_touching;
            current_inset_index += 1;
        }
    }
    for (index, entity) in entities.iter().enumerate() {
        if !included[index] {
            reordered.push(entity.clone());
        }
    }
    reordered
}

// Follow shared first-inner walls to the other outer walls, including those
// reached through newly added outers (`PerimeterGenerator.cpp:2528-2547`).
fn outer_group(
    entities: &[PerimeterGeneratorArachneExtrusion],
    included: &[bool],
    seed: usize,
    threshold: i64,
) -> (Vec<usize>, Vec<usize>) {
    let mut outers = vec![seed];
    let mut first_level = Vec::new();
    let mut next = 0;
    while next < outers.len() {
        for neighbor in find_all_touching_perimeters(entities, &[outers[next]], threshold, 1) {
            if first_level.contains(&neighbor) {
                continue;
            }
            first_level.push(neighbor);
            extend_outer_group(
                &mut outers,
                entities,
                included,
                &entities[neighbor].extrusion,
                threshold as f64,
            );
        }
        next += 1;
    }
    outers.sort_by_key(|&index| {
        let outer = &entities[index];
        !outer.extrusion.is_closed || outer.is_contour
    });
    (outers, first_level)
}

fn extend_outer_group(
    outers: &mut Vec<usize>,
    entities: &[PerimeterGeneratorArachneExtrusion],
    included: &[bool],
    neighbor: &ExtrusionLine,
    threshold: f64,
) {
    for (index, outer) in entities.iter().enumerate() {
        if outer.extrusion.inset_index == 0
            && !included[index]
            && !outers.contains(&index)
            && lines_touch(&outer.extrusion, neighbor, threshold)
        {
            outers.push(index);
        }
    }
}

fn append_unincluded(
    entities: &[PerimeterGeneratorArachneExtrusion],
    touching: &[usize],
    included: &mut [bool],
    reordered: &mut Vec<PerimeterGeneratorArachneExtrusion>,
) {
    for &index in touching {
        if !included[index] {
            reordered.push(entities[index].clone());
            included[index] = true;
        }
    }
}

fn largest_perimeter_position(
    entities: &[PerimeterGeneratorArachneExtrusion],
    touching: &[usize],
) -> usize {
    let mut max_position = 0;
    for (position, index) in touching.iter().enumerate() {
        if entities[*index].extrusion.length() > entities[touching[max_position]].extrusion.length()
        {
            max_position = position;
        }
    }
    max_position
}

/// `findAllTouchingPerimeters` (`PerimeterGenerator.cpp:2486-2508`).
fn find_all_touching_perimeters(
    entities: &[PerimeterGeneratorArachneExtrusion],
    references: &[usize],
    threshold: i64,
    inset: usize,
) -> Vec<usize> {
    let mut touching = Vec::new();
    let mut seen = vec![false; entities.len()];
    // Native prepends new members in the small, noncolliding sets used by
    // this walk. Do not depend on Rust hash-table iteration order.
    for &reference in references.iter().rev() {
        let line = &entities[reference].extrusion;
        for (index, entity) in entities.iter().enumerate() {
            if !seen[index]
                && entity.extrusion.inset_index == inset
                && lines_touch(line, &entity.extrusion, threshold as f64)
            {
                seen[index] = true;
                touching.push(index);
            }
        }
    }
    touching.reverse();
    touching
}

/// `arachne_lines_touch` (`PerimeterGenerator.cpp:2357-2377`).
fn lines_touch(a: &ExtrusionLine, b: &ExtrusionLine, min_threshold: f64) -> bool {
    let one_way = |from: &ExtrusionLine, to: &ExtrusionLine| {
        from.junctions.iter().any(|&junction| {
            to.junctions
                .windows(2)
                .any(|pair| junction_touches_segment(junction, pair[0], pair[1], min_threshold))
        })
    };
    one_way(a, b) || one_way(b, a)
}

fn junction_touches_segment(
    junction: ExtrusionJunction,
    start: ExtrusionJunction,
    end: ExtrusionJunction,
    min_threshold: f64,
) -> bool {
    let (px, py) = (junction.point.x() as f64, junction.point.y() as f64);
    let (x0, y0) = (start.point.x() as f64, start.point.y() as f64);
    let (dx, dy) = (end.point.x() as f64 - x0, end.point.y() as f64 - y0);
    let length_squared = dx * dx + dy * dy;
    let t = if length_squared > 0.0 {
        (((px - x0) * dx + (py - y0) * dy) / length_squared).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let width = start.width as f64 + t * (end.width - start.width) as f64;
    let threshold = min_threshold.max(0.5 * (junction.width as f64 + width));
    let (x, y) = (x0 + t * dx - px, y0 + t * dy - py);
    (x * x + y * y).sqrt() <= threshold
}
