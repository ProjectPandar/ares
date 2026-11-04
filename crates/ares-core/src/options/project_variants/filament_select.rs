//! Shared payload gathering (`PrintConfig.cpp:11862–11882`).

pub(super) fn select_filament<T: Clone>(destination: &mut Vec<T>, source: &[T], indices: &[usize]) {
    let Some(first) = source.first() else {
        return;
    };
    *destination = indices
        .iter()
        .map(|&index| source.get(index).unwrap_or(first).clone())
        .collect();
}
