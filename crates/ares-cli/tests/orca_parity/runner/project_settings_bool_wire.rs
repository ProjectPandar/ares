//! `project_settings.config` boolean wire-form parity: upstream
//! `DynamicPrintConfig::load_from_json` (`Config.cpp:905-990`, JSON-string
//! branch `:927-929`) only deserializes JSON strings and arrays — every
//! other scalar JSON type hits the `invalid json type` else-branch and the
//! key is dropped, leaving the option default. With
//! `ConfigOptionBool::deserialize` (`Config.hpp:1828-1848`, exact `"1"`
//! true / `"0"` false) and the EnableSilent retry
//! (`enum_looks_like_true_value`, `Config.hpp:96-99`: `enabled`/`on` only),
//! a JSON `true` on `staggered_inner_seams` (default false,
//! `GCode/SeamPlacer.cpp:1601-1620`) slices Native UNSTAGGERED while Ares'
//! `OrcaBool::visit_bool` (`options/config_types/scalar_bool_int.rs`)
//! accepts the JSON boolean as true. See
//! `docs/architecture/ard-project-settings-bool-wire.md`.

#[test]
fn k1_staggered_json_true_bool_wire() {
    crate::runner::native_stream_case::check(
        (
            "project-settings-bool-wire",
            &[
                "Config.cpp:DynamicPrintConfig::load_from_json:905-990:scalar-else-branch-drops-non-string-wires",
                "Config.hpp:ConfigOptionBool::deserialize:1828-1848",
                "Config.hpp:96-99:enum_looks_like_true_value",
                "GCode/SeamPlacer.cpp:1601-1620:staggered_inner_seams",
            ],
        ),
        "k1-staggered-json-true",
        include_bytes!(
            "../../../../../tests/parity/project-settings-bool-wire/k1-staggered-json-true.3mf"
        ),
        "803a4bf91e7259068f7c1704c34454dde301281708a5d04b92fe741869a4eb86",
        (false, |_, _| {}),
    );
}
