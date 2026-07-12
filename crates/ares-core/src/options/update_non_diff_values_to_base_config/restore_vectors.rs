use super::normalize_stride2_floats;
use crate::SliceError;

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M253 ports this helper before the full non-diff consumer"
    )
)]
pub(super) fn apply_non_diff_stride1_set_with_restore<T: Clone>(
    source: &mut Vec<T>,
    target: &[T],
    restore_index: &[isize],
) -> Result<(), SliceError> {
    let backup_values = source.clone();
    source.clear();
    source.extend_from_slice(target);

    if target.len() != restore_index.len() {
        return Err(SliceError::InvalidInput(
            "ConfigOptionVector::set_with_restore(): Assigning from an vector with invalid restore_index size".to_owned(),
        ));
    }

    for (target_index, restore_index) in restore_index.iter().enumerate() {
        if *restore_index != -1 {
            source[target_index] = backup_values[*restore_index as usize].clone();
        }
    }
    Ok(())
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M252 ports this helper before the full non-diff consumer"
    )
)]
pub(super) fn normalized_non_diff_stride1_target_temp<T: Clone>(
    target: &[T],
    expected_size: usize,
) -> Vec<T> {
    let mut temporary = target.to_vec();
    if expected_size == 0 {
        temporary.clear();
    } else if expected_size < temporary.len() {
        temporary.truncate(expected_size);
    } else if expected_size > temporary.len() {
        temporary.resize(expected_size, target[0].clone());
    }
    temporary
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M251 ports this helper before the full non-diff consumer"
    )
)]
pub(super) fn resize_non_diff_stride1_source<T: Clone>(
    source: &mut Vec<T>,
    target: &[T],
    expected_size: usize,
) {
    if expected_size == 0 {
        source.clear();
    } else if expected_size < source.len() {
        source.truncate(expected_size);
    } else if expected_size > source.len() {
        let fill = source.first().cloned().unwrap_or_else(|| target[0].clone());
        source.resize(expected_size, fill);
    }
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M250 ports this helper before the full non-diff consumer"
    )
)]
pub(super) fn non_diff_stride1_restore_sizes<T, U>(
    source: &[T],
    target: &[U],
    expected_size: usize,
) -> (usize, usize, bool) {
    let source_size = source.len();
    let target_size = target.len();
    (
        source_size,
        target_size,
        source_size != expected_size || target_size != expected_size,
    )
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M249 ports this helper before the full non-diff consumer"
    )
)]
pub(super) fn apply_non_diff_stride2_set_with_restore(
    source: &mut Vec<f64>,
    target: &[f64],
    restore_index: &[isize],
) -> Result<(), SliceError> {
    const STRIDE: usize = 2;
    let backup_values = source.clone();
    source.clear();
    source.extend_from_slice(target);

    if target.len() != restore_index.len() * STRIDE {
        return Err(SliceError::InvalidInput(
            "ConfigOptionVector::set_with_restore(): Assigning from an vector with invalid restore_index size".to_owned(),
        ));
    }

    for (target_index, restore_index) in restore_index.iter().enumerate() {
        if *restore_index != -1 {
            let source_index = *restore_index as usize;
            for offset in 0..STRIDE {
                source[target_index * STRIDE + offset] =
                    backup_values[source_index * STRIDE + offset];
            }
        }
    }
    Ok(())
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M248 ports this helper before the full non-diff consumer"
    )
)]
pub(super) fn normalize_non_diff_stride2_restore_pair(
    source: &mut Vec<f64>,
    target: &[f64],
    expected_size: usize,
) -> Vec<f64> {
    let mut target_tmp = target.to_vec();
    normalize_stride2_floats(source, expected_size);
    normalize_stride2_floats(&mut target_tmp, expected_size);
    target_tmp
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M247 ports this helper before the full non-diff consumer"
    )
)]
pub(super) fn non_diff_stride2_restore_sizes(
    source: &[f64],
    target: &[f64],
    expected_size: usize,
) -> (usize, usize, bool) {
    let source_size = source.len();
    let target_size = target.len();
    (
        source_size,
        target_size,
        source_size != expected_size || target_size != expected_size,
    )
}
