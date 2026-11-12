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

## Non-negotiable decisions

- Admit omitted process and filament files only on the project-backed path
  with embedded process and filament state. Keep the public, context-free
  `ProfileSelection` constructor checks and existing complete-list behavior.
  A format extension or a Boolean flag alone is not proof of retained state.
- Do not synthesize process or filament files, resolve an omitted kind from
  default catalog names, or serialize an intermediate replacement archive.
  Inspect embedded settings without loading another copy of mesh geometry.
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

The table identifies allowed production boundaries, not a requirement to change
an unaffected file. Any additional production boundary needs a new source-based
scope decision. No inherited-machine or general resolver behavior is introduced.

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
