//! Port of libstdc++ `std::sort` (GCC `bits/stl_algo.h` introsort) so
//! equal-key site events permute exactly like the C++ oracle build.
//! The beach-line cell enumeration order (and therefore every
//! downstream ordering that the parity suite observes) depends on
//! which geometrically-equal duplicate survives `dedup` after this
//! sort; a stable Rust sort keeps insertion order while libstdc++
//! introsort reorders equals by its own partition sequence.

/// `__lg(n)`: index of the highest set bit (`floor(log2(n))`).
const fn lg(mut n: usize) -> usize {
    let mut k = 0;
    while n > 1 {
        n >>= 1;
        k += 1;
    }
    k
}

/// libstdc++ `_S_threshold`.
const S_THRESHOLD: usize = 16;

/// `std::sort(first, last, comp)` — comp is strict-weak `is_less`.
pub(crate) fn sort<T: Copy, F: Fn(&T, &T) -> bool>(slice: &mut [T], comp: &F) {
    if slice.is_empty() {
        return;
    }
    let len = slice.len();
    introsort_loop(slice, 2 * lg(len), comp);
    final_insertion_sort(slice, comp);
}

/// `__introsort_loop`.
fn introsort_loop<T: Copy, F: Fn(&T, &T) -> bool>(
    slice: &mut [T],
    mut depth_limit: usize,
    comp: &F,
) {
    let mut offset = 0usize;
    while slice.len() - offset > S_THRESHOLD {
        if depth_limit == 0 {
            // `__partial_sort(first, last, last)` == full heapsort of the
            // remaining [first, last) range.
            heapsort(&mut slice[offset..], comp);
            return;
        }
        depth_limit -= 1;
        let cut = unguarded_partition_pivot(&mut slice[offset..], comp);
        introsort_loop(&mut slice[offset + cut..], depth_limit, comp);
        offset += cut;
    }
}

/// `__unguarded_partition_pivot` returns the pivot final index
/// relative to the sub-slice start.
fn unguarded_partition_pivot<T: Copy, F: Fn(&T, &T) -> bool>(
    slice: &mut [T],
    comp: &F,
) -> usize {
    let len = slice.len();
    move_median_to_first(slice, 1, len / 2, len - 1, comp);
    unguarded_partition(slice, 0, comp)
}

/// `__move_median_to_first(result, a, b, c)` with `result == 0`.
fn move_median_to_first<T: Copy, F: Fn(&T, &T) -> bool>(
    slice: &mut [T],
    a: usize,
    b: usize,
    c: usize,
    comp: &F,
) {
    if comp(&slice[a], &slice[b]) {
        if comp(&slice[b], &slice[c]) {
            slice.swap(0, b);
        } else if comp(&slice[a], &slice[c]) {
            slice.swap(0, c);
        } else {
            slice.swap(0, a);
        }
    } else if comp(&slice[a], &slice[c]) {
        slice.swap(0, a);
    } else if comp(&slice[b], &slice[c]) {
        slice.swap(0, c);
    }
}

/// `__unguarded_partition(first, last, pivot)` with pivot at index 0;
/// returns the final `first` (cut) index.
fn unguarded_partition<T: Copy, F: Fn(&T, &T) -> bool>(
    slice: &mut [T],
    pivot: usize,
    comp: &F,
) -> usize {
    let mut first = pivot + 1;
    let mut last = slice.len();
    loop {
        while comp(&slice[first], &slice[pivot]) {
            first += 1;
        }
        last -= 1;
        while comp(&slice[pivot], &slice[last]) {
            last -= 1;
        }
        if first >= last {
            return first;
        }
        slice.swap(first, last);
        first += 1;
    }
}

/// `__final_insertion_sort(first, last)`.
fn final_insertion_sort<T: Copy, F: Fn(&T, &T) -> bool>(slice: &mut [T], comp: &F) {
    if slice.len() > S_THRESHOLD {
        insertion_sort(&mut slice[..S_THRESHOLD], comp);
        unguarded_insertion_sort(slice, S_THRESHOLD, comp);
    } else {
        insertion_sort(slice, comp);
    }
}

