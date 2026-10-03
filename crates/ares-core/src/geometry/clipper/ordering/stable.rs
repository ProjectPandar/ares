//! Fixed libstdc++ `stl_algo.h` normal-buffer stable-sort ordering for
//! `FillLightning::DistanceField`'s non-transitive comparator.

pub(crate) fn stable_sort_by<T: Clone>(items: &mut [T], mut less: impl FnMut(&T, &T) -> bool) {
    if items.len() < 2 {
        return;
    }
    let middle = items.len().div_ceil(2);
    let mut buffer = items[..middle].to_vec();
    let (left, right) = items.split_at_mut(middle);
    sort_half(left, &mut buffer, &mut less);
    sort_half(right, &mut buffer[..right.len()], &mut less);
    if middle > items.len() - middle {
        let right_len = items.len() - middle;
        let buffer = &mut buffer[..right_len];
        buffer.clone_from_slice(&items[middle..]);
        let (mut left, mut right, mut out) = (middle, right_len, items.len());
        while left > 0 && right > 0 {
            out -= 1;
            if less(&buffer[right - 1], &items[left - 1]) {
                left -= 1;
                items[out] = items[left].clone();
            } else {
                right -= 1;
                items[out] = buffer[right].clone();
            }
        }
        items[out - right..out].clone_from_slice(&buffer[..right]);
        return;
    }
    buffer.clone_from_slice(&items[..middle]);
    let (mut left, mut right, mut out) = (0, middle, 0);
    while left < middle && right < items.len() {
        if less(&items[right], &buffer[left]) {
            items[out] = items[right].clone();
            right += 1;
        } else {
            items[out] = buffer[left].clone();
            left += 1;
        }
        out += 1;
    }
    items[out..out + middle - left].clone_from_slice(&buffer[left..]);
}

fn sort_half<T: Clone>(items: &mut [T], buffer: &mut [T], less: &mut impl FnMut(&T, &T) -> bool) {
    const CHUNK_SIZE: usize = 7;
    for chunk in items.chunks_mut(CHUNK_SIZE) {
        insertion_sort(chunk, less);
    }
    let mut width = CHUNK_SIZE;
    while width < items.len() {
        merge_pass(items, buffer, width, less);
        width *= 2;
        merge_pass(buffer, items, width, less);
        width *= 2;
    }
}

fn insertion_sort<T: Clone>(items: &mut [T], less: &mut impl FnMut(&T, &T) -> bool) {
    for index in 1..items.len() {
        let value = items[index].clone();
        let mut hole = index;
        if less(&value, &items[0]) {
            while hole > 0 {
                items[hole] = items[hole - 1].clone();
                hole -= 1;
            }
        } else {
            while less(&value, &items[hole - 1]) {
                items[hole] = items[hole - 1].clone();
                hole -= 1;
            }
        }
        items[hole] = value;
    }
}

fn merge_pass<T: Clone>(
    source: &[T],
    destination: &mut [T],
    width: usize,
    less: &mut impl FnMut(&T, &T) -> bool,
) {
    for start in (0..source.len()).step_by(2 * width) {
        let middle = (start + width).min(source.len());
        let end = (start + 2 * width).min(source.len());
        let (mut left, mut right, mut out) = (start, middle, start);
        while left < middle && right < end {
            if less(&source[right], &source[left]) {
                destination[out] = source[right].clone();
                right += 1;
            } else {
                destination[out] = source[left].clone();
                left += 1;
            }
            out += 1;
        }
        destination[out..out + middle - left].clone_from_slice(&source[left..middle]);
        out += middle - left;
        destination[out..end].clone_from_slice(&source[right..end]);
    }
}
