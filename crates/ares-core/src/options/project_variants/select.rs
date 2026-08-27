use crate::SliceError;

pub(super) fn select_stride<T: Clone>(
    values: &[T],
    base_indices: &[usize],
    stride: usize,
    key: &str,
) -> Result<Vec<T>, SliceError> {
    if values.is_empty() {
        return Err(invalid(key));
    }

    base_indices
        .iter()
        .flat_map(|base| (0..stride).map(move |offset| base + offset))
        .map(|index| {
            values
                .get(index)
                .or_else(|| values.first())
                .cloned()
                .ok_or_else(|| invalid(key))
        })
        .collect()
}

fn invalid(key: &str) -> SliceError {
    SliceError::InvalidInput(format!("invalid Orca option {key}"))
}