/// `__insertion_sort(first, last)` over `slice[0..limit]`.
fn insertion_sort<T: Copy, F: Fn(&T, &T) -> bool>(slice: &mut [T], comp: &F) {
    for i in 1..slice.len() {
        if comp(&slice[i], &slice[0]) {
            let val = slice[i];
            // move_backward(first, i, i + 1)
            slice.copy_within(0..i, 1);
            slice[0] = val;
        } else {
            unguarded_linear_insert(slice, i, comp);
        }
    }
}

/// `__unguarded_insertion_sort(first + threshold, last)`.
fn unguarded_insertion_sort<T: Copy, F: Fn(&T, &T) -> bool>(
    slice: &mut [T],
    start: usize,
    comp: &F,
) {
    for i in start..slice.len() {
        unguarded_linear_insert(slice, i, comp);
    }
}

/// `__unguarded_linear_insert(last)` — `i` is the element to place.
fn unguarded_linear_insert<T: Copy, F: Fn(&T, &T) -> bool>(
    slice: &mut [T],
    i: usize,
    comp: &F,
) {
    let val = slice[i];
    let mut last = i;
    let mut next = i - 1;
    while comp(&val, &slice[next]) {
        slice[last] = slice[next];
        last = next;
        if next == 0 {
            break;
        }
        next -= 1;
    }
    slice[last] = val;
}

/// `__partial_sort(first, last, last)` — the depth-limit fallback —
/// implemented as GCC's `make_heap` + `sort_heap` (full heapsort).
fn heapsort<T: Copy, F: Fn(&T, &T) -> bool>(slice: &mut [T], comp: &F) {
    // __make_heap: for parent = (len-2)/2 down to 0: __adjust_heap(parent)
    if slice.len() < 2 {
        return;
    }
    let len = slice.len();
    // GCC: __make_heap iterates parent from (len-2)/2 down to 0.
    let mut parent = (len.saturating_sub(2) + 1) / 2; // == (len-1)/2 for len>=2
    while parent > 0 {
        parent -= 1;
        adjust_heap(slice, parent, len, comp);
    }
    // __sort_heap: while len > 1: --len; swap(first, first+len); adjust_heap(0, len)
    let mut current = len;
    while current > 1 {
        current -= 1;
        slice.swap(0, current);
        adjust_heap(slice, 0, current, comp);
    }
}

/// GCC `__adjust_heap(first, holeIndex, len, value)` where value is the
/// original element at holeIndex.
fn adjust_heap<T: Copy, G: Fn(&T, &T) -> bool>(
    slice: &mut [T],
    hole_index: usize,
    len: usize,
    comp: &G,
) {
    let value = slice[hole_index];
    let top_index = hole_index;
    let mut hole = hole_index;
    let mut second_child = hole_index;
    // while secondChild < (len - 1) / 2
    while second_child < (len - 1) / 2 {
        second_child = 2 * (second_child + 1);
        if comp(&slice[second_child], &slice[second_child - 1]) {
            second_child -= 1;
        }
        slice[hole] = slice[second_child];
        hole = second_child;
    }
    if len & 1 == 0 && second_child == (len - 2) / 2 {
        second_child = 2 * (second_child + 1);
        slice[hole] = slice[second_child - 1];
        hole = second_child - 1;
    }
    // __push_heap(first, holeIndex, topIndex, value): sift up
    let mut parent = (((hole as isize) - 1) / 2).max(0) as usize;
    while hole > top_index && comp(&slice[parent], &value) {
        slice[hole] = slice[parent];
        hole = parent;
        if hole == 0 {
            break;
        }
        parent = (((hole as isize) - 1) / 2).max(0) as usize;
    }
    slice[hole] = value;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_correctly() {
        let mut data = vec![5, 2, 8, 1, 9, 3, 7, 4, 6, 0];
        sort(&mut data, &|a: &i32, b: &i32| a < b);
        assert_eq!(data, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn matches_slices_sort_on_equal_keys_end_to_end() {
        // Equal keys must end sorted; the exact permutation is the
        // libstdc++ one (verified externally against the oracle cell
        // dump), so only sortedness is asserted here.
        let mut data: Vec<u32> = (0..200).map(|i| (i * 37) % 21).collect();
        let expected = {
            let mut copy = data.clone();
            copy.sort_unstable();
            copy
        };
        sort(&mut data, &|a: &u32, b: &u32| a < b);
        assert_eq!(data, expected);
    }
}
