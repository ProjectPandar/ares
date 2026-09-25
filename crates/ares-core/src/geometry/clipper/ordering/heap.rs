//! MSVC `std::sort` heap fallback (`sort_heap`/`make_heap`/`pop_heap`) split
//! from `ordering.rs` to keep each source file under the LOC cap.
use super::{HeapHole, Less};

pub(super) fn push_heap_by_index<T: Copy, F: Less<T>>(
    items: &mut [T],
    position: HeapHole,
    value: T,
    compare: &mut F,
) {
    let HeapHole(first, hole, top) = position;
    let mut hole = hole as isize;
    let top = top as isize;
    let mut index = (hole - 1) >> 1;
    while top < hole && compare(&items[first + index as usize], &value) {
        items[first + hole as usize] = items[first + index as usize];
        hole = index;
        index = (hole - 1) >> 1;
    }
    items[first + hole as usize] = value;
}

pub(super) fn pop_heap_hole_by_index<T: Copy, F: Less<T>>(
    items: &mut [T],
    position: HeapHole,
    value: T,
    compare: &mut F,
) {
    let HeapHole(first, mut hole, bottom) = position;
    let top = hole;
    let mut index = hole;
    let max_non_leaf = (bottom - 1) >> 1;
    while index < max_non_leaf {
        index = 2 * index + 2;
        if compare(&items[first + index], &items[first + index - 1]) {
            index -= 1;
        }
        items[first + hole] = items[first + index];
        hole = index;
    }
    if index == max_non_leaf && bottom.is_multiple_of(2) {
        items[first + hole] = items[first + bottom - 1];
        hole = bottom - 1;
    }
    push_heap_by_index(items, HeapHole(first, hole, top), value, compare);
}

pub(super) fn make_heap<T: Copy, F: Less<T>>(
    items: &mut [T],
    first: usize,
    last: usize,
    compare: &mut F,
) {
    let bottom = last - first;
    let mut hole = bottom >> 1;
    while hole > 0 {
        hole -= 1;
        let value = items[first + hole];
        pop_heap_hole_by_index(items, HeapHole(first, hole, bottom), value, compare);
    }
}

pub(super) fn pop_heap<T: Copy, F: Less<T>>(
    items: &mut [T],
    first: usize,
    last: usize,
    compare: &mut F,
) {
    if last - first >= 2 {
        let destination = last - 1;
        let value = items[destination];
        items[destination] = items[first];
        pop_heap_hole_by_index(
            items,
            HeapHole(first, 0, destination - first),
            value,
            compare,
        );
    }
}

pub(super) fn sort_heap<T: Copy, F: Less<T>>(
    items: &mut [T],
    first: usize,
    mut last: usize,
    compare: &mut F,
) {
    while last - first >= 2 {
        pop_heap(items, first, last, compare);
        last -= 1;
    }
}
