// Apache-2.0 WITH LLVM-exception rewrite of the separately audited MSVC STL
// 14.44 `sort` control flow used as the fixed comparator-compatibility target.
const INSERTION_SORT_MAX: usize = 32;
#[derive(Clone, Copy)]
struct SortRange(usize, usize);
#[derive(Clone, Copy)]
struct MedianIndices(usize, usize, usize);
#[derive(Clone, Copy)]
struct HeapHole(usize, usize, usize);
trait Less<T>: FnMut(&T, &T) -> bool {}
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
#[cfg(test)]
#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct SortTrace {
    pub(crate) insertion_calls: usize,
    pub(crate) median3_calls: usize,
    pub(crate) ninther_calls: usize,
    pub(crate) partition_calls: usize,
    pub(crate) heap_fallback_calls: usize,
    pub(crate) partitions: Vec<[usize; 5]>,
    pub(crate) heap_entry_identities: Vec<Vec<usize>>,
}

#[cfg(test)]
struct TestTrace<I> {
    result: SortTrace,
    identity: I,
}

#[cfg(test)]
impl<T, I> TraceSink<T> for TestTrace<I>
where
    I: Fn(&T) -> usize,
{
    fn insertion(&mut self) {
        self.result.insertion_calls += 1;
    }
    fn median3(&mut self) {
        self.result.median3_calls += 1;
    }
    fn ninther(&mut self) {
        self.result.ninther_calls += 1;
    }
    fn partition(&mut self, length: usize, ideal: isize, parts: [usize; 3]) {
        self.result.partition_calls += 1;
        self.result
            .partitions
            .push([length, ideal as usize, parts[0], parts[1], parts[2]]);
    }
    fn heap_fallback(&mut self, items: &[T], first: usize, last: usize) {
        self.result.heap_fallback_calls += 1;
        self.result
            .heap_entry_identities
            .push(items[first..last].iter().map(&self.identity).collect());
    }
}
pub(in crate::geometry) fn fixed_msvc_sort_by<T, F>(items: &mut [T], mut compare: F)
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
#[cfg(test)]
pub(crate) fn fixed_msvc_sort_by_for_test<T, F, I>(
    items: &mut [T],
    mut compare: F,
    identity: I,
) -> SortTrace
where
    T: Copy,
    F: FnMut(&T, &T) -> bool,
    I: Fn(&T) -> usize,
{
    let mut trace = TestTrace {
        result: SortTrace::default(),
        identity,
    };
    let length = items.len();
    sort_unchecked(
        items,
        SortRange(0, length),
        length as isize,
        &mut compare,
        &mut trace,
    );
    trace.result
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

fn push_heap_by_index<T: Copy, F: Less<T>>(
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

fn pop_heap_hole_by_index<T: Copy, F: Less<T>>(
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

fn make_heap<T: Copy, F: Less<T>>(items: &mut [T], first: usize, last: usize, compare: &mut F) {
    let bottom = last - first;
    let mut hole = bottom >> 1;
    while hole > 0 {
        hole -= 1;
        let value = items[first + hole];
        pop_heap_hole_by_index(items, HeapHole(first, hole, bottom), value, compare);
    }
}

fn pop_heap<T: Copy, F: Less<T>>(items: &mut [T], first: usize, last: usize, compare: &mut F) {
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

fn sort_heap<T: Copy, F: Less<T>>(items: &mut [T], first: usize, mut last: usize, compare: &mut F) {
    while last - first >= 2 {
        pop_heap(items, first, last, compare);
        last -= 1;
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
