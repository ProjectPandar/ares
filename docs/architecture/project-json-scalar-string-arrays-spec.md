# Project JSON scalar-string array rewrite slice

## Upstream boundary

Pin: OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`.

`libslic3r/Config.cpp::ConfigBase::load_from_json:1004-1060` routes JSON
arrays through legacy key handling, option-definition lookup and `parse_str_arr`.
`parse_str_arr:882-923` checks element-type consistency at every array level and
accepts strings or recursively consistent arrays, not numbers/booleans/nulls.
The separator switch at1030-1045 uses comma by default, semicolon plus C-style
string quoting for `coStrings`, and `#` for `coPointsGroups`; nested arrays use
`#` between groups. No arbitrary JSON-to-string conversion occurs.

`PrintConfig.cpp:8595` defines `thumbnails` as `coString`. Its string array
therefore becomes one comma-separated string, including the empty-string result
for an empty array. The complete config then runs thumbnail legacy/composite
normalization (`PrintConfig.cpp:10045-10070`), including format handling and
validation. Array input must not bypass that existing conversion.

The invalid-array path logs an error and breaks the option loop at1048-1053;
`load_from_json:1147-1150` subsequently runs composite conversion and returns
zero if nothing throws. Therefore distinguish rejection of an invalid option
value from the enclosing Native CLI's natural exit status. Record actual exit,
diagnostic and output behavior for the mixed-type control rather than inventing
a fatal Native return code or silently converting invalid elements.

## Rust destination and ownership

Extend the existing source-backed array normalizer in
`crates/ares-core/src/options/json_deserialize.rs` from numeric scalar kinds to
`OptionValueKind::String`. Rename `normalize_numeric_scalar_arrays` to
`normalize_scalar_arrays` and update only its existing call sites in
`project/raw_settings.rs` and `profiles/catalog/store.rs`. W2 overlap must be
cleared before editing these production paths.

Reuse the existing `encode_array`/`decode` path and its per-kind separators.
Keep the first raw legacy-key pass and the second joined-token legacy pass in
their existing order. Keep later typed thumbnail normalization unchanged.
This shared helper is the temporary Ares compatibility shell around
`ConfigBase::load_from_json`, not a second importer or an Ares-owned coercion
pipeline. The catalog call-site rename does not introduce a separate policy.

## Included and deferred behavior

Included: arrays of string tokens bound to known scalar string options in the
project-settings reader, with the same comma join as Native; empty arrays;
existing legacy handling and complete output behavior. Existing numeric scalar
handling, string-vector quoting/separators and point-group separators remain
unchanged. Preserve raw key presence and the joined scalar in downstream views.

Non-string or mixed primitive arrays must remain invalid and must never be
joined with generic formatting. Mixed string/nested-array levels must retain
the encoder's consistency rejection. Native's diagnostic/nonfatal partial-load
behavior must be reported distinctly; no new partial-load fallback is added to
Ares. Boolean/enum/point scalar expansion, unrelated option definitions and
configuration validation changes are deferred.

## Test-first observable contract

Use the tracked Native stock cube project
`tests/parity/machine-acceleration-cache-rounding/stock-control.3mf`, SHA-256
`b1a3ee564b1e78d1b87f6957f2177f6cd528906ce4790218a0dca410b4191bb8`.
Construct project-settings variants in memory; never commit a generated 3MF or
copy user bytes, option values or names into a fixture.

A valid multi-string thumbnail array must have complete Native output equal to
its comma-string control, then match the public API and CLI on that same array
input. Cover the empty-array result and a second known scalar-string option so
this is not a thumbnail-only workaround. Preserve Native replay and complete
ordered-byte comparison, including the full footer and generator-only allowance.
A mixed-type thumbnail array must exercise Native and Ares; preserve its actual
Native diagnostics/status and prove Ares does not coerce it to a valid string.

### Recorded control divergence (not a slice gate)

At test commit `ee71d08b1`, both official and seeded Native runs accepted the
mixed string/integer thumbnail array at the enclosing process level: natural
exit zero and a generated `plate_1.gcode`. Ares rejected project admission via
the public API and CLI (natural exit one, no G-code). This is the named **Native
mixed-array partial-load versus Ares admission-rejection divergence**. Preserve
it honestly; do not port Native's silent partial-load behavior in this slice.
The mixed case records outcomes, not successful Native/Ares parity. The gates
are valid string-array joining and the private user-file load.

After the failing public E2Es are committed, implement the scoped normalizer and
repeat the private user-file load gate by SHA only. Run both oracle modes,
filtered project/3MF regressions, fmt, strict workspace/all-target Clippy, WASM,
module-size/scope/trailer/hygiene gates and exact-tip evidence. Core remains
platform-neutral and uses linear existing string assembly without file I/O,
new dependencies, compatibility flags or parser fallback.
