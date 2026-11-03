# CLI 3MF + `--load-settings`: `-17` adjudication branches 1–2 (loaded printer name)

Bounded follow-up slice to worker5's `2026-10-10-cli-3mf-load-settings.md`
(deferred item: "four-branch `-17` adjudication under load-settings,
branches 1–2"). Ports the upstream loaded-names semantics so the
`-17 CLI_PROCESS_NOT_COMPATIBLE` gate (`exit 239` on the CLI side) also
fires when a machine preset file comes in through `--load-settings`.

## Upstream boundary

`OrcaSlicer.cpp`, `main` load-settings path:

- `:2023-2032` — the `--load-settings`/`--load-filaments` file list is
  iterated after the 3MF block, so at adjudication time the `new_*`
  names are known.
- `:2034-2071` — a `machine`-type file sets `new_printer_name =
  config_name` (`:2039`); `new_printer_system_name` is the name itself
  for `config_from == "system"`, else the file's `inherits` value
  (`:2041-2046`). A second machine file exits
  `CLI_CONFIG_FILE_ERROR` (`:2035-2038`) — load-order/duplicate handling
  is worker5's CLI-adapter domain, not this slice.
- `:2073-2098` — a `process`-type file sets `new_process_name`,
  `new_process_system_name` (same system/inherits rule), and
  `new_print_compatible_printers =
  config.compatible_printers` (`:2090`).
- 3MF-embedded side (`:1704-1770`): `current_printer_name =
  printer_settings_id`, `current_process_name = print_settings_id`
  (`:1704-1707`); `inherits_group` supplies
  `current_process_system_name` (element 0, else
  `current_process_name`, `:1743-1750`) and `current_printer_system_name`
  (last element, else `current_printer_name`, `:1734-1741`);
  `current_print_compatible_printers =
  print_compatible_printers` (`:1770`).
- `:2513-2557` — for BBL 3MF (`is_bbl_3mf`),
  `current_print_compatible_printers` is **refetched** from
  `resources/profiles/BBL/process_full/<current_process_system_name>.json`
  → `compatible_printers` when `current_process_system_name` is
  non-empty and the file exists (`:2541-2557`); missing file keeps the
  embedded values.
- Four-branch adjudication (`:2575-2620`):
  - **Branch 1** (`:2575-2584`) — `!new_printer_name.empty() &&
    !new_process_name.empty()`: `process_compatible` iff
    `new_print_compatible_printers` contains
    `new_printer_system_name`.
  - **Branch 2** (`:2585-2594`) — `!new_printer_name.empty() &&
    new_process_name.empty()`: iff `current_print_compatible_printers`
    contains `new_printer_system_name`.
  - Branch 3 (`:2595-2604`) — only `new_process_name`: deferred.
  - Branch 4 (`:2605-2620`) — no loaded names: already ported by
    worker5's plain-3MF `-17` slice
    (`2026-10-10-cli-pcp-incompat-exit.md`,
    `project_slice/compatibility.rs::reject_incompatible_process`).
- Machine-switch relaxation (`:2624-2647`): when the branch did not
  mark compatible AND `!new_printer_name.empty() &&
  !current_printer_name.empty() && new_printer_name !=
  current_printer_name`, upstream sets `process_compatible = true`
  (and `machine_switch`) — with an `upward_compatible_printers` gate
  that can instead exit `-16` (`:2630-2643`). For **branch 1** this
  relaxation never applies (`new_process_name.empty()` required). For
  **branch 2** a `-17` verdict requires
  `new_printer_name == current_printer_name`; a different named machine
  file upstream machine-switches (accept, or `-16` when the upward
  list is non-empty and lacks the new system name).
- Rejection (`:2650-2654`): `record_exit_reson(outfile_dir,
  CLI_PROCESS_NOT_COMPATIBLE=-17, …); flush_and_exit(-17)` —
  `Utils.hpp:42`; CLI-visible as exit `239`, stdout
  `run found error, return -17, exit...\n`, empty stderr, sorted-key
  `result.json` with
  `"error_string": "The selected printer is not compatible with the
  process preset in the 3mf."` and `"return_code": -17`
  (channels per worker5's `cli-pcp-incompat-exit` spec, same family).

## Ares destination

- `crates/ares-core/src/project_slice/compatibility.rs` — extend the
  existing `reject_incompatible_process` module (lands with worker5's
  plain-3MF slice; this slice must not recreate it on its own branch).
  Add the adjudication-input record the four branches consume:

  ```rust
  /// `:2562-2620` adjudication inputs, already resolved by the caller.
  pub(super) struct CompatAdjudication {
      current_printer_name: String,
      current_printer_system_name: String,
      current_print_compatible_printers: Vec<String>,
      new_printer_name: Option<String>,
      new_printer_system_name: Option<String>,
      new_process_name_present: bool,
      new_print_compatible_printers: Vec<String>,
  }
  ```

  plus `reject_incompatible_load_settings(&CompatAdjudication)
  -> Result<(), SliceError>` implementing branches 1–2 and the
  `new_printer_name == current_printer_name` gate on the
  machine-switch relaxation for branch 2. Existing
  `SliceError::ProcessNotCompatible` is reused; the `-17` failure
  family in `ares-cli/failure.rs` is worker5's, untouched here.

- Inputs derivation lives in `compatibility.rs` too: a narrow
  `PresetDescriptor { name, system_name, compatible_printers }`
  sourced from worker5's composed load-settings metadata
  (`composed`/`ProfileInput` — the composition API is worker5's
  boundary; this slice only reads the descriptors it already
  produces). Embedded-side values keep using
  `project.documents().project_settings_raw` with the existing
  `printer_system_name` / `text_list` helpers extended for
  `print_settings_id` + `inherits_group[0]` when needed.
- `project_slice/state.rs` — the existing
  `reject_incompatible_process` call site gains the composite dispatch:
  profile-less path keeps the branch-4 function; when the source
  carries load-settings descriptors, route to the new adjudication.
  No CLI-adapter or file-ordering change (worker5 boundary).

## Included

- Branch 1 adjudication (machine + process files loaded):
  `new_print_compatible_printers ∋ new_printer_system_name`.
- Branch 2 adjudication (machine file only): `new_printer_name ==
  current_printer_name` required for a `-17` verdict;
  `current_print_compatible_printers` = embedded
  `print_compatible_printers`, with the `:2541-2557` system-process
  refetch applied when the composed catalog exposes a `process_full`
  entry for `current_process_system_name` (fallback: embedded list,
  same as upstream's missing-file path).
- The `:2624-2647` relaxation only insofar as it gates branch 2
  (`new_printer_name != current_printer_name` ⇒ compatible, no `-17`
  from this slice).
- Machine-switch `upward_compatible_printers` reads **reuse worker1's
  `project/upward_machine.rs`** once merged (owner addendum): this
  slice adds no second parser; until that module exists the witness
  fixtures simply avoid the non-empty upward path (see case 4).
- E2E RED witness: same argv on Native vs Ares, all channels
  (`exit/stdout/stderr/result.json`):
  1. **Branch-1 reject**: BBL 3MF + `--load-settings` list = one
     machine file + one process file whose `compatible_printers`
     excludes the machine's system name → `-17` family on both sides.
  2. **Branch-1 accept**: same pair with `compatible_printers`
     containing the system name → both proceed (whole-output or
     family-only comparison per runner support).
  3. **Branch-2 reject**: `--load-settings` with a machine file whose
     `name` equals the embedded `printer_settings_id` while the
     effective `current_print_compatible_printers` (post-refetch rule)
     excludes the system name → `-17` both sides.
  4. **Branch-2 machine-switch accept**: machine file with a different
     name than embedded → both accept (upstream `machine_switch`;
     fixtures chosen so `upward_compatible_printers` is empty or
     contains the new system name, keeping the deferred `-16` path out
     of the witness — the list itself comes from worker1's
     `project/upward_machine.rs` reader, not a new parse here).
- Native exit/channel fixtures generated at runtime; only input
  fixtures (crafted 3MF + preset JSONs) are committed.

## Deferred

- Branch 3 (`new_process_name` only, no machine file):
  `new_print_compatible_printers ∋ current_printer_system_name`.
- `-16 CLI_3MF_NEW_MACHINE_NOT_SUPPORTED` / `-15` machine-change
  upward paths (`:2630-2648`), incl. the `machine_upwards` flag and
  `sliced_info.upward_machines` bookkeeping (`:2655`).
- `new_preset` compatible-printers rewrite for `machine_switch`
  exports (`:2659-2690`) — export-path concern, not CLI slice gating.
- CLI adapter `--load-settings` file list parsing, duplicate-file
  `CLI_CONFIG_FILE_ERROR`, and read-ordering vs the 3MF block —
  worker5 boundary (`OrcaSlicer.cpp:1953-2024`, `:1682-1690`).
- `-18/-19/-51` validation family interactions; unchanged.

## Deferred parity gaps (measured 2026-10-10)

- **`inherits` resolution in `--load-settings` files (worker5
  boundary).** Native `load_config_file` reads the `inherits` field
  directly as `new_printer_system_name` (`OrcaSlicer.cpp:2081-2086`)
  without resolving a preset parent; Ares `parse_file`→`Catalog`
  requires a resolvable parent, so an unflattened machine file
  (`from:"user"`, `inherits:"<system name>"`) exits with
  `InvalidInput("profile inheritance cycle")` where Native adjudicates
  `-17` (measured: Native 239 vs Ares non-239 on
  `machine-inherits.json`, probe `/tmp/compat17-probe/`). Compat17
  fixtures ship flattened (`inherits:""`, parents inlined) so the
  adjudication itself is covered; the gap stays worker5's
  composite/catalog slice. Recorded in `${TMP_ROOT}/ares-matrix-findings.md`.
- **`use_relative_e_distances:"0"` pin.** Measured redundant: Native
  still exits 239 with the key removed from the process clone
  (probe `/tmp/compat17-probe/B`). Kept as an explicit pin, no parity
  gap.

## Schedule

- Implementation (RED first) starts only after **both** worker5's
  3MF+load-settings slice and worker1's `project/upward_machine.rs`
  land on main. Before that, only non-Native artifacts may be
  committed (input fixture JSONs/3MF, no receipts).

## Controls

- No `dbg!`/`println!`, no `#[ignore]`, no assertion weakening; the
  RED must compile and run on the runtime comparison runner.
- No changes to worker5's `composition`/`load_settings` modules,
  `failure.rs` family table, or CLI arg handling.
- RED first: witness cases fail on Ares (today branch-1/2 inputs
  take the branch-4 path or pass silently) before production change.
