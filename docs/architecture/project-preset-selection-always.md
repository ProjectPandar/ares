# Ordinary project preset selection: Always

## Source boundary

This is a Rust rewrite slice of OrcaSlicer
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, not a new preset-selection policy.

- `src/libslic3r/PresetBundle.cpp:6158–6365`: ordinary FFF project metadata
  splitting and process, printer and single-filament `LoadAndSelect::Always`.
- `PresetBundle.cpp:5520–5532`: `project_different_keys`, including its four
  protected identity/inheritance keys even when the serialized entry is empty.
- `src/libslic3r/Preset.cpp:1090–1119`: `Preset::load_external_config`.
- `Preset.cpp:3009–3262`: `profile_print_params_same` and
  `PresetCollection::load_external_preset`, limited to `Always`.
- `Preset.cpp:4072–4081,4203–4240,4272–4297`: selected/edited dirty state,
  saved-edit snapshot and visibility.
- `src/libslic3r/Config.cpp:531–555`: common-option typed equality and diff.
- `src/libslic3r/PrintConfig.cpp:12139–12265`: protected variant restoration
  onto the available base layout.
- `src/libslic3r/Semver.hpp:66–74,164–174`, with
  `deps_src/semver/semver.c:133–212,554–569`: version parsing and BBS
  major/minor/patch-hundreds/patch-remainder formatting, including suffixes.
- `src/libslic3r/Format/bbs_3mf.cpp:1539–1551,4171–4188`: valid OrcaSlicer
  metadata takes precedence over the Application generator version.

## Rust destination and compatibility shell

Selection and collection materialization belong in `crates/ares-core/src/profiles/`,
next to `project_refresh`, using the existing collection membership, defaults,
legacy normalization and mapped-variant operations. Loaded original/parent name
lookup and system rename metadata belong in `profiles/catalog/`.
The existing `project/domain/preset_refresh.rs::apply_project_preset_refresh`
remains the application boundary for materialized settings and explicit overrides.
`project/preset_version.rs` owns the shared source version parse/format slice used
by archive metadata and selection records. Semver ordering, arithmetic and
persistence are outside this slice.

The existing `ProjectPresetRefresh` operation is a CLI refresh port, not GUI
selection. Its `uptodate || any non-empty difference entry` rule and its
system-only base resolver must not be globally changed. Its current use by the
browser is a temporary compatibility shell, to be replaced by a later adapter
slice. This slice adds a distinct core operation; it does not switch that caller.
The unresolved-base behavior retained by the project-fill baseline remains:
keep imported collection values and fill absent values from collection defaults.
There is no vendor/resource search or installation on a miss.

The existing JSON request entry gains the distinct operation
`{"from_project":true,"load_and_select":"always"}`. Existing project-derived
CLI requests remain unchanged; `uptodate` is not part of the Always operation.
Do not introduce a separate catalog-handle load/result API. Structured selection
metadata belongs with the existing core project/settings result projection;
adapter transport is deferred. The existing `ProfileSettings` return type
serializes the Always result as `{"settings": {...}, "preset_selection": {...}}`.
`settings` is the effective option map; `preset_selection` has `process`,
`printer` and `filament` entries with selected reference, edited configuration,
outcome and collection state. Existing CLI results remain flat option maps.

The core result must carry the loaded project and per-collection identity,
selected reference versus edited effective configuration, dirty state, visibility,
and existing versus project-local provenance. Equal matches, edited matches and
project-local selections are distinct outcomes; a display name alone cannot
represent them. Existing loaded system/user presets are not overwritten by edits.
Project-local candidates remain local to the loaded project/collection state.

## Input boundary

Only ordinary FFF projects with one filament participate. Source filename is
required because external names include its basename and extension. The supplied
catalog is the entire available loaded preset set, including hidden records.
Only supplied system rename metadata is eligible for renamed-name lookup.

Multi-filament `Never`/`OnlyIfModified`, embedded-preset loading and published
project overlays are not silently approximated with repeated `Always` calls.
The new operation must make its restricted admission explicit; existing CLI
refresh and other import operations retain their existing admission behavior.

## Materialization and lookup

1. Look up the stored original name exactly; on a miss, use the loaded system
   rename map. User presets are eligible for this selection.
2. Start from the appropriate collection default and apply the imported
   collection keyset, excluding the seven print-host connection fields named in
   `Preset.cpp:1099–1103`.
3. Resolve a non-empty imported parent exactly, then through the system rename
   map. With no parent, only the original-name match that is a system preset
   can serve as the refresh base. A parentless user match is not that base.
4. Always supply the protected-key set from `project_different_keys`. An empty
   difference entry still refreshes unequal unprotected values from a base.
   Preserve protected scalar values and restore protected variant values using
   mapped indices, including paired normal/silent machine-limit vectors and
   child layouts with more variants than the parent.
5. An unavailable base leaves imported values over collection defaults. Do not
   search another vendor, a resource catalog or unrelated equivalent presets.

## Equality and selection

Comparison is typed `ConfigOption` equality for common keys, not JSON spelling,
serialized fragments or only the fields visible in a settings panel. Float and
float-or-percent equality follows `Config.hpp:834,985–995,1355–1356,1501–1512`
and `libslic3r.h:57,319–323`: strict absolute difference below the existing
`EPSILON = 1e-4`, with matching percent flags and vector lengths. Point equality
remains exact. These are production option operators, not parity tolerances. Missing-only
keys do not make `profile_print_params_same` unequal. Its complete exclusion set is:

