//! AABBTreeIndirect.hpp::partition_input:157-212; preserve swaps and ties.
use super::Input;
pub(super) fn partition(
    input: &mut [Input],
    dimension: usize,
    mut left: usize,
    mut right: usize,
    k: usize,
) {
    while left < right {
        let center = (left + right) / 2;
        let mut left_value = input[left].centroid[dimension];
        let mut center_value = input[center].centroid[dimension];
        let mut right_value = input[right].centroid[dimension];
        if left_value > center_value {
            input.swap(left, center);
            std::mem::swap(&mut left_value, &mut center_value);
        }
        if left_value > right_value {
            input.swap(left, right);
            right_value = left_value;
        }
        if center_value > right_value {
            input.swap(center, right);
            center_value = right_value;
        }
        let pivot = center_value;
        if right <= left + 2 {
            break;
        }
        let mut i = left;
        let mut j = right - 1;
        input.swap(center, j);
        loop {
            i += 1;
            while input[i].centroid[dimension] < pivot {
                i += 1;
            }
            j -= 1;
            while input[j].centroid[dimension] > pivot && i < j {
                j -= 1;
            }
            if i >= j {
                break;
            }
            input.swap(i, j);
        }
        input.swap(i, right - 1);
        if k < i {
            right = i - 1;
        } else if k == i {
            break;
        } else {
            left = i + 1;
        }
    }
}
