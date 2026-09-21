use super::{fixed_gcc_sort_by, heap_sort};

#[test]
fn sorts_integers_stably_enough_for_equal_keys() {
    let mut values = vec![5, 1, 5, 2, 1, 9, 5, 2];
    fixed_gcc_sort_by(&mut values, |a, b| a < b);
    assert_eq!(values, vec![1, 1, 2, 2, 5, 5, 5, 9]);
}

#[test]
fn preserves_libstdcxx_equal_key_permutation() {
    let mut values = (0..20).map(|id| (0, id)).collect::<Vec<_>>();
    fixed_gcc_sort_by(&mut values, |left, right| left.0 < right.0);
    assert_eq!(
        values.into_iter().map(|(_, id)| id).collect::<Vec<_>>(),
        vec![
            10, 19, 18, 17, 16, 15, 14, 13, 12, 11, 0, 9, 8, 7, 6, 5, 4, 3, 2, 1
        ]
    );
}

#[test]
fn preserves_libstdcxx_grouped_key_permutation() {
    let keys = [
        2, 1, 2, 1, 0, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2,
        0, 1, 2,
    ];
    let mut values = keys
        .into_iter()
        .enumerate()
        .map(|(id, key)| (key, id))
        .collect::<Vec<_>>();
    fixed_gcc_sort_by(&mut values, |left, right| left.0 < right.0);
    assert_eq!(
        values.into_iter().map(|(_, id)| id).collect::<Vec<_>>(),
        vec![
            30, 4, 27, 6, 24, 9, 21, 12, 18, 15, 1, 31, 28, 25, 22, 19, 13, 10, 3, 16, 7, 8, 32, 0,
            2, 29, 5, 26, 23, 11, 20, 14, 17,
        ]
    );
}

#[test]
fn heap_fallback_preserves_libstdcxx_equal_key_permutation() {
    let mut values = (0..33).map(|id| (0, id)).collect::<Vec<_>>();
    heap_sort(&mut values, &mut |left, right| left.0 < right.0);
    assert_eq!(
        values.into_iter().map(|(_, id)| id).collect::<Vec<_>>(),
        vec![
            18, 22, 10, 16, 26, 8, 20, 4, 24, 12, 32, 28, 7, 17, 3, 19, 9, 21, 1, 23, 11, 25, 5,
            27, 13, 31, 29, 15, 0, 2, 6, 14, 30,
        ]
    );
}

#[test]
fn empty_and_single_element_slices_are_noops() {
    let mut empty: Vec<i32> = Vec::new();
    fixed_gcc_sort_by(&mut empty, |a, b| a < b);
    let mut single = vec![7];
    fixed_gcc_sort_by(&mut single, |a, b| a < b);
    assert_eq!(single, vec![7]);
}

#[test]
fn large_slices_cover_heapsort_and_final_insertion_paths() {
    let mut values: Vec<u32> = (0..200).map(|index| (index * 7919) % 101).collect();
    fixed_gcc_sort_by(&mut values, |a, b| a < b);
    let mut expected = values.clone();
    expected.sort_unstable();
    assert_eq!(values, expected);
}
