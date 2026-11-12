# Machine-only CLI settings over an embedded project

## Source boundary

This is a Rust rewrite slice of the pinned OrcaSlicer
`345efec64c7b95a05ce5cf96df0dde66a158835c` configuration behavior, not a new
slicing pipeline. The owning library concepts are `DynamicPrintConfig`
(`src/libslic3r/Config.hpp`, `Config.cpp`),
`is_compatible_with_printer` (`src/libslic3r/Preset.cpp:909–964`), and the
embedded configuration read by `src/libslic3r/Format/bbs_3mf.cpp`.
`src/OrcaSlicer.cpp` specifies their CLI application order:

1. `:1728–1760` loads the embedded project configuration. The independent
   machine/process arms at `:2280–2374` replace only supplied kinds;
   omitting process and filament files does not discard their embedded state.
2. `:2908–2921`, `cli_process_compat_config`, translates the project's
   `print_compatible_printers` and first `compatible_machine_expression_group`
   entry back to the process compatibility keys. Nonempty original keys win.
   `:2922–2945` checks the retained process against the supplied printer through
   `Preset.cpp::is_compatible_with_printer`. The pinned build's
   `current_process_full_config` is empty, so this slice uses the project
   configuration branch, not a fabricated process preset.
3. `:3000–3005` rejects an incompatible process before committing
   `sliced_info.upward_machines`. The report uses the original upstream vector
   and preserves its ordering and multiplicity.
4. `:3099–3164`, `update_full_config`, copies supplied option values while
   excluding its explicit metadata keys. `:3237–3259` applies the loaded
   machine with `update_all = true`; machine option ownership and variant
   handling must follow that operation, not an inferred Ares merge policy.
5. `:3164–3200`, `load_project_preset`, refreshes retained printer/process
   presets independently of the supplied kinds. A supplied machine skips
   printer refresh, not retained process refresh. `Preset.cpp:1059–1088`,
   `Preset::load_external_config`, owns the default/project/system merge;
   `OrcaSlicer.cpp:2295` sets `new_printer_name` from the supplied machine;
   retained filament resolution at `:2793` requires
   `is_bbl_3mf && new_printer_name.empty()`. That predicate is false for this
   supplied named machine: refresh the omitted process, but do not resolve or
   apply retained system filaments. The process branch at `:3199–3200` remains
   eligible independently. Without supplied filaments or `--uptodate`, the
   filament application branch at `:3510–3512` has no loaded/refreshed entries.
   Reuse `project_refresh` with these distinct predicates before registered
   defaults obscure raw absence; catalog-only `from_project` remains unchanged.

## Non-negotiable decisions

- Admit omitted process and filament files only on the project-backed path
  with embedded process and filament state. Keep the public, context-free
  `ProfileSelection` constructor checks and existing complete-list behavior.
  A format extension or a Boolean flag alone is not proof of retained state.
- Do not synthesize process or filament files, select omitted kinds by a
  replacement machine's default names, or serialize an intermediate archive.
  Resolve retained identities only through explicitly supplied original catalog
  documents and the existing refresh code. Inspect embedded settings without
  loading another copy of mesh geometry.
- Use the existing catalog/profile composition and project loading seams.
  Their current complete-list-only checks and loaded-name equality checks are
  temporary compatibility shells around the cited upstream operations; they
  are not independent Ares configuration semantics.
- Keep compatibility, option ownership, configuration application and report
  snapshot timing in core. CLI reads input bytes and serializes core results;
  it must not calculate compatibility or reconstruct an upward vector.
- Use the existing report data channel for the upward snapshot, including its
  retention across later failure. Do not add a second reporting controller.
- Core remains memory-only and portable to browser WASM, Windows, macOS and
  Linux. No filesystem/environment access, dependency, global state, unsafe
  code, parser-error fallback or secondary slicing implementation is added.
- Keep every changed Rust file at or below 400 lines. Split only the new,
  directly related code when needed; do not grow the existing runner module.

## Rust destinations

