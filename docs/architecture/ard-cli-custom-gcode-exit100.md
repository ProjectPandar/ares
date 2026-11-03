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

## 2026-10-08 wall-clock comparison finding (seam prerequisite gates)

The independent seam-radius candidate on production base `083cb1862` retained
strict workspace exit **100**: 8131 PASS, 1 FAIL, 7 configured skips. The failed
`cli_custom_gcode_ginger_8_0_matches_native` JSON differed only in
`prepare_time`: Native **1**, Ares **0**. All other JSON fields, stdout, stderr,
Unix156 and absence of final G-code retained their strict contract.

At upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer.cpp:5540` computes prepare_time from the difference of two UTC
wall-clock second readings, serialized at :436. Export_time similarly uses UTC
seconds at :7104 and is serialized at :437. Neither field is a seeded slicing
result or a seam-visibility value.

Fresh isolated reruns were exactly three on baseline `083cb1862` and three on
candidate `0d0b6f5d333dc32d4545f62c789645a9e3ce7ddd` (rebased onto test-only
`eddfff507`). All six PASS with Native prepare_time/export_time **0/0**;
no baseline instability was observed in these three reruns. This does not erase
or turn the original strict workspace failure into PASS. The original loaded
workspace Native value **1** versus fresh Native **0** and the upstream UTC
producer establish the wall-clock dependence; coordinator explicitly ruled it
non-blocking for the separate seam REVIEW, not a production-radius regression.

Six attempts, fresh caches, owned deadlines, raw JSON and receipts remain in
`${TMP_ROOT}/ares-seam-radius/ginger-nd-edd/`; audit SHA256
`9c4e130eeeceeef7b39cacf8bdc749a94b79f735266cc64bf4910f14c30d0407`.
Original strict gate records remain `${TMP_ROOT}/ares-seam-radius/green-083/`.
A separately authorized test-only slice will normalize only prepare_time and
export_time numeric values during the CLI JSON comparison, preserving every
other byte and every other assertion. This finding does not implement that
normalization and does not authorize ignoring unrelated failures.

## Independent test-only wall-clock comparison slice

Authorized scope: `OrcaSlicer.cpp:5540/436` prepare_time and :7104/437
export_time are UTC second differences serialized as unsigned integers.
Only `crates/ares-cli/tests/orca_parity/cli_custom_gcode_exit.rs` comparison
bytes change; production JSON, raw retained artifacts and Native cache receipts
remain untouched. Replace only those two integer lexemes with zero on each side,
not a parsed/re-serialized JSON object, so every other byte stays observable.
Keep raw cache-hit equality, Unix156, complete stdout/stderr and no final G-code
assertions unchanged. No broader generator/timing/config normalization.

Isolated comparator failure modes, specified before test implementation:
clock-only differences still fail; a non-clock value difference becomes equal;
JSON whitespace/order differences disappear through re-serialization; replacement
accidentally touches error text or another key; cache/native/CLI channel assertions
are weakened. Deterministic synthetic byte contracts must catch the first three;
review the exact replacement boundary and retain fresh five-case runtime Native
E2E artifacts for the remaining channel/cache contract. Synthetic contract inputs
are handwritten test data, not frozen Native outputs. The initial extracted
comparison is identity (existing strict semantics), so the clock-only contract
must run RED before adding the two local replacements.
