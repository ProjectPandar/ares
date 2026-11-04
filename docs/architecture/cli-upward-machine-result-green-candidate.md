# Upward result metadata — implementation candidate

Reached upward metadata is stored without OS gating, and original-machine
resource loading uses component-wise Path::join on all native platforms; no
computation/resource-loading OS branch or Embedded downgrade exists. Existing
Linux-only upstream result-file emission is unchanged. Windows and macOS
runtime execution has not been performed.

## Upstream boundary

Continue the `cli-compatible-machine-result-metadata.md` SPEC at
OrcaSlicer pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `OrcaSlicer.cpp:1732–1769`: original system identity and embedded vector.
- `:2282–2301`, `:2368–2385`, `:2512–2536`: explicit refresh, original-machine
  refresh and fetch-only vector replacement. Missing file retains original
  values; a present valid file replaces them even when its vector is empty.
- `:1953–2017`: file metadata accepts `system`, `User` and `user`; unsupported
  metadata/failed parsing is not absence. Do not import downward's stricter
  machine/system condition into upward selection.
- `:2656`: report commit; `:428–432`: nonempty ordered JSON array only.

This is a source-cited metadata rewrite slice, not a new Ares pipeline.

## Rust destinations

- `ares-core/src/project/upward_machine.rs`: original 3MF identity/settings,
  typed coStrings normalization and four distinguishable source-selection
  inputs. The existing typed `ProfileFragment` parses supplied file bytes;
  production does not introduce `serde_json::Value` or filesystem access.
- `project_slice/source.rs::CliUpwardMachine`: byte/source inputs and borrowed
  report storage. `validated.rs::slice_cli_project_plate_validated` clears
  report storage before validation, including when reused by a caller.
- `project_slice/state.rs::prepare_project_source`: commits the vector after
  existing early load/post-process/config checks and before plate selection,
  `Print::validate` and G-code generation. Later failures retain that snapshot;
  failure serialization never reconstructs it from final printer settings.
- `ares-cli/src/compatible_machine.rs`: reads the original system's exact
  `profiles/BBL/machine_full/<name>.json` bytes. `--resources-dir` binds the
  resource root; default is `resources` beside the executable. No vendor search,
  Native installation lookup, inverse graph or hardcoded machine vocabulary.
- `ares-cli/src/failure.rs`: takes ownership of the reached report vector and
  emits nonempty arrays in sorted/tab-indented result JSON. Empty vectors remain
  omitted. Existing return/stdout/stderr/other JSON fields remain unchanged.
  Both ordinary and `--dump-moves` 3MF paths use the same report-aware entry.
- `project_slice/load_stages.rs` contains the unchanged initial geometry stage
  bodies moved out of the parent to keep touched Rust modules below 400 LOC.

No grouping/master-extruder implementation is imported. Existing core
byte/profile APIs keep their old report-free behavior. Core remains usable on
WASM, Windows, macOS and Linux. Original-machine filesystem loading runs on
all native CLI platforms; only existing upstream Linux result-file emission
remains platform-specific.

## Evidence and remaining gates

The independent RED test and its complete-channel assertions are
unchanged: `cli_custom_gcode_failure_preserves_nonempty_upward_machine_header`.
Its immutable input SHA is
`023008f8fb816bf57abe76c5421eb9ad63850c51af1a8276d18150c5358ce593`.
An offline ZIP inspection confirms BambuStudio application metadata, original
H2D identity and the embedded H2D Pro target; this inspection is **not GREEN**.

Implementation-following isolated source-control tests are not used. The
bounded checkpoint uses only the first CLI E2E witness. Four-branch coverage
is deferred to subsequent public CLI witnesses with complete runtime Native
comparison, including original versus selected identity, missing versus empty
replacement, vector fidelity and explicit update ordering. No isolated/unit-test
substitute is planned.

The parent module's test-only stage imports are removed. Three existing geometry
tests now import the moved functions directly from `load_stages`; their existing
assertions and behavior are unchanged.

Deferred: public `up_config_to_date`/explicit updater/selected-machine overlay
orchestration (including process-source name checks), full Native loader-error
code/channel parity, and all downward discovery/geometry/check/report behavior.
Implementation of explicit/automatic source selection does not certify those
public CLI paths; they require the subsequent E2E witnesses above.
The existing first E2E witness alone does not certify additional source controls.

## GREEN witness

Directed evidence uses direct cargo nextest:

```sh
cargo nextest run -p ares-cli --test orca_parity \
  cli_custom_gcode_failure_preserves_nonempty_upward_machine_header
```

Seeded and official both PASS and retain runtime Native -100/nonempty upward,
CLI156/-100, whole exact stdout/stderr/result JSON and no final G-code. Only
existing integer prepare_time/export_time adaptation is used; downward remains
absent. Each run compares its own exact input against live Native, not a frozen
RED output. The `-17` compatibility rejection occurs before upward report
assignment.

Standard full seeded and official legs pass with empty failure sets on both
BASE and CAND; the sole new test is
`cli_upward_machine_result::cli_custom_gcode_failure_preserves_nonempty_upward_machine_header`.
No `ARES_PARITY_OPTIONS` or option key filter is set: upward does not own an
option domain.

This is standard-full test/failure subset validation, not a full option scan,
cross-platform runtime certification or new source-branch coverage. Four upward
source-branch public CLI E2Es, loader-error parity and downward geometry remain
explicitly deferred.