| File | Source-owned responsibility |
| --- | --- |
| `crates/ares-core/src/profiles/load_settings.rs` | Independent supplied-kind ingestion and project-backed composition, `OrcaSlicer.cpp:2280–2374`. |
| `crates/ares-core/src/profiles/load_settings/project.rs` | Memory-only embedded-settings context for the previous operation, `Format/bbs_3mf.cpp` and `OrcaSlicer.cpp:1728–1760`; no duplicate geometry load. |
| `crates/ares-core/src/profiles/composition.rs` | Scoped selection construction retaining omitted embedded kinds, `OrcaSlicer.cpp:2280–2374`; preserve global checks. |
| `crates/ares-core/src/profiles/entry.rs` | Pass the actual embedded context through the existing byte entry, `OrcaSlicer.cpp:1728–1760,2280–2374`. |
| `crates/ares-core/src/project_slice/compatibility.rs` | The retained-process/new-printer branch and compatibility-key translation, `OrcaSlicer.cpp:2908–2945`, `Preset.cpp:909–964`. |
| `crates/ares-core/src/project_slice/profile_entry.rs` | Compatibility before validation and the upstream upward snapshot, `OrcaSlicer.cpp:2922–2945,3000–3005`. |
| `crates/ares-core/src/project_slice/source.rs` | Carry the snapshot in the existing core report data, `OrcaSlicer.cpp:3005`. |
| `crates/ares-core/src/project_slice/load_stages.rs` | Preserve that snapshot while filling plate report fields, `OrcaSlicer.cpp:3005,6576–6640,6926–6931`. |
| `crates/ares-core/src/project/domain/profile_input/overlay.rs` | Only machine application/ownership/variant corrections required by `OrcaSlicer.cpp:3099–3164,3237–3259`; existing project overlay remains the owner. |
| `crates/ares-cli/src/main.rs` | Transfer core's snapshot to existing success/failure serialization, corresponding to `OrcaSlicer.cpp:3005,428–432`; no configuration decisions. |

The refresh integration also owns these directly related boundaries:

| File | Source-owned responsibility |
| --- | --- |
| `crates/ares-cli/src/load_settings.rs` | Existing file-list I/O plus explicit catalog transport; extracted from `main.rs` to preserve its size limit. |
| `crates/ares-core/src/profiles/entry/load_settings.rs` | Optional catalog JSON and existing `uptodate` switch on `LoadSettingsSliceRequest`; existing catalog decoding and source wiring, extracted from `entry.rs`. |
| `crates/ares-core/src/profiles/project_refresh.rs` | Reuse the existing refresh object for selected-profile loading without loading geometry twice. |
| `crates/ares-core/src/profiles/project_refresh/preset.rs` | Skip supplied machine/process kinds; a supplied named machine also skips retained filament resolution/application (`OrcaSlicer.cpp:2295,2793`), not omitted process refresh (`:3199–3200`). Preserve catalog-only identity, inheritance, dirty-key and filament application behavior. |
| `crates/ares-core/src/project/domain/profile_input.rs` and `profile_input/three_mf.rs` | The existing single 3MF load and pre-default refresh/application seam; split the directly related loader before exceeding 400 lines. |

The table identifies allowed production boundaries, not a requirement to change
an unaffected file. Additional boundaries require a source-based scope decision.
No new preset lookup, parser, inherited-machine or general resolver is added.
The source/profile structs carry existing refresh context and report data; they
do not create a second configuration implementation.

## Included behavior and public seam

The seam is the real CLI:

```text
Native --load-settings machine.json --slice 0 --outputdir native project.3mf
Ares slice --load-settings machine.json -o ares/plate_1.gcode project.3mf
```

Use the original repository-derived
`tests/parity/master-extruder-id/stock-right-only.3mf` unchanged
(SHA-256 `69d425b79297129c1b3894ee106269d3b33b7f1badcbeed15598dfe774cff09a`).
Reuse the stock full-machine fixture at
`tests/parity/machine-full-resource/stock-machine-full.json` byte-for-byte
(SHA-256 `2a2e89e53063119290a5b8c185ea01f4e207044dc11f4e78f5a34cd22d162e4b`).
The second case adds only `"inherits": ""`; it is otherwise identical.
No process or filament list is supplied in either case.

### Explicit catalog with loaded settings

`--project-preset-catalog <PATH>` may now accompany `--load-settings`; remove
only `load_settings` from that option's `conflicts_with_all`. Conflicts with
`--load-filaments` and `--resources-dir` remain unchanged. Ordinary catalog-only
requests keep their existing branch. Combined requests use the load-settings
entry with an optional catalog JSON field and the existing `--uptodate` value;
no catalog preserves the previous load-settings behavior. This supersedes only
the load-settings exclusion in `project-derived-preset-refresh-adapters.md`.

