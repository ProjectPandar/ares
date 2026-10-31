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
        move_median_to_front(items, 0, (1, mid, items.len() - 1), less);
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
    (a, b, c): (usize, usize, usize),
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
    if len < 2 {
        return;
    }

    for parent in (0..=(len - 2) / 2).rev() {
        adjust_heap(items, parent, len, less);
    }
    for end in (1..len).rev() {
        items.swap(0, end);
        adjust_heap(&mut items[..end], 0, end, less);
    }
}

fn adjust_heap<T>(items: &mut [T], hole: usize, len: usize, less: &mut impl FnMut(&T, &T) -> bool) {
    // libstdc++ moves a hole down, then inserts the saved value back up.
    // Record that path and rotate it to preserve the same equal-key permutation.
    let mut path = [0usize; usize::BITS as usize];
    let mut path_len = 1;
    path[0] = hole;

    let mut second_child = hole;
    while second_child < (len - 1) / 2 {
        second_child = 2 * (second_child + 1);
        if less(&items[second_child], &items[second_child - 1]) {
            second_child -= 1;
        }
        path[path_len] = second_child;
        path_len += 1;
    }
    if len & 1 == 0 && second_child == (len - 2) / 2 {
        second_child = 2 * (second_child + 1) - 1;
        path[path_len] = second_child;
        path_len += 1;
    }

    let mut insertion = path_len - 1;
    while insertion > 0 && less(&items[path[insertion]], &items[path[0]]) {
        insertion -= 1;
    }
    for index in 0..insertion {
        items.swap(path[index], path[index + 1]);
    }
}
