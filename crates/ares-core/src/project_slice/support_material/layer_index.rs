//! `SupportCommon.hpp::idx_lower_or_equal` with its descending cached index.

pub(super) fn idx_higher_or_equal<T>(
    layers: &[T],
    cached: &mut i32,
    higher_equal: impl Fn(&T) -> bool,
) -> i32 {
    idx_higher_or_equal_indexed(layers.len(), cached, |index| higher_equal(&layers[index]))
}

pub(super) fn idx_higher_or_equal_indexed(
    len: usize,
    cached: &mut i32,
    higher_equal: impl Fn(usize) -> bool,
) -> i32 {
    if len == 0 {
        *cached = 0;
    } else if *cached == -1 {
        let mut low = 0;
        let mut high = len - 1;
        while low + 1 < high {
            let middle = (low + high) / 2;
            if higher_equal(middle) {
                high = middle;
            } else {
                low = middle;
            }
        }
        *cached = if higher_equal(low) {
            low as i32
        } else if higher_equal(high) {
            high as i32
        } else {
            len as i32
        };
    } else {
        while (*cached as usize) < len && !higher_equal(*cached as usize) {
            *cached += 1;
        }
    }
    *cached
}

pub(super) fn idx_lower_or_equal<T>(
    layers: &[T],
    cached: &mut i32,
    lower_equal: impl Fn(&T) -> bool,
) -> i32 {
    if layers.is_empty() {
        *cached = -1;
    } else if *cached < -1 {
        let mut low = 0;
        let mut high = layers.len() - 1;
        while low + 1 < high {
            let middle = (low + high) / 2;
            if lower_equal(&layers[middle]) {
                low = middle;
            } else {
                high = middle;
            }
        }
        *cached = if lower_equal(&layers[high]) {
            high as i32
        } else if lower_equal(&layers[low]) {
            low as i32
        } else {
            -1
        };
    } else {
        while *cached >= 0 && !lower_equal(&layers[*cached as usize]) {
            *cached -= 1;
        }
    }
    *cached
}
