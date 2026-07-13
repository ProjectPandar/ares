use std::collections::BTreeSet;

use super::load::Fixture;

macro_rules! assert_source_fields {
    ($fixture:ident, $source:ident; $($key:literal => $field:ident),* $(,)?) => {{
        let mut keys = BTreeSet::new();
        $(
            assert!(keys.insert($key), "duplicate explicit projection key {}", $key);
            assert_eq!(
                &$fixture.projected.$field,
                &$fixture.$source.$field,
                "fixture projection for {}",
                $key
            );
        )*
        keys
    }};
}

mod filament;
mod printer;
mod process;
mod project;

pub(super) fn assert_all_fields(fixture: &Fixture) {
    let expected = [
        ("printer", printer::assert_fields(fixture), 62),
        ("process", process::assert_fields(fixture), 17),
        ("filament", filament::assert_fields(fixture), 53),
        ("residual", project::assert_fields(fixture), 17),
    ];
    let mut union = BTreeSet::new();

    for (scope, explicit, count) in expected {
        assert_eq!(explicit.len(), count, "{scope} explicit count");
        let source_keys = fixture.source_keys[scope]
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        assert_eq!(explicit, source_keys, "{scope} source keys");
        for key in explicit {
            assert!(union.insert(key), "duplicate source projection key {key}");
        }
    }

    assert_eq!(union.len(), 149);
}
