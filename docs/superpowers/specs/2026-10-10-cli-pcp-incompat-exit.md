# CLI 3MF `print_compatible_printers` incompatibility rejection (`-17 CLI_PROCESS_NOT_COMPATIBLE`)

Status: proposed. Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Problem

A plain CLI slice of a 3MF whose embedded process declares
`print_compatible_printers` that does not contain the current printer's
system name is rejected by Native with `-17 CLI_PROCESS_NOT_COMPATIBLE`
before any slicing. Ares currently slices it and exits 0.

Admission evidence (worker1 handoff, `${TMP_ROOT}/ares-pcp-embedded/admission`):
`native-slice/command.txt` runs `--slice 0 --outputdir <dir> input.3mf`;
products are `exit 239` (≡ `-17` mod 256), stdout
`run found error, return -17, exit...\n`, empty stderr, the sorted-key
`result.json` below, and no G-code:

```json
{
	"error_string": "The selected printer is not compatible with the process preset in the 3mf.",
	"export_time": 0,
	"layer_height": 0.0,
	"plate_index": 0,
	"prepare_time": 0,
	"return_code": -17,
	"sparse_infill_density": 0.0,
	"wall_loops": 0
}
```

`input-receipt.json` records the input identity (members + SHA256);
`input.3mf` is the pinned fixture source.

## Upstream boundary (source-cited, pin `8500fcdcc`)

- `OrcaSlicer.cpp:1687-1770` — 3MF load: `current_printer_name` /
  `current_process_name` from `printer_settings_id` / `print_settings_id`;
  `inherits_group` layout `[process, filaments…, printer]`
  (`[0]` → process, `[size-1]` → printer, middle → filaments); a missing or
  empty slot falls back to the preset's own name (":" no-inherits comment,
  `:1737-1753`); `current_print_compatible_printers` = the embedded
  `print_compatible_printers` option values (`:1770`).
- `OrcaSlicer.cpp:2562-2620` — the four-branch `process_compatible`
  adjudication. A plain CLI slice (no `--load-settings`, no machine/process
  switch) takes the fourth branch (`:2608-2620`): compatible iff
  `current_print_compatible_printers` is empty (old-3mf rule) or contains
  `current_printer_system_name`.
- `OrcaSlicer.cpp:2652-2654` + `Utils.hpp:42` + `cli_errors`
  (`OrcaSlicer.cpp:111-129`) — `record_exit_reson` + `flush_and_exit(-17)`
  with error string `The selected printer is not compatible with the
  process preset in the 3mf.`.
- `OrcaSlicer.cpp:396-460` — `flush_and_exit` / `record_exit_reson`
  channels; already ported in `ares-cli/src/failure.rs`
  (`FamilyFailure`, exhaustively mapped `Utils.hpp:24-59`).

## Rust destination

- `ares-core`: a typed `SliceError::ProcessNotCompatible` variant and a
  load-time gate in the plain-3MF slice path
  (`project_slice::slice_source` with `profile: None`, i.e. the path
  `slice_project_plate*` feeds): after the embedded project loads and
  before any slicing, read the embedded runtime preset's
  `printer_settings_id`, `print_settings_id`, `inherits_group`, and
  `print_compatible_printers` (all already registry options in
  `ares-core`), apply the fourth-branch semantics above, and reject.
  The gate is a read of already-parsed embedded values; no 3MF
  serialization or write path changes.
- `ares-cli`: `failure.rs` family entry for `-17` — error string above,
  empty stderr, existing `flush_and_exit` / `result.json` shape
  (POSIX exit `239`).

## Single E2E RED

`crates/ares-cli/tests/orca_parity/cli_exit_channels.rs`: new
`EarlyExit::ProcessNotCompatible` (`code() == -17`) in
`runner/cache/cli_result.rs` and a new case
`cli_exit_process_compatible_family_matches_native` running
`assert_family("process-not-compatible", ProcessNotCompatible)` with the
fixture `tests/parity/validation/cli-exit/process-not-compatible.3mf`
(bytes pinned from `${TMP_ROOT}/ares-pcp-embedded/admission/input.3mf`).
RED = Ares exits 0 and exports G-code while Native exits 239 with the
channels above. GREEN = exit byte, stdout, stderr, `result.json`
(`result_comparison_bytes`) and no-G-code all identical.

## Included

- Fourth-branch adjudication only (plain 3MF CLI slice).
- Empty `print_compatible_printers` ⇒ compatible (upstream old-3mf rule,
  `:2614-2618`).
- System-name fallback: empty `inherits_group` slot ⇒ preset's own name.
- Exact channel bytes: stdout line, empty stderr, sorted-key
  `result.json`, POSIX exit `239`, no G-code export.
- Fixture pinned to the admission input (identity in `input-receipt.json`).

## Deferred

- Branches 1–3 of `:2562-2606` (new printer / new process / both): Ares
  CLI has no machine/process switch affordances; nothing can reach them.
- `upward_compatible_machine` refetch and the `-15`/`-16` paths
  (`:2515-2538`, `:2622-2650`): BBL `machine_full`/`process_full`
  resource lookups.
- `:2410-2432` load-settings-side compatible refetch: requires
  3MF+`--load-settings`, which the CLI currently rejects outright.
- `:2656+` project-embedded preset creation on machine switch.
- GUI-side `is_compatible_with` filtering (`Preset.cpp:801-840`): already
  ported for catalog composition in
  `ares-core/src/profiles/compatible.rs`; not a CLI concern here.

## Boundary with worker1 (PCP serialization fidelity)

Worker1 owns writing `print_compatible_printers` into 3MF serialization
fidelity. This slice only **reads** the embedded
`print_compatible_printers` / `inherits_group` values at load time and
contains no serialization, export, or `orca_block_keys` changes. If the
RED surfaces a serialization defect in the fixture path, it is reported
to worker1 rather than fixed here.

## Controls

- No `dbg!`/`println!` probing, no `#[ignore]`, no `allow`/`expect`
  suppressions, no `UnsupportedProjectFeature` guard, no assertion
  weakening; the RED must compile.
- Native crash cases stay documentation-only (not applicable here: the
  reference path is a clean `-17` exit).
- One commit per item: RED first, then GREEN.
