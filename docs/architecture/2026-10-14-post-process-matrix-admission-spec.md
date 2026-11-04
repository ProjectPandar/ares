# Slice 5: post_process bounded CLI admission versus matrix baseline

Docs only; audit baseline 96328f269/f637b4122.
This narrows accounting for the existing
[CLI-boundary spec](option-post-process-cli-boundary-spec.md), not permission
to implement script execution or revive its deferred load-settings tracer.

## Upstream ownership and exact stage

- libslic3r/PrintConfig.cpp::PrintConfigDef::init_fff_params:5068-5079:
  post_process is coStrings with empty-vector default, not a finite enum.
- OrcaSlicer.cpp:1438,1466-1468 enables normative_check by default;
  :1682-1690 rejects an embedded nonempty vector by length, including [""],
  records CLI_POSTPROCESS_NOT_SUPPORTED/-19 and plate0, then exits before
  external settings composition at :1953-3495. Channels/result handling is
  :398-460. No whitespace/text-content policy may replace the length check.
- CLI external run_post_process_scripts call :6247-6248 is commented out.
  Actual external execution belongs to libslic3r/GCode/PostProcessor.cpp::
  run_post_process_scripts:238-377, with POSIX run_script:159-185 and Windows
  handling:63-153. That process/filesystem adapter boundary is deferred.
- GCode.cpp:2174 and GCode/GCodeProcessor.cpp:981,2685-2716 own a separate
  internal G-code statistics/progress pass; it is outside this script domain.

The existing validated-loading admission shell reflects this upstream CLI
phase. This slice accounts it; it does not design a new Ares script pipeline.

## Rust boundary and three reused cases

Own only crates/ares-cli/tests/orca_parity/option_coverage/post_process.rs,
its domains.rs/sweep.rs connection and narrow metadata/report wiring. Reuse
existing CLI runner, EarlyExit::Postprocess and result-channel comparator.
If a small new helper file is necessary, keep it under this test boundary
and each file <=400 lines. No production, core/WASM I/O or process changes.

| Existing witness | Native contract | Counted comparison kind |
|---|---|---|
| [] | CLI success, complete exported G-code equality | strict whole G-code |
| [""] | -19/Unix237, plate0, exact stdout/stderr/full normalized result JSON, no G-code | expected CLI admission rejection |
| [owned harmless absolute script path] | same rejection, absent execution marker before/after each slicer | expected CLI admission rejection/no execution |

Retain every existing assertion, the input-only option edit, all other ZIP
payloads, producer/command/exit/channel/result receipts and absent-marker
proofs. The existing third case creates an inert owned temporary script;
its actual path is generated per fresh root, not a stable golden value.
Declare that case's concrete requested vector in its application receipt.
Keep the nonempty script case distinct from an empty-text singleton.

Replace the None baseline with a finite **embedded ordinary CLI admission**
plan, not a claim of arbitrary coStrings or external execution coverage.
Reuse the three existing compare paths rather than rerunning separate
implementations or treating a rejected project as a sliced G-code stream.
Outcome credit requires actual successful contract comparisons: one full
G-code comparison and two exact expected-rejection comparisons. Publish
these separate counts/kinds in plan/outcome evidence. The matrix's generic
unexpected-rejection field must remain distinct from expected admission
comparisons; do not turn unexpected INPUT_REJECTED into PASS.

Proposed bounded-domain PASS means all three declared CLI contracts passed;
it does not mean three G-code comparisons. External script execution remains
explicitly deferred/uncovered in metadata and audit. This comparison-count
meaning must be settled before replacing baseline accounting.

## Public RED and GREEN

Submit a separate public matrix-accounting RED with original three Native
CLI tests as controls in official and seeded modes. The selected post_process
sweep must expose the three explicit admission cases and per-kind receipts;
the current single None baseline/INCOMPLETE cannot satisfy that expectation.
No fabricated production behavior RED: the controls already agree.

GREEN reuses those same cases in the plan/executor. Preserve individual tests,
exact diagnostic and full output comparators. Fail missing outputs/receipts,
wrong return/plate/channels, unexpected acceptance/rejection or a marker
appearing; stop and report rather than modify assertions or production.
If accounting needs shared outcome-format changes that affect other domains,
submit the exact additional diff separately first.

Directed runs use fresh roots/cache/targets. Required later gates: paired
directed including the three controls and option coverage; strict/fmt/wasm32.
No runs accompany this documentation commit.

## Deferred boundary

Defer whitespace/two-entry expansions beyond the existing three cases,
load-settings phase composition, normative_check=false, script evaluation,
quoting/env/permissions/order, rename/upload handling and Windows/macOS
process receipts. Linux cfg on existing CLI witnesses remains explicit;
no Linux result is reported as a platform-independent executable oracle.
The matrix must not silently skip this key on unsupported test platforms:
retain an explicit unresolved domain if these witnesses cannot execute.
Do not add runtime UnsupportedProjectFeature guards or independent flags.

Exit: bounded semantics, three consumed contracts with explicit
comparison kinds and truthful uncovered dimensions, source-cited metadata,
unchanged unrelated outcomes and comparators, independent code review.