- `compatible_prints`, `compatible_prints_condition`
- `compatible_printers`, `compatible_printers_condition`
- `inherits`
- `print_settings_id`, `filament_settings_id`, `printer_settings_id`
- `sla_print_settings_id`, `sla_material_settings_id`
- `printer_model`, `printer_variant`
- `default_print_profile`, `default_filament_profile`
- `default_sla_print_profile`, `default_sla_material_profile`

An equal original-name match selects that record, makes it visible and stays
clean. Do not apply ignored imported metadata to its edited configuration.
Only when the original-name lookup failed may a non-empty parent become the
selection candidate. Equal parent matches follow the same clean branch.

An unequal original-name or parent match selects that existing record and
applies the materialized collection keyset to its edited copy. A same-name
unequal user record wins over an equal parent. Do not scan for an equivalent
record elsewhere or create a project-local replacement on this branch.

Dirty calculation is separate from match equality: it skips only the three FFF
settings IDs and also detects presence differences for optional
`compatible_prints` and `compatible_printers` (`Preset.cpp:4203–4221`). The exposed
`dirty_options` list follows `PresetCollection::dirty_options` (`4224–4240`):
it retains settings-ID differences; only the dirty boolean excludes those IDs.
Selecting clears prior collection dirty flags, initializes edited/saved state, and makes
that selection visible. Do not encode dirtiness by renaming a stored preset or
adding a localized GUI marker.

With neither candidate available, form `prefix(basename)` where the prefix is
original name, else parent, else empty. For unequal collisions append `-1`, `-2`,
etc. to a non-empty prefix; use `1`, `2`, etc. for an empty prefix. Reuse an equal
candidate. A newly inserted candidate receives its collection settings ID,
external/project-local identity and available filament identity/version, and
starts clean. It is not persisted in the ordinary user library.

## Applying the result

Use the existing project application machinery rather than reconstructing a
project from selected names. Preserve placement, object/volume configuration,
project-specific noncollection options and filament mappings. Apply subsequent
explicit overrides last. Retain selected reference and edited values separately
so later adapters can expose identity without recomposing away imported edits.

## Included and deferred

Included: ordinary FFF process/printer/single-filament materialization; loaded
system/user exact-name selection; loaded system rename aliases; original before
parent selection; exact match exclusion and dirty rules; visibility; external
project-local naming, collision reuse and effective settings application.

Deferred: multiple-filament reverse traversal and `Never`/`OnlyIfModified`
(`PresetBundle.cpp:6366–6430`); embedded JSON loading (`Preset.cpp:2287–2387`);
published overlays; SLA; persistent saving; cloud synchronization; vendor
installation; browser/WASM/JS transport and dirty-marker rendering. No
`update_compatible(Always)` replacement is added after project selection.

## Failure modes and executable acceptance

Public core API cases and the native collection oracle use the same supplied
catalog and project configuration. The oracle calls the cited libslic3r
collection operations, not the CLI resource resolver. Retain input project,
catalog, materialized/reference/edited option values, selected identity and
flags, native command/result, and public API result as repeatable artifacts.
Run the same cases with official and seeded native environments. A CLI G-code
comparison alone does not establish installed-catalog selection parity.

`tools/chain-probe/preset-select.patch` adds an opt-in test-only entry before
normal CLI startup and a materialized-config trace at the collection operation.
It reads a caller-owned isolated data directory with `PresetBundle::load_presets`
in read-only mode, including the supplied system and user presets, then calls
`load_config_model`. The normal upstream collection operations perform the
metadata split and selection; the probe implements no matching algorithm.
It records typed option tags plus canonical serialization, materialized
collection configs and before/after selected, edited and catalog states.
Its JSON request contains `data_directory`, `projects` (each with `filename`,
`version` and a map of upstream serialized `options`) and an exclusive-create
`output` path. It never imports embedded presets or runs resource resolution.
Build it separately at the cited upstream revision; keep the executable under
a gcroot whose name contains `preset-select` and `40ecc66f7`. Never replace
an existing oracle or commit generated native results.

Cases must establish the native premise before asserting the public API result:

1. Exact system name: select existing, clean, effective values equal.
2. Exact user name: select the user record without treating it as a parentless
   system-refresh base.
3. Renamed system original and parent: use only the loaded rename map.
4. Absent child/equal parent: select parent, not a new external record.
5. Same-name unequal user/equal parent: keep the user selection and edited values.
6. Ignored metadata-only differences: clean match without importing those edits.
7. Protected real option differences: edited existing preset, dirty true.
8. Unprotected option differences with empty serialized difference entries:
   refresh from the available base, unlike the CLI rule.
9. Typed-equal differently spelled values and unequal non-panel options:
   neither raw JSON equality nor visible-field projection determines the match.
10. Missing-only options and optional compatibility presence: match and dirty
    checks follow their separate upstream rules.
11. Wholly unavailable vendor/base: project-local result, retained imported
    values, defaults for absent values, no installed-catalog mutation.
12. Candidate collisions: equal reuse, unequal suffix progression, and
    parentless/name-less numbering; newly created local selections stay clean.
13. Hidden existing match: selection becomes visible without installing a vendor.
14. Protected mapped variants: preserve child variant layout and paired machine
    limits rather than flattening or truncating to the base.
15. Applying selection preserves placement, model settings, mappings,
    noncollection options and subsequent explicit overrides.
16. Unsupported input boundary is explicit; existing CLI refresh semantics and
    the project-fill baseline miss cases remain unchanged.

Tests precede production implementation. A missing public result channel is an
observable baseline failure, not a compiler-only substitute for a failing
behavioral assertion. Static checks include workspace/all-targets strict Clippy,
rustfmt and the ordinary browser WASM build; core remains filesystem-, UI-,
terminal- and Nix-independent, with Rust modules at most 400 lines.
