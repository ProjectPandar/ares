# Spec: `--load-settings` -17 branch 3 + `-16` upward gate (compat17 continuation)

Continues `2026-10-10-cli-load-settings-compat17-branches.md`. Owns the
adjudication verdict only; the compatible-path outputs stay in the partial-list
slice.

## Upstream boundary

- `OrcaSlicer.cpp:2575-2606` — the four-branch `process_compatible`
  computation. Branch 3 is `:2595-2605`: `new_printer_name` empty and
  `new_process_name` non-empty → `new_print_compatible_printers`
  must contain `current_printer_system_name`.
- `OrcaSlicer.cpp:2624-2647` — `machine_switch` relaxation plus the
  upward gate. Entry requires the branch verdict incompatible AND
  `new_printer_name` non-empty AND `current_printer_name` non-empty
  AND `new_printer_name != current_printer_name` (`:2624`). Inside,
  `new_process_name` empty sets `process_compatible = true` BEFORE
  the upward scan (`:2625-2627`), so the `-16` check at `:2638`
  (`if (!process_compatible)`) can never fire for a machine-only
  list — `-16` is reachable only with machine+process lists. With a
  non-empty `upward_compatible_printers` vector: a
  `new_printer_system_name` hit sets `machine_upwards` and
  `process_compatible` (`:2630-2637`); a miss exits
  `CLI_3MF_NEW_MACHINE_NOT_SUPPORTED` (`Utils.hpp:41` → 240).
  `machine_upwards`'s downstream effects are deferred (see Deferred).
- `OrcaSlicer.cpp:2284-2304` — `upward_compatible_printers` source for
  the CLI path: `uptodate_configs` (the `--load-settings` file list)
  iterated for `machine` files; a machine file whose name differs from
  `current_printer_system_name` exits `CLI_CONFIG_FILE_ERROR` (-5);
  the file's `upward_compatible_machine` replaces the embedded vector
  (`UpwardMachineInput::UpToDateFiles` already ports this,
  `project/upward_machine.rs:96-142`).
- `OrcaSlicer.cpp:2650-2654` + `Utils.hpp:42` — `-17` rejection channel
  (already landed for branches 1-2).
- `OrcaSlicer.cpp:2597-2606`, `:3050-3051`, `:3056-3070` — compatible-path
  outputs (auto rename, `compatible_printers` append, slot-0
  `different_settings_to_system`) — **partial-list slice boundary, deferred**.

## Correction to merged branch-1 slice

The merged branch-1 slice applies the `machine_switch` accept unconditionally on
`new_printer_name != current_printer_name`. Upstream (`:2625-2627`)
auto-accepts only when `new_process_name` is empty; with a process
file present the verdict still needs the upward-vector hit (non-empty
vector) or falls to `-17`. Fix `reject_incompatible_load_settings`:
restrict the relaxation to `names.process.is_none()` and route the
machine+process case through the upward gate / `-17`.

## Ares destination

- `project_slice/compatibility.rs` — extend
  `reject_incompatible_load_settings`:
  1. Branch 3: `names.machine.is_none() && names.process.is_some()` →
     `process.compatible_printers` must contain the embedded printer
     system name (`printer_system_name(raw)`).
  2. Upward gate (`:2624-2647`): fires only when ALL hold —
     branch verdict incompatible AND `machine.name` non-empty AND
     `current_printer_name` non-empty AND
     `machine.name != current_printer_name` AND `names.process`
     is `Some` (machine-only lists set `process_compatible` first,
     `:2625-2627`, so `-16` is unreachable without a process file)
     AND the upward vector is non-empty AND it does not contain
     `new_printer_system_name` →
     `SliceError::NewMachineNotSupported` (-16/240). With a process
     file the relaxation does NOT auto-accept: compatibility still
     requires the upward hit (no `machine_switch` accept shortcut
     for machine+process — `:2625-2627` guards the `else` arm).
- `source.rs` — `ProfileInput.load_names` already carries the
  descriptors; add `upward_compatible_printers: &'a [String]` to
  `LoadedPresetNames` (populated from `CliUpwardMachine::select`'s
  report, see wiring).
- `profiles/entry.rs` — populate the upward vector: the composite path
  must wire `CliUpwardMachine{source,input:UpToDateFiles}` the same way
  `main.rs:178-184` wires `FetchOriginal` for the plain path, with
  `UpToDateFiles(request.settings)`. *Wiring detail*: the report
  currently lands after adjudication inside `prepare_project_source`;
  for load-settings slices the select must run before/inside
  `prepare_profile_slice`'s gate. Keep the same `select` code path —
  no parallel parser.
