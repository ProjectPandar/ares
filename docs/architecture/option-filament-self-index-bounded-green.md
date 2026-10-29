# Filament self-index: bounded variant-slot coverage

Status: finite harness GREEN, pending review/CI and merge authorization.
Parent: reviewed master `72f51a6bee587b3f01ab39f5eb7133e6d86207d6`, main96328.
Approved [slice4 SPEC](option-master-filament-self-index-bounded-spec.md):
`dc2428a61` / carry92860; self-index expectation update authorized only after
all four Native cases in both modes, and required as a separate commit.

## Source-owned finite witnesses

PresetBundle.cpp:172-195 appends variant payloads and constructs complete
one-based self identities. PrintApply.cpp:1164-1172 materializes them;
PrintConfig.cpp:9004-9054 matches explicit IDs with variant strings,
:9805-9839 selects logical filament/physical nozzle, :9892-9910 selects
integer payloads. Initial/normal temperatures are in the variant set
(:8399-8400), consumed by GCode.cpp:870-871,1217-1220.
No production materializer or new diagnostic API was added.

The runtime G4 export provides H2D two0.4mm Standard nozzles, one by-layer
cube10, two used PLA logical filaments, Manual[1,2], no tower/support.
The input helper expands every existing exported variant payload coherently
using the existing public filament_options_with_variant registry. Native
export omits old set names with no live option; those remain absent, following
PrintConfig.cpp:9873-9877/9894-9897. Singleton payload duplication follows
ConfigOptionVector::get_at semantics. Non-variant palette/type stays logical2.
Source values and geometry are otherwise fixed; paired inputs differ only
in self-index. No arranging or context replacement hides a divergence.

| Case | Explicit IDs | Variant source slots | Active first Native temperature |
|---|---|---|---|
| F12 | [1,2] | Standard,Standard | M104 S211 |
| F21 | [2,1] | Same two-slot payload | M104 S233 |
| F1122 | [1,1,2,2] | Standard,High Flow,Standard,High Flow | M104 S211 |
| F2211 | [2,2,1,1] | Same four-slot payload | M104 S233 |

All first active commands are line831 in the two modes' RED captures;
subsequent actual M104/M109 commands consume both211 and233. Unused High
Flow payload temperatures213/235 never appear in the active temperature
sequence. G4_ACTIVE plus real wall/sparse XY extrusion proves both logical
filaments are used. The Native source self-index header retains each request.
Paired ZIP member/settings audits prove only self-index changed; all model,
placement and other settings bytes remain exact. These observations supplement
the complete ordered-byte comparator; they never filter its input.

## Separate commits and expectation update

RED `ebcbe76ba`: Native captures all four cases before public comparison.
Both modes accept4/4 with active slot consumption; validated API and CLI
both match Native's complete output and line counts. Only the new key-local
expectation fails: PASS expected, INCOMPLETE actual (one None baseline).
Both nextest legs exit100. Root `/tmp/worker6-self-index-red.YA3Y4c`,
`{official,seeded}/{log,exit}` and red-audit.json retain source/request,
input/Native/API/CLI SHA values and all acceptance/consumption proofs.
Actual Native executable sampling:11 PID/exe/mask entries, CPU11 only.

GREEN `b6d958a41`:41 added harness lines integrate the four-case finite
plan and executor into domains/sweep. The original entire RED test body
and input helper remain exact; production code unchanged.

Only after both Native proof legs ended, commit `270975c75` changes
numeric_metadata_tests.rs, and only that file:

- Before: assert !identity.provably_legal and identity.omission.is_some().
- After: assert identity.provably_legal and omission.is_none(), with all four
  explicit vectors and the bounded H2D slot-domain metadata locked positively.

This is an authorized expectation update, not deletion of coverage.
All other assertions before/after that self-index block are exact text.
REVIEW must inspect the standalone diff at
`/tmp/worker6-self-index-negative-expectation-approved.diff` (or
`git show 270975c75 -- .../numeric_metadata_tests.rs`). This update was not
mixed into GREEN, and no earlier negative assertion was changed before proof.
assertion-and-expectation-audit.json binds the exact RED body/helper and
unaffected assertions to the Native proof root.

## Gates on 270975c75

Each mode runs all option_coverage tests, including non-Native inventory/
classification tests; explicit OPTIONS=1 / OPTION_KEYS=filament_self_index
executes the real four-case matrix, not a visibly skipped sweep.
CPU11 outertaskset, jobs1/threads2; official then seeded, independent fresh
cache/artifact/target roots. Only compiled dependencies were copied; CLI/test
fingerprints were rebuilt in each new target path.

| Gate | Result |
|---|---|
| official complete group | 224/224 PASS, exit0,773.032s |
| seeded complete group | 224/224 PASS, exit0,1059.208s |
| each key-local matrix | 1/1 domain PASS,4 generated/4 strict compared/0 rejected |
| independent reread |16 Native/API/CLI captures whole-byte/line-count equal, active consumer correct; validated raw unchanged from RED |
| other domain plans | all651 keys retained; other650 exact versus reviewed master |
| common input Ares raw | official376/376 and seeded384/384 unchanged, zero unmatched/changed |
| actual Native executable affinity |1327 PID/exe/mask entries, CPU11 only, zero violations |
| strict Clippy / fmt / wasm32 |0 /0 /0 |

Directed evidence root `/tmp/worker6-self-index-green.dywJTA` contains
{official,seeded}/{log,exit}, artifacts/option-coverage-summary.md,
capture-reread.json, *-plan-invariance.json, *-common-raw-audit.json,
assertion-and-expectation-audit.json and native-affinity.json. Baseline for
plan/raw equality is reviewed master `/tmp/worker6-master-green.CmYW1O`.
Standard-env strict workspace/all-targets/all-features -D warnings, fmt and
core/vgcode/wasm wasm32 checks: `/tmp/worker6-self-index-engineering.QwGEUC`.

Only complete2/4 source slots with both active nozzles Standard are closed.
Invalid/short/empty IDs, missing variants, generated-index fallback, active
High Flow nozzle, exotic strides and arbitrary vectors defer. No new guards,
ignore, suppression, comparator change or Native output file is committed.
Files<=400 LOC. Final conclusion commit is docs only; tested code stays
270975. CI workspace/full are pending coordinator allocation; finite results
are not merge authorization. Master remains independently reviewable at72f51.
