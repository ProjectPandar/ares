# O3 regression: numeric scalar arrays in JSON presets

No global scalar-shape relaxation.

## Upstream boundary

`ConfigBase::load_from_json` (`libslic3r/Config.cpp:945-1001`) handles
JSON arrays by applying legacy key handling, joining array elements with
`parse_str_arr` (`:831-866`, numeric separator comma), then calling
`set_deserialize`. `ConfigOptionFloat::deserialize`
(`Config.hpp:784-790`) uses `std::istringstream >> value` and checks
stream failure, not complete token consumption. Thus `"50,60"` loads 50.
This is **join then scalar deserialize**, not selecting an array element.
`set_deserialize` (`Config.cpp:577-579`) runs `handle_legacy` again on
that **full joined wire**, before numeric-prefix extraction. Normalizing
first can lose a legacy discriminator: `initial_layer_print_height` array
`["50%","60%"]` must be dropped by the percentage legacy rule, not
converted to absolute height 50. The LH whole-file case additionally
pins this failure mode before the helper's ordering correction.

Existing Ares `json_deserialize::{encode_array,decode}` and
`scalar::normalize` already port the typed array/prefix behavior; O3's raw
range view retains the original list and incorrectly rejects scalar shape.

## Non-negotiable boundary

Normalize numeric scalar string arrays only as JSON catalog preset
options enter the raw/typed load boundary (`profiles/catalog.rs`). Reuse
the same `encode_array` plus scalar `decode` used by the typed loader.
Keep the raw view consistent with the scalar option the loader produced
before O3 range validation. Restrict normalization to numeric scalar
kinds; do not alter vectors, non-string elements, unknown keys, or nested
array encoding rules.

Do not relax `loaded.rs::elements_of` globally. Embedded project shape
checks and `ConfigOverrides::validate_entry` remain unchanged; in
particular multi-value scalar `--overrides` still reports
`scalar option expects a single value`. Overrides are applied after
preset loading, not normalized as preset documents.

This is a `Config.cpp`/`Config.hpp` rewrite correction, not a new Ares
configuration policy. Adjacent bool/string/enum array behavior and wider
project document normalization are deferred.

## Verification

Native-first CLI preset E2E on the existing LH whole-file profile-entry
control: inherited user process with `bridge_speed=["50","50"]` and
`["50","60"]`, native single-element `["50"]` control. Preserve both
complete streams and compare only with generated-by normalization. Ares
public profile-slice entry consumes the unflattened system/user inheritance
chain; Native receives independently flattened CLI preset documents with
the selected inheritance metadata preserved. The original BBL preset
export/loading test separately locks real BBL array input and the decoded
raw scalar `"50"`.

The initial BBL/Ender-3 3MF drafts newly exposed embedded-script,
model-label and CLI filament-variant metadata differences after the range
failure was removed. Whether they are pre-existing or unrelated has not
been established; they remain open and are triaged in
`o3-bbl-profile-overlay-triage.md`.
Do not change those behaviors or normalize them out of G-code. The
already-established non-BBL STL whole-file control isolates this
`Config.cpp` scalar-array rewrite boundary without discarding the BBL
discoveries.

The test rewrite is not a comparator relaxation:
Native still runs first, both `[50,50]` and `[50,60]` remain, the Native
`[50]` control remains, and complete generated-by-only byte comparison is
unchanged.

Lock the existing override cardinality rejection with a control, and run
workspace nextest, clippy, fmt, wasm and sequential seeded/official v2 gates.