- `SliceError` gains `NewMachineNotSupported` (variant only; the CLI
  family row is in `failure.rs`, see Dependencies).

## Dependencies (boundary, do not implement)

1. **Partial-list composition** — machine-only or process-only
   `--load-settings` lists must compose before branches 2/3 are
   reachable (`ProfileSelection::new` currently requires all three).
   Once merged, the two archived branch-2 REDs are restored + GREEN.
2. **CLI `failure.rs`** — family row mapping
   `SliceError::NewMachineNotSupported` → `-16`/240 with
   `cli_errors[CLI_3MF_NEW_MACHINE_NOT_SUPPORTED]` stderr and
   `record_exit_reson` JSON shape; plus test-runner
   `EarlyExit::NewMachineNotSupported` (test-side enum is in the shared
   runner cache module; flagged, not assumed).
3. **inherits parent-resolution gap** — belongs to the `Catalog`/`parse_file`
   boundary. Not touched here: fixtures stay flattened.

## Test plan (RED → GREEN)

Fixture reuse: `tests/parity/validation/cli-exit/compat17/` —
`process-not-compatible.3mf` (embedded printer = Ender-3 0.4 nozzle,
embedded `upward_compatible_machine` currently empty).

- `branch3_reject` — process-only list (`process-compatible-excluding`):
  `new_pcp` excludes embedded system name → `-17` (239). *Blocked on
  dependency 1.*
- `branch3_accept` — process-only list (`process-compatible-including`)
  → slices; banner-normalized whole-file vs native.
- `branch2_reject`/`branch2_machine_switch_accept` — restore from the
  archived RED file verbatim; machine-switch accept asserts whole-file
  bytes *only if* the partial-list output slice (`:2597-2606` rename, `:3050-3051`
  append) is on main; otherwise assert exit 0 + nonempty gcode and
  defer byte-assertion.
- `branch1_switch_reject` — machine+process list: machine names a
  different printer, process `compatible_printers` excludes the new
  system name, upward vector empty → `-17` (239). Guards the merged
  over-accept bug in the Correction section.
- `upward16_reject` — machine+process list: machine file names a
  different printer AND the process file's `compatible_printers`
  excludes `new_printer_system_name` (branch verdict incompatible)
  AND `upward_compatible_machine` is non-empty but excludes the new
  system name → `-16` (240). Machine-only `-16` is impossible per
  `:2625-2627`. *Blocked on dependencies 1+2.*
- `upward16_accept` — same shape but `upward_compatible_machine`
  lists the new system name → `machine_upwards` + compatible, exit 0.
- `upward16_machine_only` — machine-only file with a non-empty
  upward vector that excludes the new system name → still exit 0
  (`:2625-2627` sets compatible before the upward scan; upward hit
  only flips `machine_upwards`, `:2633`).
- Baseline re-assert: embedded-nonempty case already covered by
  `cli_upward_machine_result.rs` tests — reuse
  `UpwardMachineInput::UpToDateFiles`, no new parsing.

## Deferred

- `machine_upwards` downstream effects (`OrcaSlicer.cpp` post-gate
  consumers of the flag and the `:2659-2680` `new_preset` rewrite /
  `:3050-3051` append / `:3056-3070` slot-0 marker) — the
  partial-list slice; the adjudication may compute the flag but does
  not emit those outputs.
- `-16` `-15` neighboring families (e.g. `CLI_CONFIG_FILE_ERROR`
  inside the `UpToDateFiles` loop) — already owned by the
  `upward_machine.rs` surface; reused, not re-implemented.

## Controls

- No `dbg!`/`println!`, no `#[ignore]`; REDs must compile and fail on
  the runtime comparison runner (`load_settings_early_reference` with
  the new `EarlyExit` variant).
- No edits to the `composition`/`load_settings` domain,
  `failure.rs` family table, or CLI adapter arg handling (the
  `CliUpwardMachine` wiring in `profiles/entry.rs` is core-side).
- `-17` channel byte-shape stays `record_exit_reson`-identical; `-16`
  follows `cli_errors[CLI_3MF_NEW_MACHINE_NOT_SUPPORTED]`.
- File ≤ 400 LOC; clippy `-D warnings`, fmt, wasm32 check,
  directed + workspace + four-leg full.
