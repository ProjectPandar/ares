# Project-derived preset refresh

Status: accepted. The strict complete-config witness remains an enabled regression until the independent legacy-enum import-order dependency is integrated.

## Owning upstream boundary

Pinned upstream: OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`.

- `src/libslic3r/Preset.cpp:1059–1088`, `Preset::load_external_config`: start from the owning preset's defaults, overlay the project's values, and refresh non-dirty keys from the resolved system base. Connection credentials are excluded.
- `src/libslic3r/PresetBundle.cpp:5483–5495`, `project_different_keys`: decode the C-style string list and protect preset identity keys. `:724+`, `resolve_system_preset`, owns vendor resolution and inheritance; `Preset.cpp` owns the per-kind option membership/default base.
- `src/OrcaSlicer.cpp:1857–1891`: derive system identities from nonempty `inherits_group` slots, otherwise the project's preset IDs. Slot order is process, filaments, machine.
- `src/OrcaSlicer.cpp:2774–2816`: derive `refresh_unlisted` from `uptodate` or any nonempty dirty-key entry; resolve un-replaced project filaments in slot order.
- `src/OrcaSlicer.cpp:3163–3200`, `load_project_preset`: apply printer/process refreshed values, preserving already-present values when `refresh_unlisted` is false and excluding separately owned bookkeeping keys.
- `src/OrcaSlicer.cpp:3510–3728`: merge refreshed filament options per original slot, protect authored/dirty values, and construct the dynamic condition groups from the applied filament configs.
- `src/libslic3r/GCode.cpp:8244–8354`, `append_full_config`: serialize the applied config. It does not discover presets or synthesize profile-dependent keys.

This is a Rust rewrite of that preset-load/apply slice, not a new slicing pipeline.

## Public byte contract

Reuse `ProfileSliceRequest`, `profile_settings`, and `slice_with_profiles` (including processed/validated variants). The caller continues to supply `catalog_json` explicitly. Add a distinct selection shape:

```json
{"from_project": true, "uptodate": false}
```

`uptodate` defaults to false. This mode is exclusive of explicit machine/process/filament names, and requires a 3MF project with embedded preset identities. The existing explicit-selection shape and its replacement semantics remain unchanged. `from_project: false` is not an alternative spelling of explicit selection.

The catalog envelope is the existing vendor index/files plus imports transport. The adapter supplies the original preset documents and all required parent documents as bytes, not precomputed effective settings. Core resolves identity and inheritance from those bytes. No filesystem, resource-directory lookup, environment lookup, network access, built-in machine catalog, or implicit fallback is permitted in core.

A nonempty required identity with an unavailable document/parent is an explicit request error naming the missing preset. Upstream CLI can log and retain a project when its installed resolver cannot find a preset; this request instead requires a complete supplied catalog and must not silently claim a successful refresh from an incomplete supply. This is a deliberate public supply-boundary restriction, not permission to substitute host defaults. An empty upstream identity contributes no resolved preset.

CLI resource discovery/packaging is adapter-owned and outside this initial API slice. A bare-project request without catalog bytes does not secretly gain refresh behavior. Native runs in the E2E tests receive the corresponding upstream system preset tree; Rust receives its same documents through the catalog transport.

## Rust destination and invariants

Introduce a small project-refresh module under `crates/ares-core/src/profiles/`, sharing the existing catalog's record/parent resolution and per-kind option ownership. Dispatch the new selection shape in `profiles/entry.rs`; feed a refreshed loaded project into the existing project/plate lifecycle. Do not route it through the explicit replacement overlay and then try to undo overwritten values.

The settings view and slicing entries must share the same preparation result. Existing `project/domain/profile_input` replacement behavior remains a separate source-owned mode, not a general refresh algorithm. Existing scaffolding is reused only as the temporary transport/lifecycle shell around the cited upstream preset loading boundary.

1. Preserve authored preset IDs; resolve the system identity from the source-owned `inherits_group` layout. Preserve original filament positions and count.
2. Resolve the inheritance chain and the per-kind preset default base before applying project values. Do not confuse registered PrintConfig defaults with a named system preset's values.
3. Decode dirty lists using the source string-list rules, not ad-hoc splitting. Protect source identity and connection-info exclusions.
4. With all dirty entries empty and `uptodate=false`, fill missing keys but retain present project values. With dirty entries or `uptodate=true`, refresh unlisted values and protect listed changes. A dirty machine slot must not protect process or filament values.
5. Apply filament refresh with source slot/variant semantics; preserve complete vector length/order and nils. Reuse existing source-owned variant mapping rather than invent a column-selection rule.
6. Carry applied raw presence separately from typed defaults. Rebuild bookkeeping only where the source filament application creates it.
7. Apply explicit user overrides last. Never alter unrelated registry/metadata records or global PrintConfig defaults to reproduce a named profile.
8. Keep handwritten files below 400 lines and core platform-neutral for browser WASM, Windows, macOS, and Linux.

## Why the footer keys follow refresh

`PrintConfig.cpp:3275+` defines `pellet_flow_coefficient` as `coFloats` with default `{0.4157}`; `Preset.cpp:1493` includes it in the filament preset. That does not make it unconditionally present in a loaded dynamic print config. `compatible_process_expression_group` likewise has dynamic presence: the CLI filament application may create indexed empty slots, while `PresetBundle` aggregation's `add_if_some_non_empty` has a different condition.

Retained Native controls prove both keys can be absent when both are absent from the 3MF; a resolved filament preset can instead contribute the coefficient and an empty condition slot. Therefore neither key is added as an export default or by matching a fixture/preset name. Their production change follows actual preset application; validate an omission control as well as a resolved-filament witness before claiming footer closure.

The preset default base also honors the registered `default_is_nil` state. A nullable override's numeric seed is not an applied override: materialize its source nil vector, then overlay actual supplied values. This preserves the existing exporter omission rule without introducing key-specific footer bans or changing registry defaults.

## Test-first qualification

First RED: the real public `profile_settings` call receives catalog bytes and the project-derived selection shape, after Native has exported its applied settings for the same project. Baseline rejection of the new mode is a runnable public request failure, not a compile-error RED. Write all tests before production changes.

Use the existing two-object A1 3MF (unchanged missing-key case) and authored variants of the same archive. Native `--export-settings` runs first. Compare entire values for the three demonstrated preset-owned fields: bottom/top penetration layers and rod clearance. Cover missing-key fill, present-value retention without refresh, dirty-key protection across process/machine slots, and explicit `uptodate`. Preserve inputs, catalog, producer identity, Native settings, API result/error, and comparison manifests as repeatable artifacts. Negative controls must demonstrate that a blanket named-preset overwrite is incorrect.

Before final GREEN qualification, add public slicing/export coverage of resolved-filament presence and omitted-key controls, alongside unchanged explicit-selection regression cases. Full G-code comparison remains strict; do not remove a Native precondition to make a case reach comparison. A settings-boundary success alone does not claim whole-output or footer parity.

## Included and deferred

Included: byte-supplied named printer/process/filament refresh, defaults and inheritance, dirty-key protection, missing-key fill, applied presence, and existing public API lifecycle integration. No fixture-specific constants or output snapshots.

Deferred: CLI/browser catalog acquisition UI, printer switching, replacement-preset adjudication, layout-engine rewrites, the other scalar-to-vector migrations, arc-fitting investigation, unrelated exporter gaps, and FULL acceptance. Existing explicit-selection and bare-project APIs retain their current contract; they are not fallback paths for an unsuccessful project-derived request.

Implementation gate: submit this ARD and public RED evidence for review before any GREEN production edit.
