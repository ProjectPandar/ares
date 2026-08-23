// GCC libstdc++ `std::sort` introsort control-flow port (sort.cc) used as the
// fixed comparator-compatibility target for the GCC-built OrcaSlicer oracle.
// Structure: __introsort_loop (median-of-three pivot moved to front, unguarded
// partition, depth-limited heapsort fallback), then __final_insertion_sort.

const INSERTION_SORT_THRESHOLD: usize = 16;

pub(crate) fn fixed_gcc_sort_by<T>(items: &mut [T], mut less: impl FnMut(&T, &T) -> bool) {
    let len = items.len();
    if len < 2 {
        return;
    }
    introsort_loop(items, &mut less, 2 * len.ilog2() as isize);
    final_insertion_sort(items, &mut less);
}

fn introsort_loop<T>(
    mut items: &mut [T],
    less: &mut impl FnMut(&T, &T) -> bool,
    depth_limit: isize,
) {
    let mut depth_limit = depth_limit;
    while items.len() > INSERTION_SORT_THRESHOLD {
        if depth_limit == 0 {
            heap_sort(items, less);
            return;
        }
        depth_limit -= 1;
        // __unguarded_partition_pivot: move the median of
        // (first+1, mid, last-1) to first, then partition from first+1.
        let mid = items.len() / 2;
        move_median_to_front(items, 0, 1, mid, items.len() - 1, less);
        let cut = unguarded_partition(items, less);
        // libstdc++ recurses on the right half [cut, last) and loops on the left.
        let (left, right) = items.split_at_mut(cut);
        introsort_loop(right, less, depth_limit);
        items = left;
    }
}

fn move_median_to_front<T>(
    items: &mut [T],
    result: usize,
    a: usize,
    b: usize,
    c: usize,
    less: &mut impl FnMut(&T, &T) -> bool,
) {
    if less(&items[a], &items[b]) {
        if less(&items[b], &items[c]) {
            items.swap(result, b);
        } else if less(&items[a], &items[c]) {
            items.swap(result, c);
        } else {
            items.swap(result, a);
        }
    } else if less(&items[a], &items[c]) {
        items.swap(result, a);
    } else if less(&items[b], &items[c]) {
        items.swap(result, c);
    } else {
        items.swap(result, b);
    }
}

fn unguarded_partition<T>(items: &mut [T], less: &mut impl FnMut(&T, &T) -> bool) -> usize {
    // Pivot at index 0; first_pivot on one side, last on the other.
    let len = items.len();
    let mut first = 1;
    let mut last = len;
    loop {
        while less(&items[first], &items[0]) {
            first += 1;
        }
        last -= 1;
        while less(&items[0], &items[last]) {
            last -= 1;
        }
        if first >= last {
            return first;
        }
        items.swap(first, last);
        first += 1;
    }
}

fn final_insertion_sort<T>(items: &mut [T], less: &mut impl FnMut(&T, &T) -> bool) {
    if items.len() > INSERTION_SORT_THRESHOLD {
        insertion_sort(&mut items[..INSERTION_SORT_THRESHOLD], less);
        for index in INSERTION_SORT_THRESHOLD..items.len() {
            let mut index = index;
            while index > 0 && less(&items[index], &items[index - 1]) {
                items.swap(index, index - 1);
                index -= 1;
            }
        }
    } else {
        insertion_sort(items, less);
    }
}

fn insertion_sort<T>(items: &mut [T], less: &mut impl FnMut(&T, &T) -> bool) {
    for index in 1..items.len() {
        let mut index = index;
        while index > 0 && less(&items[index], &items[index - 1]) {
            items.swap(index, index - 1);
            index -= 1;
        }
    }
}

fn heap_sort<T>(items: &mut [T], less: &mut impl FnMut(&T, &T) -> bool) {
    let len = items.len();
    for start in (0..len / 2).rev() {
        sift_down(items, start, len, less);
    }
    for end in (1..len).rev() {
        items.swap(0, end);
        sift_down(items, 0, end, less);
    }
}

fn sift_down<T>(items: &mut [T], start: usize, end: usize, less: &mut impl FnMut(&T, &T) -> bool) {
    let mut root = start;
    loop {
        let mut child = 2 * root + 1;
        if child >= end {
            return;
        }
        if child + 1 < end && less(&items[child], &items[child + 1]) {
            child += 1;
        }
        if !less(&items[root], &items[child]) {
            return;
        }
        items.swap(root, child);
        root = child;
    }
}

#[cfg(test)]
mod tests {
    use super::fixed_gcc_sort_by;

    #[test]
    fn sorts_integers_stably_enough_for_equal_keys() {
        let mut values = vec![5, 1, 5, 2, 1, 9, 5, 2];
        fixed_gcc_sort_by(&mut values, |a, b| a < b);
        assert_eq!(values, vec![1, 1, 2, 2, 5, 5, 5, 9]);
    }

    #[test]
    fn preserves_libstdcxx_equal_key_permutation() {
        let mut values = (0..20).map(|id| (0, id)).collect::<Vec<_>>();
        fixed_gcc_sort_by(&mut values, |left, right| left.0 < right.0);
        assert_eq!(
            values.into_iter().map(|(_, id)| id).collect::<Vec<_>>(),
            vec![
                10, 19, 18, 17, 16, 15, 14, 13, 12, 11, 0, 9, 8, 7, 6, 5, 4, 3, 2, 1
            ]
        );
    }

    #[test]
    fn preserves_libstdcxx_grouped_key_permutation() {
        let keys = [
            2, 1, 2, 1, 0, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1,
            2, 0, 1, 2,
        ];
        let mut values = keys
            .into_iter()
            .enumerate()
            .map(|(id, key)| (key, id))
            .collect::<Vec<_>>();
        fixed_gcc_sort_by(&mut values, |left, right| left.0 < right.0);
        assert_eq!(
            values.into_iter().map(|(_, id)| id).collect::<Vec<_>>(),
            vec![
                30, 4, 27, 6, 24, 9, 21, 12, 18, 15, 1, 31, 28, 25, 22, 19, 13, 10, 3, 16, 7, 8,
                32, 0, 2, 29, 5, 26, 23, 11, 20, 14, 17,
            ]
        );
    }

    #[test]
    fn empty_and_single_element_slices_are_noops() {
        let mut empty: Vec<i32> = Vec::new();
        fixed_gcc_sort_by(&mut empty, |a, b| a < b);
        let mut single = vec![7];
        fixed_gcc_sort_by(&mut single, |a, b| a < b);
        assert_eq!(single, vec![7]);
    }

    #[test]
    fn large_slices_cover_heapsort_and_final_insertion_paths() {
        let mut values: Vec<u32> = (0..200).map(|index| (index * 7919) % 101).collect();
        fixed_gcc_sort_by(&mut values, |a, b| a < b);
        let mut expected = values.clone();
        expected.sort_unstable();
        assert_eq!(values, expected);
    }
}
