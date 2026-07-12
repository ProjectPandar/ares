use super::super::option_definitions;

mod first;
mod second;
mod third;

#[test]
fn option_definitions_cover_typed_keys() {
    let keys: Vec<_> = option_definitions()
        .iter()
        .map(|definition| definition.key)
        .collect();
    let expected: Vec<_> = first::FIRST_TYPED_KEYS
        .iter()
        .chain(second::SECOND_TYPED_KEYS)
        .chain(third::THIRD_TYPED_KEYS)
        .copied()
        .collect();
    assert_eq!(keys, expected);
}

#[test]
fn option_definitions_are_sorted_without_duplicates() {
    for pair in option_definitions().windows(2) {
        assert!(pair[0].key < pair[1].key);
    }
}
