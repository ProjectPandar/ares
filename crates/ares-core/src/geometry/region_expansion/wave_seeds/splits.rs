use crate::geometry::clipper::ordering::fixed_msvc_sort_by;
use crate::geometry::clipper::z::{KernelPoint, ZPath};

use super::WaveSeed;

#[derive(Clone, Copy)]
struct Split {
    point: KernelPoint,
    path: Option<usize>,
}

pub(super) fn split_registry(paths: &[ZPath]) -> Vec<(KernelPoint, Option<usize>)> {
    let mut splits = paths
        .iter()
        .map(|path| {
            debug_assert!(path.len() >= 2);
            debug_assert!(path.first().unwrap().full_eq(*path.last().unwrap()));
            Split {
                point: path[0],
                path: None,
            }
        })
        .collect::<Vec<_>>();
    fixed_msvc_sort_by(&mut splits, |left, right| {
        left.point.full_cmp(right.point).is_lt()
    });
    splits
        .into_iter()
        .map(|split| (split.point, split.path))
        .collect()
}

pub(super) fn merge_splits(paths: &mut Vec<ZPath>, records: &mut [(KernelPoint, Option<usize>)]) {
    let mut index = 0;
    while index < paths.len() {
        debug_assert!(paths[index].len() >= 2);
        let front = paths[index][0];
        let back = *paths[index].last().unwrap();
        if front == back {
            index += 1;
            continue;
        }
        let found = find_end(records, front)
            .map(|record| (record, true))
            .or_else(|| find_end(records, back).map(|record| (record, false)));
        let Some((record, source_front)) = found else {
            index += 1;
            continue;
        };
        let Some(destination) = records[record].1 else {
            records[record].1 = Some(index);
            index += 1;
            continue;
        };

        let split = records[record].0;
        let mut source = std::mem::take(&mut paths[index]);
        let destination_front = paths[destination][0] == split;
        merge_path(
            &mut paths[destination],
            destination_front,
            &mut source,
            source_front,
        );
        if index + 1 == paths.len() {
            paths.pop();
            break;
        }
        let last = paths.pop().unwrap();
        paths[index] = last;
    }
}

fn find_end(records: &[(KernelPoint, Option<usize>)], point: KernelPoint) -> Option<usize> {
    let mut first = 0;
    let mut count = records.len();
    while count > 0 {
        let step = count / 2;
        let middle = first + step;
        if records[middle].0.full_cmp(point).is_lt() {
            first = middle + 1;
            count -= step + 1;
        } else {
            count = step;
        }
    }
    (first < records.len() && records[first].0 == point).then_some(first)
}

fn merge_path(
    destination: &mut ZPath,
    destination_front: bool,
    source: &mut ZPath,
    source_front: bool,
) {
    if destination_front {
        if source_front {
            destination.reverse();
        } else {
            std::mem::swap(destination, source);
        }
    } else if !source_front {
        source.reverse();
    }
    destination.append(source);
}

pub(super) fn sort_seeds(seeds: &mut Vec<WaveSeed>) {
    let mut order = (0..seeds.len()).collect::<Vec<_>>();
    fixed_msvc_sort_by(&mut order, |left, right| {
        let left = &seeds[*left];
        let right = &seeds[*right];
        left.boundary < right.boundary || (left.boundary == right.boundary && left.src < right.src)
    });
    let mut source = std::mem::take(seeds)
        .into_iter()
        .map(Some)
        .collect::<Vec<_>>();
    seeds.extend(
        order
            .into_iter()
            .map(|index| source[index].take().expect("sort permutation is unique")),
    );
}

#[cfg(test)]
pub(in crate::geometry) fn sort_seeds_for_test(seeds: &mut Vec<WaveSeed>) {
    sort_seeds(seeds);
}

#[cfg(test)]
pub(in crate::geometry) fn reconcile_for_test(source_paths: &[ZPath], paths: &mut Vec<ZPath>) {
    let mut records = split_registry(source_paths);
    merge_splits(paths, &mut records);
}

#[cfg(test)]
pub(in crate::geometry) fn merge_path_for_test(
    destination: &mut ZPath,
    destination_front: bool,
    source: ZPath,
    source_front: bool,
) {
    let mut source = source;
    merge_path(destination, destination_front, &mut source, source_front);
}

#[cfg(test)]
pub(in crate::geometry) fn split_registry_for_test(
    source_paths: &[ZPath],
) -> Vec<(KernelPoint, Option<usize>)> {
    split_registry(source_paths)
}
