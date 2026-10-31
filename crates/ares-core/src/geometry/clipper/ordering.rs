mod gcc;
mod heap;
mod stable;
pub(crate) use gcc::fixed_gcc_sort_by;
use heap::{make_heap, sort_heap};
pub(crate) use stable::stable_sort_by;

// Apache-2.0 WITH LLVM-exception rewrite of the separately audited MSVC STL
// 14.44 `sort` control flow used as the fixed comparator-compatibility target.
const INSERTION_SORT_MAX: usize = 32;
#[derive(Clone, Copy)]
struct SortRange(usize, usize);
#[derive(Clone, Copy)]
struct MedianIndices(usize, usize, usize);
#[derive(Clone, Copy)]
pub(super) struct HeapHole(pub usize, pub usize, pub usize);
pub(super) trait Less<T>: FnMut(&T, &T) -> bool {}
impl<T, F> Less<T> for F where F: FnMut(&T, &T) -> bool {}
trait TraceSink<T> {
    fn insertion(&mut self) {}
    fn median3(&mut self) {}
    fn ninther(&mut self) {}
    fn partition(&mut self, _length: usize, _ideal: isize, _parts: [usize; 3]) {}
    fn heap_fallback(&mut self, _items: &[T], _first: usize, _last: usize) {}
}
struct NoTrace;
impl<T> TraceSink<T> for NoTrace {}

pub(crate) fn fixed_msvc_sort_by<T, F>(items: &mut [T], mut compare: F)
where
    T: Copy,
    F: FnMut(&T, &T) -> bool,
{
    let mut trace = NoTrace;
    let length = items.len();
    sort_unchecked(
        items,
        SortRange(0, length),
        length as isize,
        &mut compare,
        &mut trace,
    );
}

fn insertion_sort<T: Copy, F: Less<T>, S: TraceSink<T>>(
    items: &mut [T],
    first: usize,
    last: usize,
    compare: &mut F,
    trace: &mut S,
) {
    trace.insertion();
    if first == last {
        return;
    }

    for middle in first + 1..last {
        let mut hole = middle;
        let value = items[middle];
        if compare(&value, &items[first]) {
            for source in (first..middle).rev() {
                items[source + 1] = items[source];
            }
            items[first] = value;
        } else {
            while compare(&value, &items[hole - 1]) {
                items[hole] = items[hole - 1];
                hole -= 1;
            }
            items[hole] = value;
        }
    }
}
fn median3<T, F: Less<T>, S: TraceSink<T>>(
    items: &mut [T],
    indices: MedianIndices,
    compare: &mut F,
    trace: &mut S,
) {
    let MedianIndices(first, middle, last) = indices;
    trace.median3();
    if compare(&items[middle], &items[first]) {
        items.swap(middle, first);
    }
    if compare(&items[last], &items[middle]) {
        items.swap(last, middle);
        if compare(&items[middle], &items[first]) {
            items.swap(middle, first);
        }
    }
}
fn guess_median<T, F: Less<T>, S: TraceSink<T>>(
    items: &mut [T],
    indices: MedianIndices,
    compare: &mut F,
    trace: &mut S,
) {
    let MedianIndices(first, middle, last) = indices;
    let count = last - first;
    if count > 40 {
        trace.ninther();
        let step = (count + 1) >> 3;
        let two_step = step << 1;
        median3(
            items,
            MedianIndices(first, first + step, first + two_step),
            compare,
            trace,
        );
        median3(
            items,
            MedianIndices(middle - step, middle, middle + step),
            compare,
            trace,
        );
        median3(
            items,
            MedianIndices(last - two_step, last - step, last),
            compare,
            trace,
        );
        median3(
            items,
            MedianIndices(first + step, middle, last - step),
            compare,
            trace,
        );
    } else {
        median3(items, indices, compare, trace);
    }
}
fn swap_distinct<T>(items: &mut [T], first: usize, second: usize) {
    if first != second {
        items.swap(first, second);
    }
}
fn partition_by_median<T, F: Less<T>, S: TraceSink<T>>(
    items: &mut [T],
    range: SortRange,
    ideal: isize,
    compare: &mut F,
    trace: &mut S,
) -> (usize, usize) {
    let SortRange(first, last) = range;
    let middle = first + ((last - first) >> 1);
    guess_median(
        items,
        MedianIndices(first, middle, last - 1),
        compare,
        trace,
    );
    let mut pivot_first = middle;
    let mut pivot_last = pivot_first + 1;

    while first < pivot_first
        && !compare(&items[pivot_first - 1], &items[pivot_first])
        && !compare(&items[pivot_first], &items[pivot_first - 1])
    {
        pivot_first -= 1;
    }
    while pivot_last < last
        && !compare(&items[pivot_last], &items[pivot_first])
        && !compare(&items[pivot_first], &items[pivot_last])
    {
        pivot_last += 1;
    }

    let mut greater_first = pivot_last;
    let mut greater_last = pivot_first;
    loop {
        while greater_first < last {
            if compare(&items[pivot_first], &items[greater_first]) {
                greater_first += 1;
                continue;
            }
            if compare(&items[greater_first], &items[pivot_first]) {
                break;
            }
            swap_distinct(items, pivot_last, greater_first);
            pivot_last += 1;
            greater_first += 1;
        }

        while first < greater_last {
            let previous = greater_last - 1;
            if compare(&items[previous], &items[pivot_first]) {
                greater_last -= 1;
            } else if compare(&items[pivot_first], &items[previous]) {
                break;
            } else {
                pivot_first -= 1;
                swap_distinct(items, pivot_first, previous);
                greater_last -= 1;
            }
        }

        if greater_last == first && greater_first == last {
            trace.partition(
                last - first,
                ideal,
                [
                    pivot_first - first,
                    pivot_last - pivot_first,
                    last - pivot_last,
                ],
            );
            return (pivot_first, pivot_last);
        }
        if greater_last == first {
            if pivot_last != greater_first {
                items.swap(pivot_first, pivot_last);
            }
            pivot_last += 1;
            items.swap(pivot_first, greater_first);
            pivot_first += 1;
            greater_first += 1;
        } else if greater_first == last {
            greater_last -= 1;
            pivot_first -= 1;
            if greater_last != pivot_first {
                items.swap(greater_last, pivot_first);
            }
            pivot_last -= 1;
            items.swap(pivot_first, pivot_last);
        } else {
            greater_last -= 1;
            items.swap(greater_first, greater_last);
            greater_first += 1;
        }
    }
}
fn sort_unchecked<T: Copy, F: Less<T>, S: TraceSink<T>>(
    items: &mut [T],
    range: SortRange,
    mut ideal: isize,
    compare: &mut F,
    trace: &mut S,
) {
    let SortRange(mut first, mut last) = range;
    loop {
        if last - first <= INSERTION_SORT_MAX {
            insertion_sort(items, first, last, compare, trace);
            return;
        }
        if ideal <= 0 {
            trace.heap_fallback(items, first, last);
            make_heap(items, first, last, compare);
            sort_heap(items, first, last, compare);
            return;
        }

        let (middle_first, middle_last) =
            partition_by_median(items, SortRange(first, last), ideal, compare, trace);
        ideal = (ideal >> 1) + (ideal >> 2);
        if middle_first - first < last - middle_last {
            sort_unchecked(items, SortRange(first, middle_first), ideal, compare, trace);
            first = middle_last;
        } else {
            sort_unchecked(items, SortRange(middle_last, last), ideal, compare, trace);
            last = middle_first;
        }
    }
}
