# Master extruder: bounded native-witnessed option coverage

Status: finite harness GREEN, pending review/CI and merge authorization.
Base: `96328f269d64165c02296ba9924f5c6ee972777c`.
Approved boundary: [slices2/4 SPEC](option-master-filament-self-index-bounded-spec.md)
(`dc2428a61`, clean carry `92860e611`). This conclusion covers slice2 only;
filament_self_index and its existing negative expectation remain unchanged.

## Source and behavior

PrintConfig.cpp:5399-5403 defines master1 without an arbitrary safe integer
range. ToolOrdering.cpp:1150 converts it to zero-based; :1165-1201 selects
used TPU; FilamentGroup.cpp:40-49 assigns TPU to master and PLA opposite.
ToolOrdering.cpp:1288-1308 remaps Auto and bypasses Manual. No FilamentGroup
optimizer or GCode.hpp:104-107 wipe-tower offset port is claimed.

The existing tool_context H2D input supplies two0.4mm Standard nozzles,
one by-layer cube10, wall1/infill2, no tower/support/raft. Its runtime Native
export is authored only to set TPU/PLA types, legal printability masks,
master1/2 and explicit project/plate Auto or Manual mode before either
slicer consumes it. Paired1/2 input members/settings differ only in master;
geometry, placement and all other ZIP member bytes remain exact.

| Case | Request | Native actual wall/infill map | Public result in each mode |
|---|---|---|---|
| M1 | master1, Auto For Flush | [1,2] | accepted, API/CLI whole bytes equal |
| M2 | master2, Auto For Flush | [2,1] | accepted, API/CLI whole bytes equal |
| MC1 | master1, Manual [1,2] | [1,2] | accepted, API/CLI whole bytes equal |
| MC2 | master2, Manual [1,2] | [1,2] | accepted, API/CLI whole bytes equal |

Actual wall and sparse XY extrusion follow the rendered G4_ACTIVE map,
using both logical filaments. Auto first rendered witness is line1139
F0N1 versus F0N2. Beyond markers, first actual command change is Native
line1186 `G1 E-.4 F1800` versus line1187 `G1 X178.6 Y164.564 E-.4` in both
modes. Manual paired motion remains byte-identical outside CONFIG_BLOCK.
These are observation-only consumer proofs; the whole-file comparisons
retain every command/config/footer byte except existing generator identity
normalization. No Native hash is rebound and no file is frozen as an oracle.

## RED and GREEN

RED `e0d317e8f`: runtime Native accepts four cases per mode; validated public
API plus real CLI each match complete ordered Native bytes and line counts.
Only the final key-local completeness assertion fails: expected PASS,
actual INCOMPLETE, original one None baseline. Both nextest legs exit100:

- official: `/tmp/worker6-master-red.PRHUmE/official`;
- seeded: `/tmp/worker6-master-red.0rPajp/seeded`;
- independent reread: `/tmp/worker6-master-red-e0d317e8f-audit.json`.

Earlier compile101 at master-red.P8Fcxr and resource-mount failure in the
first master-red.0rPajp official leg are invalid trials, not RED evidence.
The new worktree initially lacked upstream profiles; the same read-only
upstream symlink as the original workspace was restored before accepted
seeded Native execution, and official reran with fresh cache/artifacts.
No Native refusal, request substitution or Ares runtime difference occurred.

Separate GREEN `fd15b3ea2` adds38 harness lines: finite M1/M2 domain metadata,
executor and key-specific domains/sweep hooks. No production line changes.
The complete original RED test/comparator text is unchanged, SHA256
`27e23fb1adbcb93712de0d9a458873aa93d918107bed5e64b3050c31a23a5a0c`;
input helper also unchanged. Exact proof: assertion-preservation.json below.
No existing assertion reversal/removal is included.

## Final directed and engineering gates

Exact tested code HEAD `fd15b3ea2879b1531ebf092de17f66dab7b1d704`.
Root `/tmp/worker6-master-green.CmYW1O`, official and seeded serial,
outertaskset CPU11, jobs1/threads2, independent fresh Native cache/artifacts
and targets. Only compiled dependencies were copied; CLI/test fingerprints
were rebuilt for each new target path. Both complete option_coverage groups
include the non-Native tests; explicit ARES_PARITY_OPTIONS=1 and
ARES_OPTION_KEYS=master_extruder_id run this key's real matrix, not a skip.

| Gate | Result |
|---|---|
| official complete group | 223/223 PASS, exit0,761.563s |
| seeded complete group | 223/223 PASS, exit0,1044.772s |
| matrix in each mode | 1/1 domain PASS,2 generated/2 strictly compared/0 rejected |
| master captures reread | 12 Native/API/CLI full comparisons equal; line counts equal; validated API raw unchanged from RED |
| plan fingerprints | all651 keys retained; other650 plans exact in both modes |
| common raw against approved option audit | official376/376, seeded384/384 same-input Ares raw unchanged; zero unmatched/changed |
| actual Native executable affinity | 1312 recorded PID/exe/mask entries, CPU11 only, zero violations |
| strict Clippy / fmt / wasm32 | exit0 /0 /0 |

Evidence within the GREEN root: `{official,seeded}/{log,exit}`, their
artifacts/option-coverage-summary.md, master-capture-reread.json,
assertion-preservation.json, *-plan-invariance.json, *-common-raw-audit.json,
native-affinity.json. Plan/raw baseline is the accepted f637 option sweep
`/tmp/worker2-option-full-103wd7e0`; audit96328 adds docs only.
Strict is workspace/all-targets/all-features with -D warnings. wasm32 checks
core/vgcode/wasm. Standard-environment engineering logs and exit files:
`/tmp/worker6-master-engineering.m0U1RA`; no Native runs in that batch.
Rust files remain <=400 LOC. All Native outputs/receipts remain outside git.

Only H2D TPU Auto For Flush selector1/2 is closed. Manual is a negative
control, not all-mode coverage. Auto For Match, non-TPU/AMS optimization,
wipe tower offsets, invalid indices, SEMM and arbitrary machines defer.
No guards, fallback, ignore, suppression or comparator change was added.
CI workspace/full have not been run for this candidate and need coordinator
allocation. No push is authorized by these finite results; slice4 waits for
this slice's review. Its negative-expectation update remains a separate
commit conditioned on all four self-index Native witnesses in both modes.
