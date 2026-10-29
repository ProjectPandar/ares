# D-A: admitted negative anchor projection public RED

Approved SPEC aa7645f6a, initially carried onto dcbd3a084 as d2a3e5a39.
After D-M merged, clean carry onto main73e8e712e produced SPEC6c51a8d18;
range-diff1/1 '='. Restored public RED Rust files are byte-identical to the
interrupted source snapshot. Production is exactly main73e8e712e.

Source owners: Fill.cpp::group_fills:983-989 narrows/projects/mins anchors;
Layer::generate_sparse_infill_polylines_for_anchoring:1477-1482 passes those
lengths into FillParams. PrintObject.cpp::region_config_from_model_volume
:3664-3709 merges local overrides. FillBase.cpp::connect_infill:1580-1587 /
connect_base_support:2247-2252 assertions alone did not establish release
admission. Actual Native acceptance below supplies that evidence.

## Native-first acceptance and repeatability

/tmp/worker2-da-preflight.py used the existing solid-ceiling exported fixture,
ordinary global anchor400%/max20, and local object/part anchor=-1/-50%
controls paired with -1e39/-1e39% extremes. Non-metadata ZIP payloads remain
unchanged. Both modes accepted all8 cases, exit0 with complete footer,
52 positive Internal Bridge and378 Sparse infill extrusion moves each.
Fresh repeat8/8 per mode had identical inputs and full Native streams except
one canonical generator timestamp. Total32 preflight invocations completed.

| Mode | First root | Fresh repeat root |
|---|---|---|
| official | /tmp/worker2-da-preflight-official.a5i3y1wx | /tmp/worker2-da-preflight-official.bpitewep |
| seeded | /tmp/worker2-da-preflight-seeded.f2mdmkk3 | /tmp/worker2-da-preflight-seeded.7zsoyoe3 |

These completed before FULL4. After main advanced, recorded wrapper/ELF SHA
values were checked against the current files and remained identical. This
retains completed Native admission evidence; it does not salvage the aborted
public batch below or claim a runtime result from its build interruption.

## Public RED on new main, separate from any repair

New runner/compatible_regions/bridge_anchor_length_projection.rs reuses the
existing parent fixture module. Four tests cover object/part × absolute/
percent. Each first verifies ordinary global anchors/max, finite negative
public f64 carrier, and a finite-control versus overflowing-f32 distinction.
It compares finite negative control before the extreme, preserving strict
whole Native/API/CLI parity assertions and raw API/CLI equality.

Driver /tmp/worker2-da-red-main73-leg.sh, outer taskset officialCPU13 /
seededCPU14, jobs2/threads2, independent fresh target/cache/root. Filter:
test(runner::compatible_regions::solid_pattern::anchor_length_projection::).

| Mode | Root | Nextest result | Comparison cases |
|---|---|---|---|
| official | /tmp/worker2-da-red-main73-official.KWsyYE | 4expectedFAIL, exit100 | 4controlsPASS +4exact guard errors |
| seeded | /tmp/worker2-da-red-main73-seeded.X3oG6T | 4expectedFAIL, exit100 | 4controlsPASS +4exact guard errors |

All16 public Native cases completed with footer and52bridge/378sparse moves.
Each extreme fails the line96 PASS assertion: ARES_ERROR, exact
"unsupported project feature: infill_anchor"; CLI exits1 with that same key.
The effective maximum stays finite20; D-M cannot mask this witness. Native
rejection/crash/timeout or missing roles are not counted as a RED.

Offline reread verified input/native hashes, ordinary global options,
unchanged members, all4 control whole-stream comparisons per mode and raw
API/CLI equality. /tmp/worker2-da-red-evidence.json and verifier
/tmp/worker2-da-red-verify.py retain the exact case directories and receipts.
No Native products are committed.

## Gates, interruption and next boundary

fmt and test-target strict clippy exit0; /tmp/worker2-da-red-main73-gates.lDGDCh:
- cargo fmt --all -- --check
- cargo clippy -p ares-cli --test orca_parity -- -D warnings

FULL4 interrupted original public batch sessions11784/48809 and gate60835;
roots /tmp/worker2-da-red-official.B5o9ky and
/tmp/worker2-da-red-seeded.yRlYNj are marked VOID, with no valid public result.
No old partial result is included above. All resumed runs used fresh roots.
All touched files <=400 lines. No production, D-M assertion, comparator,
retry, exception or schema changes. No full/workspace or push.
Await RED review/repair approval before changing the remaining guard.
