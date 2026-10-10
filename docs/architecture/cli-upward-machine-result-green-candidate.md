# Upward result metadata — implementation candidate

Status: **implementation candidate, not verified GREEN**. Owner requested
consolidation of local WIP `7bc55ddc4` and candidate `3469dedde` after early
feedback. No check, Clippy, fmt or Native execution was performed during
worker5's exclusive four-leg full.

## Upstream boundary

Continue the approved `cli-compatible-machine-result-metadata.md` SPEC at
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

No worker3 grouping/master-extruder implementation is imported. Existing core
byte/profile APIs keep their old report-free behavior. Core remains usable on
WASM, Windows, macOS and Linux; new filesystem/result behavior remains Linux-only.

## Evidence and remaining gates

The approved independent RED test and its complete-channel assertions are
unchanged: `cli_custom_gcode_failure_preserves_nonempty_upward_machine_header`.
Its immutable input SHA is
`023008f8fb816bf57abe76c5421eb9ad63850c51af1a8276d18150c5358ce593`.
An offline ZIP inspection confirms BambuStudio application metadata, original
H2D identity and the embedded H2D Pro target; this inspection is **not GREEN**.

Owner rejected the implementation-following isolated source controls;
`ares-core/tests/upward_machine_sources.rs` is deleted. Those tests were never
executed and establish no RED/GREEN evidence. The bounded checkpoint uses only
the approved first CLI E2E witness. Four-branch coverage is deferred to subsequent
public CLI witnesses with complete runtime Native comparison, including original
versus selected identity, missing versus empty replacement, vector fidelity and
explicit update ordering. No isolated/unit-test substitute is planned.

The parent module's test-only stage imports are removed. Three existing geometry
tests now import the moved functions directly from `load_stages`; their existing
assertions and behavior are unchanged.

Deferred: public `up_config_to_date`/explicit updater/selected-machine overlay
orchestration (including process-source name checks), full Native loader-error
code/channel parity, and all downward discovery/geometry/check/report behavior.
Implementation of explicit/automatic source selection does not certify those
public CLI paths; they require the subsequent E2E witnesses above.
The existing first E2E witness alone does not certify additional source controls.

After worker5's four-leg END and owner scheduling permits, run check, strict
Clippy and fmt, then fresh directed seeded and official Native
comparisons at admitted actual-start load. Retain any compilation/runtime
failures without relabelling them RED/GREEN. Exact-hash review/integration remains
separate; no push is authorized by this candidate record.
