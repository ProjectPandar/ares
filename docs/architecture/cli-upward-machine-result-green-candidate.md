# Upward result metadata — implementation candidate

Status: **bounded first upward E2E GREEN; not reviewed/integrated**. Product
`905213f16b313cadafec45517f32aabc00d79309` is rebased onto main
`73e78279d766f8d500757204f505e1d43356b72a`. Owner requested consolidation
of local WIP `7bc55ddc4` and candidate `3469dedde`; rejected isolated tests are
absent. No gates ran during worker5's exclusive four-leg full.

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

## Fresh 73e GREEN receipts and remaining gates

Production main advance invalidated all d4 engineering gates:
`/tmp/ares-cli-upward/gates-b7/void-main-73e.json`. The incomplete d4 archive
was cancelled; it is not a completed archive or acceptance result. Compilation
and lint errors were retained and fixed without suppression. Rebase was clean;
worker5's -17 compatibility rejection is before upward report assignment.

Fresh workspace/all-target/all-feature `cargo check`, strict Clippy `-D warnings`,
fmt check and WASM-target check all exited 0. Logs and stage admissions:
`/tmp/ares-cli-upward/gates-73e/`. Build jobs were 2; check/strict actual-start
loads were below 30. This is not a workspace nextest/full-pair result.

Per owner direction, final directed evidence uses **direct** cargo nextest, not
an archive adapter:

```sh
cargo nextest run -p ares-cli --test orca_parity \
  cli_custom_gcode_failure_preserves_nonempty_upward_machine_header \
  --test-threads 2 --success-output immediate
```

- Seeded: `207e3ab1-10b3-4053-a55e-0aee4abbae49`, 1 PASS, exit0,
  actual-start load6.52. Root `/tmp/ares-cli-upward/green-73e-direct-seeded/`.
- Official: `87e6a036-2f9f-4b58-a5d3-c00ba7fd3c9c`, 1 PASS, exit0,
  actual-start load7.88. Root `/tmp/ares-cli-upward/green-73e-direct-official/`.

Both retain runtime Native -100/nonempty upward, CLI156/-100, whole exact
stdout/stderr/result JSON and no final G-code. Only existing integer
prepare_time/export_time adaptation is used; downward remains absent. Input
ZIP bytes and every member SHA, raw results and Native receipts are retained in
`/tmp/ares-cli-upward/green-73e-direct-audit.json`. Fresh exported ZIP SHAs differ
from the original RED; each run compares its own exact input against live Native,
not a frozen RED output. Original RED artifacts/assertions remain unchanged.

Earlier archive launches failed before test/Native execution, with exit2
argument parsing (including filter `-F` interpreted as features) and exit96
missing extraction directory. Original logs/outcomes remain in green-73e-*;
build-environment hypotheses were not acceptance evidence. Completed archive
seeded/official comparisons also passed but are separately inventoried in
`green-73e-audit.json`, not substituted for the owner-directed final runs.

Four-branch E2E, workspace nextest/full independent BASE/CAND controls and exact-
hash review/integration remain separate and require owner scheduling. No push
has been performed or authorized by these directed GREEN receipts.
