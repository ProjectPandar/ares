# Upward result metadata — implementation candidate

Status: **platform correction, workspace and standard full GREEN; REVIEW pending**.
Owner rejected product905213f16 for introduced OS gates/non-Linux Embedded
downgrade. The correction removes those conditions, uses component-wise
Path::join on all native platforms and stores reached upward metadata without
OS gating. Existing Linux-only upstream result-file emission is unchanged.
Product `de7fbc8e38e96c0f5627b62bcc77c0ee1a94f208` removes all added
target_os conditions relative to main73e; no computation/resource-loading OS
branch or Embedded downgrade remains. New check, strict Clippy, fmt and WASM
checks each exited0 (`${TMP_ROOT}/ares-cli-upward/platform-fix/`). Direct seeded and
official witness reruns each passed once, load9.18/9.46, threads2, preserving
whole channels/result and no-Gcode assertions. New full raw ZIP/member/result
and receipt audit: `${TMP_ROOT}/ares-cli-upward/green-platform-audit.json`. Windows and
macOS runtime execution was not performed on this Linux host. Workspace and
standard independent full controls subsequently passed; see the d5b acceptance
section below. The historical receipts
below describe the superseded product, not correction acceptance. Product
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
WASM, Windows, macOS and Linux. Original-machine filesystem loading runs on
all native CLI platforms; only existing upstream Linux result-file emission
remains platform-specific.

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
`${TMP_ROOT}/ares-cli-upward/gates-b7/void-main-73e.json`. The incomplete d4 archive
was cancelled; it is not a completed archive or acceptance result. Compilation
and lint errors were retained and fixed without suppression. Rebase was clean;
worker5's -17 compatibility rejection is before upward report assignment.

Fresh workspace/all-target/all-feature `cargo check`, strict Clippy `-D warnings`,
fmt check and WASM-target check all exited 0. Logs and stage admissions:
`${TMP_ROOT}/ares-cli-upward/gates-73e/`. Build jobs were 2; check/strict actual-start
loads were below 30. This is not a workspace nextest/full-pair result.

Per owner direction, final directed evidence uses **direct** cargo nextest, not
an archive adapter:

```sh
cargo nextest run -p ares-cli --test orca_parity \
  cli_custom_gcode_failure_preserves_nonempty_upward_machine_header \
  --test-threads 2 --success-output immediate
```

- Seeded: `207e3ab1-10b3-4053-a55e-0aee4abbae49`, 1 PASS, exit0,
  actual-start load6.52. Root `${TMP_ROOT}/ares-cli-upward/green-73e-direct-seeded/`.
- Official: `87e6a036-2f9f-4b58-a5d3-c00ba7fd3c9c`, 1 PASS, exit0,
  actual-start load7.88. Root `${TMP_ROOT}/ares-cli-upward/green-73e-direct-official/`.

Both retain runtime Native -100/nonempty upward, CLI156/-100, whole exact
stdout/stderr/result JSON and no final G-code. Only existing integer
prepare_time/export_time adaptation is used; downward remains absent. Input
ZIP bytes and every member SHA, raw results and Native receipts are retained in
`${TMP_ROOT}/ares-cli-upward/green-73e-direct-audit.json`. Fresh exported ZIP SHAs differ
from the original RED; each run compares its own exact input against live Native,
not a frozen RED output. Original RED artifacts/assertions remain unchanged.

Earlier archive launches failed before test/Native execution, with exit2
argument parsing (including filter `-F` interpreted as features) and exit96
missing extraction directory. Original logs/outcomes remain in green-73e-*;
build-environment hypotheses were not acceptance evidence. Completed archive
seeded/official comparisons also passed but are separately inventoried in
`green-73e-audit.json`, not substituted for the owner-directed final runs.

Four-branch E2E and exact-hash review/integration remain separate. No push has
been performed or authorized by these GREEN receipts.

## Standard full on d5b — review binding

- BASE: `d5b554b886e3f4d2b493b0bde0e61c212c7ec272`.
- Tested CAND/code+docs: `ebcfe03816609406beaf7106da433bb1cfb31231`.
- Final production-code commit: `23af32a4e989d622146a2f4cc01c8262f6b5dc12`
  (platform correction), above `b43e611e5a8e6350e3148a5137bf21ac0348e391`
  (staged upward metadata). This evidence update changes documentation only.
- Owner authorized carry of 73e engineering/directed gates to d5b, whose main
  advance contains only worker6 timelapse tests/docs. Clean rebase preserved
  production and approved witness assertions.
- Workspace run `69138d92-039a-47e6-b881-fed21b4611e4`: 8289 PASS,
  5 skipped, exit0; `${TMP_ROOT}/ares-cli-upward/workspace-de7-after-w6/`.
  The prior workspace interrupted for worker6 exclusive START remains VOID.

Four standard full legs finished using the owner-directed worker6 `run.py`
layout, jobs2/threads2, fresh per-leg Native caches and independent BASE/CAND
build targets. Both detached source trees bind the ignored OrcaSlicer source
resources to the verified upstream pin8500. No `ARES_PARITY_OPTIONS` or option
key filter is set: upward does not own an option domain. Actual d5b harness
counts are BASE922 and CAND923, not a filter forcing the older 919 count.

| Provider | BASE | CAND | Failure sets |
| --- | --- | --- | --- |
| Seeded | 922/922 PASS, exit0 | 923/923 PASS, exit0 | both empty |
| Official | 922/922 PASS, exit0 | 923/923 PASS, exit0 | both empty |

Evidence root: `${TMP_ROOT}/ares-cli-upward/standard-d5-resources.ViAmJ7/`.
`subset-audit.py`, `subset-audit.json`, `subset-audit.log` and exit0 verify the
exact named BASE test set is contained in CAND, all existing statuses remain
PASS, candidate-only failures are empty, and the sole new test is the approved
`cli_upward_machine_result::cli_custom_gcode_failure_preserves_nonempty_upward_machine_header`.
The audit records distinct BASE/CAND test and CLI binary SHAs, complete log SHAs
and run IDs. Both added live witnesses independently verify whole stdout/stderr
and result bytes (only existing integer times adapted), Native-100/Ares156,
ordered H2D Pro upward vector, absent downward, no final G-code, input ZIP/member
and all witness-file SHAs and Native receipts. No frozen RED output is used.

Infra attempts are preserved, never called parity RED/GREEN:

- `full-d5-fresh.Z8kZuF/infra-void.json`: custom archive driver expected the
  nonexistent `target/debug/ares-cli` binary; exit1, zero Native executions.
  Its pre-Native full650 configuration correction and script hashes remain.
- `standard-d5-worker6.qd94dU/infra-void.json`: interrupted seeded BASE lacked
  ignored source-resource binding. Eighteen partial Native receipt files were
  observed; zero Native execution is **not** claimed for this attempt. Owner
  authorized matching resource symlinks and a fresh restart. Partial output is
  retained separately, not substituted into final receipts.

This is completed standard-full test/failure subset validation, not a full650
option scan, cross-platform runtime certification or new source-branch coverage.
Cross-tree fixed-input raw-stream inventory beyond the named subset/witness
remains unaudited here. Four upward source-branch public CLI E2Es, loader-error
parity and downward geometry remain explicitly deferred. Review and integration
approval are still required; nothing has been pushed.