The CLI reads the explicitly requested catalog and transports its bytes. Core
uses the existing `Catalog::from_json` and `ProjectPresetRefresh` machinery,
refreshing retained kinds only when their upstream request predicates permit.
A selected machine must not be refreshed from its catalog identity; its nonempty
name also prevents retained filament refresh (`OrcaSlicer.cpp:2295,2793`), while
omitted process refresh remains applicable (`:3199–3200`). This supersedes any
reading that omission alone permits filament refresh with a supplied machine.
Catalog-only `from_project` behavior, existing complete-list merge and
context-free constructor checks remain intact. No new file search,
environment input, catalog generation or exported settings enter production.

Reuse the existing E2E `project_supply`/`native_arguments` adapters. With supply
present, pass the same catalog file explicitly to Ares and the matching staged
Native system tree through the existing Native argument adapter. Keep original
archive/machine bytes, comparisons, Native success premises and both modes.
Record catalog identity and the complete argv. With supply absent, do not
silently discover a catalog, skip the case or excuse a mismatch.

The loaded project's embedded post-process check and retained-process
compatibility precede the original upward snapshot. Commit that snapshot before
retained refresh and later validation so a later refresh/slice failure cannot
lose it. Run refresh before collection-default materialization and explicit
machine overlay; apply user overrides last. Keep complete-list adjudication on
its existing path and load mesh geometry only once.

Runtime Native in both official and seeded modes is the independent oracle.
Both cases must return exit 0, report return code 0 and upward machines exactly
`["Bambu Lab H2D Pro 0.4 nozzle"]`, and produce nonempty G-code.
Those literals establish the premise; they do not replace the complete
Native/Ares output comparisons.

## Verification and failure modes

Before production changes, commit CLI E2Es under
`crates/ares-cli/tests/orca_parity/load_settings_cli/machine_only.rs`, registered
from the existing `load_settings_cli.rs`. Run them against actual Native and
Ares binaries on the committed test revision with an initially absent, owned
Rust target. A compiler error, missing Native output, skipped Native invocation
or stale binary is not a behavioral failure demonstration.

The tests retain input/config bytes and digests, complete argv, raw CLI path,
exit status, stdout/stderr, `result.json` and G-code. Native premise artifacts
are written before invoking Ares. Preserve the original fixture bytes and the
single declared empty-inheritance mutation; do not export substitute projects.

Failures to expose include: losing an omitted kind; relaxing the context-free
constructor; evaluating renamed compatibility keys incorrectly; using the
post-overlay printer in place of the supplied printer config; changing machine
ownership/variants; dropping the upward snapshot; losing it on a later failure;
and comparing only the first visible difference instead of complete outputs.
The catalog extension must also expose the former CLI mutual-exclusion failure,
accidentally refreshing the supplied machine or retained filaments despite its
nonempty name, skipping the eligible omitted process, filling absent values
from registered defaults before refresh, losing dirty/present-value protection,
and relaxing the retained filament/resource argument conflicts. The existing
whole-output cases must expose both config and emitted G-code changes caused
by violating the filament predicate, without changing the comparisons. Commit and execute
the catalog-supplied E2Es before production wiring; a compile error is not a failing assertion.

Compare exit status and the entire report through unchanged
`native_wall_clock::result_comparison_bytes`, including upward metadata.
That existing comparator handles elapsed wall-clock fields only. Compare the
entire G-code through unchanged `load_settings_cli::banner_normalized`, which
removes only the generator banner. Missing output is a failure, never an empty
equal stream. Keep stdout/stderr; successful Native's progress logs
are not interchangeable with Ares terminal output.

Run both Native modes, retained CLI rejection controls, rustfmt, strict locked
workspace/all-targets Clippy and host/WASM core/adapter checks. WASM compilation
does not establish browser execution. A passing focused set does not establish
whole-workspace or arbitrary machine/partial-selection parity.

## Deferred upstream behavior

- STL machine-only inputs.
- Process-only and filament-only partial selections.
- Inherited machines and machines missing their type.
- Partial filament slots.
- TPU/grouping repair.
- Other machine identities and their printer-switch paths.
- Nonempty `current_*_full_config` branches and broader preset resolution.

If faithful machine application exposes a deferred TPU/grouping or other
slicing difference, preserve its complete Native/Ares outputs and report it.
Do not repair an adjacent subsystem, encode an Ares-only expected difference,
filter the differing output, weaken comparisons or claim whole-output parity.
