# Proposed CLI -100 custom-G-code exception rewrite slice

Base main1a8c92f55; prior six-family/-102 slice merged before starting this work.
Status APPROVED by coordinator: PlaceholderParserError GCode.cpp:3640-3645 to
OrcaSlicer.cpp:6286-6291, stderr is the complete ex.what() message.
Fresh captures /tmp/worker4-100-native-UfhU0J confirm all five Unix156, exact
channels/result.json, and only result.json exported (no final G-code).
Tracked evidence: tests/parity/validation/cli-exit/custom-gcode/*.native-receipt.json.

## Upstream and Rust destination

- Utils.hpp:72 CLI_SLICING_ERROR=-100; OrcaSlicer.cpp:156 generic JSON message.
- OrcaSlicer.cpp:6286-6291 per-plate slicing/export std::exception catch prints
  ex.what()+newline, record_exit_reson(index+1, generic cli_errors[-100]), then
  flush_and_exit(-100). Existing record_exit_reson:416-463 and flush:396-413.
- GCode.cpp:3637-3645 failed_templates throw;3920-3933 collection through
  placeholder_parser_process. Ares already ports aggregation in
  crates/ares-core/src/project_slice/gcode_emit.rs:348-361 and exact parser
  details in gcode_emit/template/error.rs. This slice replaces only its generic
  InvalidInput compatibility shell with a typed custom-G-code failure variant.
- Rust CLI destination crates/ares-cli/src/failure.rs exhaustive typed -100 arm;
  Linux metadata serialization and platform-specific stdout/negative exit reused.

## Included / deferred

Included only collected custom-template failures (five worker3 stock witnesses:
FLSun QQ-S Pro and Ginger G1 1.2/3.0/5.0/8.0). Preserve exact name/line/caret/text
and ordering; no parser changes, string-matched classification or broad catchall.
New typed variant carries detail plus same applied metadata fields/types as
Validate/-102. Producer uses already-applied traversal config and existing
state::first_plate_id on selected view, same f32 widening, no reload/snapshot
helper copy/stream parsing. Core stays filesystem/terminal-free on all Tier1.
CLI emits native Unix156, exact stdout, exact detailed stderr with trailing
newline, and exact Linux JSON with GENERIC error_string from cpp:156, plate and
applied configuration. No new dynamic Value/allowlist/lint suppression/fallback.
Preserve original six families, -102 whole retained artifact, legacy Debug
STL/other errors, -2 pre-read override hook and platform cfg contract.
Defer other upstream std::exception causes (geometry/export/timing), success JSON,
stock template expansion fixes and any unrelated core/parser behavior.

## RED / GREEN / evidence contract

Historical diagnosis /tmp/worker3-custom-errors-879/report.json: five Native156
versus Ares1, decoded Ares detailed parser text EXACTLY equals Native text;
historical baseline879163bac is not substituted for new source execution.
First confirm all five inputs/resource/oracle identities and take fresh Native
seed0/single-CPU captures using existing binary (no diagnostic Orca rebuild).
Missing input/resources/seed or dual-oracle ambiguity is setup-invalid, not RED.
Write CLI process E2E before production; exact exit/stdout/stderr/whole JSON and
absence of final exported G-code (confirm fresh Native artifact state first).
Preserve independent RED commit and real runtime evidence, then independent
GREEN typed producer/mapping. If test requires compiling new enum before arm,
report structural RED honestly without a placeholder or substituted binary.
Run directed CLI/core affected controls, fmt/clippy-Dwarnings and actual CLI
wasm32 check. Fresh paired full seeded/official per latest coordinator contract;
16threads/buildjobs2/incremental0, exclusive target and fresh separate caches,
serial sole heavy job. Same-label Ares RAW bytes/candidate-only failures and
Native generator-only ND receipts required; no pass inferred from counts.
Latest main rebase/gate scope, exact-hash REVIEW-REQUEST, approval then ff-only.
