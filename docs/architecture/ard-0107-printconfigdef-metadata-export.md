# ARD-0107: PrintConfigDef metadata export for the web settings UI

## Upstream boundary

`OrcaSlicer/src/libslic3r/PrintConfig.cpp`
(`PrintConfigDef::init_common_params`, `init_fff_params`,
`init_extruder_option_keys`, `init_filament_option_keys`) and
`OrcaSlicer/src/libslic3r/Config.hpp` (`ConfigOptionDef` field set,
`ConfigOptionMode`, `GUIType`). `MaterialType.cpp` supplies the
`filament_type` enum vocabulary.

## Decision

Option presentation metadata is generated, never hand-copied.
`scripts/gen-configdef.mjs` parses the upstream definition statements and
emits `crates/ares-core/src/options/registry/definitions/metadata.rs`, a
`&'static` table sorted by key for binary search. The generator's `--check`
mode diffs the committed table; `.github/workflows/tier1.yml` runs it in CI,
so an upstream `PrintConfig.cpp` change without regeneration fails the gate.

`OptionDefinition` keeps only `key`, `kind`, `default_value`; the generated
`OptionMetadata` table carries the GUI fields. `print_config_def_json()` in
`ares-core` merges both into the exported JSON; `ares-wasm` exposes it as
`printConfigDef() -> Vec<u8>`.

## Included upstream fields

label, full_label, category, tooltip, sidetext, enum_values, enum_labels,
enum_keys_map (as `enum_keys` ints aligned to `enum_values`, `-1` for values
absent from the map), min, max, max_literal, mode, nullable, gui_type,
gui_flags, multiline, full_width, is_code, readonly, height, width,
ratio_over, aliases, printer_technology. Loop-generated defs
(`machine_max_*` axes block, `filament_extruder_override_keys` nullable
copies, `MaterialType::all()` enum values) are expanded by the generator,
not hardcoded.

## Deferred upstream behavior

- `cli` / `cli_params` (command-line help metadata, unused by the web UI).
- `init_sla_params` section (SLA technology is not a Tier-1 deliverable;
  SLA-only keys stay kind-only in `OPTION_DEFINITIONS`).
- `shortcut` (no upstream assignments in the covered range; field kept as
  `[]`).
- `serialization_key_ordinal`, `is_value_valid` overrides, `placeholder`
  machinery.
- Typed value decoding (`decodeConfigValue`) is the follow-up slice:
  serialize()-shaped strings → JSON bool/number/string/array, with nullable
  `nil` elements → `null`.

## Schema notes for consumers

- `type` uses `OptionValueKind` snake_case; vector kinds end in `s`
  (`floats`, `bools`, …), `*_nullable` vectors accept `nil` elements.
- `mode`: `simple|advanced|expert|develop` (comSimple..comDevelop).
- `gui_type`: `undefined|i_enum_open|f_enum_open|color|select_open|slider|legend|one_string`.
- `default_value` is the crate's existing deserialize-compatible string
  (NOT upstream `serialize()` quoting; `%` suffixes, `1`/`0` bools in
  vectors and `x`-separated points match `ConfigOption*` deserialization
  input, not the escaped-and-quoted Strings serialize form).
- `label`/`tooltip`/`sidetext`/`category`/`full_label` are the upstream
  English msgid source strings; no translation is performed.
- `enum_labels` empty ⇒ fall back to `enum_values` (Field.cpp rule).
- `min`/`max` apply per element for vector options.
- `groups` exports upstream `m_extruder_option_keys`, `m_filament_option_keys`,
  `m_extruder_retract_keys`, `m_filament_retract_keys` (from the existing
  `key_lists` tables).

## Split with pi-tip

This slice exports static definitions only. `ConfigManipulation` toggle /
update detection logic is pi-tip's slice and consumes this metadata.

## Verification

`scripts/verify-configdef.mjs` re-extracts every field independently from
`PrintConfig.cpp` (region-based scan; shares no code with the generator) and
compares each field against `printConfigDef()` output produced by
`cargo run -p ares-core --release --example print_config_def`. The report
(`artifacts/configdef-verification.json`) reconciles the add-call counts:
838 `this->add` calls in the file = 653 PrintConfigDef calls (32 common +
621 fff, including 3 axis-loop and 16 override-loop sites) + 76
`init_sla_params` (excluded: SLA out of scope) + 109 calls in sibling
`*ConfigDef` classes (Timestamps, Dimensions, OtherPresets, CustomGcode,
… — excluded: not `PrintConfigDef`). 653 sites collapse to 675 exported
keys: −3 duplicate re-adds (upstream map overwrite semantics:
`preset_names`, `retract_lift_above`, `retract_lift_below`), +9 from the
axis loop (3 sites → 12 keys), +16 nullable filament overrides.

## Binary size

`ares_wasm.wasm` release build, before/after this slice:

| build | raw bytes | gzip -9 |
| ----- | --------- | ------- |
| base  2863ef3e | 10,321,423 | 3,188,585 |
| this slice     | 10,586,848 | 3,242,808 |
| delta          | +265,425   | +54,223 |

The gzip delta (+53 kB) stays under the 150 kB budget, so the metadata ships
inline in the main wasm module. If later slices (e.g. full enum translations
or per-locale tables) push the export much larger, revisit a split data
package (`@ares-slicer/profiles`-style) with lazy loading; field parsing
stays in Rust either way.
