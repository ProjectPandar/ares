# Adaptive PA bridge option: bounded coverage classification (APA-G1)

Status: **SPEC ONLY — awaiting coordinator approval before RED/harness GREEN**.
Analysis main `af7398b78168ebe497f1b38302de2da4878cad40`.
Owner: worker1; only key `adaptive_pressure_advance_bridges`.
This is a source-cited test-harness continuation of APA-R1 and G3, not a new
Ares pipeline or authorization for a production repair.

## Upstream boundary and current gap

OrcaSlicer v2.4.2 pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`;
paths below relative to `OrcaSlicer/src/libslic3r/`.

| Source owner | Contract being exercised |
| --- | --- |
| `PrintConfig.cpp:2401–2408`, `PrintConfig.hpp:1316` | `coFloats`, default `[0]`, explicit max 2; tooltip names zero as disable |
| `Config.hpp:2475–2479`, `libslic3r.h:52` | Absent explicit minimum inherits `-FLT_MAX`; effective bridge predicate uses EPSILON=1e-4, not >=0 |
| `Config.hpp:846–865,910–912` | Floating vector export uses default stream precision; close requests may collapse during export |
| `GCode.cpp:6720–6745,6816–6841::_extrude` | Non-calibration + PA + adaptive gates create ordered PA_CHANGE records with actual flow, tool and BR flag |
| `GCode/AdaptivePAProcessor.cpp:24–42` | Model map created only when both PA toggles enabled |
| `AdaptivePAProcessor.cpp:214–269::process_layer` | Missing/invalid model uses static PA; initialized model predicts, then overrides only for BR!=0 and bridge value>EPSILON |
| `AdaptivePAProcessor.cpp:270–295` | Debug override comment predicate is separate from valid-model prediction; writer update additionally depends on change/epsilon |
| `AdaptivePAInterpolator.cpp:17–75,83–118` | Model initialization/sample grouping and prediction are existing consumers, not changed here |

`2026-10-09-option-full-current-audit.md` reports this key INCOMPLETE from
unproven authored context boundaries, not a production DIVERGENT group.
`option_coverage/domains/probes.rs` currently injects only `.5` and `2`;
`domains.rs` treats authored numeric probes as incomplete. The generic sweep
uses cube10 and does not establish a reachable adaptive bridge dependency.
Merely changing `provably_legal` would be false closure.

Already integrated `adaptive_pressure_advance/bridge_domain.rs` supplies eight
runtime contexts with applied-value checks, real Native Bridge/BR1/BR0 activity
and whole ordered comparison. APA-R1 proved these contexts, explicitly NOT the
global coFloats schema. This slice moves that bounded admission contract into
the Option harness; it does not reinterpret the historical full matrix as a
fresh run or replace the established runtime witness with frozen outputs.

## Exact included finite contract

Single logical filament, ordinary non-calibration Ender-3/classic context;
existing `tests/parity/overhang.stl`, no prime tower, scarf None, adaptive-overhang
0, comments1, real bridge and nonbridge features. Reuse the applied settings
and exact valid nonconstant model/static PA0.035 from integrated bridge_domain.
Verify every key-local dependency after export, including both PA toggles,
model, comments, prime-tower/scarf/overhang settings; selected bridge request
replaces only its own dependency. Retain the raw input and full G-code.

Canonical selected-key Option sweep (no option-value filter):

| Request | Meaning |
| --- | --- |
| 0 | Disable override sentinel, NOT raw schema minimum |
| 0.0001 | Exact EPSILON, override predicate false |
| 0.000101 | Materializable above EPSILON, override predicate true |
| 0.06 | Existing active interior witness |
| 0.5 | Preserve the old authored interior request (`probe-min` is historical label, not a minimum claim) |
| 2 | Preserve the old inclusive explicit max request |

The declared classification PASS is **these six finite requests in this named
single-slot context**, not all reals in [0,2], all admitted negative values or
an exhaustive floating-point/vector domain. Preserve both old requests and
labels; replace no successful probe with a easier value. A partial value-filtered
run must still clear legality and cannot become PASS.

Keep the integrated eight contexts intact (five bridge values 0/EPS/above/.06/2
plus three `.06` dependency controls: PA off, adaptive off, invalid model).
Dedicated Option E2E(s) execute equivalent dependency controls beside the
canonical sweep, without claiming three contexts are three new option values.
They establish that positive bridge PA does not bypass the parent gates/model.
Invalid model may still print bridge override debug text while emitting static
PA0.035; do not assert the debug comment absent. Source command rounding/update
suppression means M900 presence alone is not threshold proof. Require actual
Native tags/comments, bridge/nonbridge reachability and complete byte comparison.

Previous `0.0001000001`→`0.0001` collapse is a retained admission observation;
keep the already approved representable probe. No new collapse control, export
rounding workaround, epsilon adjustment or requested/effective relaxation.

## Rust destination / minimal shared wiring

New logic lives in a key-local file under
`crates/ares-cli/tests/orca_parity/option_coverage/`, planned name
`adaptive_pa_bridges.rs`; split context/E2E siblings if needed to keep <=400 LOC.
Responsibilities: finite authored requests, explicit context/deferred metadata,
activation, dependency verification, Native reachability and dedicated tests.
Reuse OrcaRunner export/slice/cache, application checks, full-output comparator,
shared native_environment and existing summary machinery. No generic context
framework, production API, Native patch/build or new oracle cache.

`domains.rs`: module declaration + narrowly keyed delegation for this option
before the old authored-numeric rule. No generic completeness rule change.
Remove only this key's now-owned entry from `domains/probes.rs` when the new
module owns it; preserve all other entries.
Minimal call sites as necessary in `activation.rs`, `sweep.rs` and summary/plan
metadata output: activate this key's dependencies, select overhang geometry,
verify them and retain the bounded contract beside the row. Key-local behavior
remains in the new file, not expanded match logic in shared files. Geometry
selection must not reclassify this as a support-family option.

Keep OptionOutcome accounting and comparator semantics unchanged. If proving
Native activation requires a comparison hook, add only a key-specific call via
the new module; do not introduce a new scanner/export pipeline. A failed
reachability assertion is missing evidence, never PASS or an Ares RED.
Coordinate shared file wiring with worker2/3/6; inspect latest `domains.rs` and
related call sites after rebase, use minimal disjoint edits. No edits to their
G2/G3/G4 option logic.

## TDD stages (approval before tests)

1. Commit this source-cited spec alone and request review.
2. **Independent Native-backed RED:** add dedicated runtime E2Es first. Under
   current incomplete classification, export/slice the finite requests and
   controls, verify applied dependencies and Native BR1/BR0 gates, retain full
   PASS comparisons, then assert the bounded Option plan/result is complete.
   It must fail on the current INCOMPLETE classification. Existing `.5/2`
   generic plan can remain incomplete until GREEN; RED records which new
   dependency-backed requests were exercised separately.
3. If full bytes fail before the metadata assertion, preserve first difference
   and STOP for source-owner admission. Do not manufacture metadata RED around
   a divergent stream, adjust geometry/values or fix production in this slice.
4. **Harness GREEN only:** implement the key-local finite plan/context and
   bounded-positive classification, wire the canonical selected-key sweep into
   the same reachable context. Retain all successful/rejection assertions and
   original 8-context tests. Metadata expectation updates are limited to this
   key's explicit scoped-positive assertion; add a nonvacuous unproven/partial
   countercontrol that still yields INCOMPLETE.
5. Prove the other inventory plans unchanged (keys, cases, legal flags,
   omissions, context metadata). One key's bounded classification can change;
   no global PASS claim or reduction of the unfiltered full matrix's residuals.

No unit tests after implementation. Any needed isolated metadata assertions
are authored in the RED commit with explicit failure inventory: inert cube,
missing enabler/model, collapsed export, threshold off-by-one, invalid-model
comment/static confusion, wrong slot, omitted dependency proof, partial-sweep
false PASS and missing Native falsely passing.

## Execution and exit evidence

Native directed runs: start only when load1<60, nextest **2 threads**; one heavy
stage at a time. Full runs: start when load1<30; retain the measured load/admission
receipt. CPU pinning and inherited producer identities remain unchanged. Builds
and tests freeze the source tree; caches/artifacts/logs outside Git. Optional
missing Native visibly SKIPPED, required missing Native FAIL; configured invalid
path/nonzero/timeout/missing output FAIL, never fallback or skip.

- Fresh required seeded and official directed cold/hot tests, all finite cases
  and dependency controls; no active-case skips/timeouts.
- Selected-key unfiltered Option sweep: all six authored requests accounted
  for, full comparisons PASS, scoped row PASS; no excused/rejected successes.
- Default full workspace via nextest; strict workspace/all-target/all-feature
  Clippy, fmt and WASM. Source-only harness slice does not authorize core changes.
- Fresh seeded/official base/candidate v2 full controls with independent targets
  and different binary SHAs before launch; compare by test name. Seeded candidate
  all PASS; official candidate FAIL subset baseline FAIL. Standard v2 Native
  nonexecution/seeded-only guards remain visibly recorded separately from
  Nextest skip counts. List all raw Ares changes and their source/input authority.
- Exact-hash review before authorized integration. Test/docs-only main carry
  permitted only with owner ruling; production main changes require fresh gates.

## Deferred / no false closure

Inherited -FLT_MAX endpoint, negative-disable equivalence, below/above raw schema
rejection and extreme materialization, malformed/nonfinite values, arbitrary
floating-point export precision, multiple logical slots/short-vector fallback,
SEMM/tool ordering, calibration mode, lookahead/reset, sloped mixed roles,
overhang ramp interaction, printer hardware limits and other PA firmware
formats are deferred. No new negative/upper-rejection witness is silently claimed.
`bridge_flow`, `internal_bridge_flow`, `filament_diameter` and
`filament_flow_ratio` stay with their own incomplete/assigned source boundaries.
Type2 activation remains worker2's work. Existing Ares scaffold is only the
compatibility shell around the cited upstream options/consumer, never an
independent pipeline design.

## Admitted .06 Native nondeterminism (coordinator ruling)

First runtime attempt `0bf96342-efc0-4adf-a0cf-46e8d2e2b67d`:
8/9 contextual whole comparisons PASS; interior .06 differs at byte20964,
line662 (internal-solid-infill travel): Native X113.388/Y113.135 versus
Ares X113.244/Y113.297. Initial Native raw SHA
`cc5769d8778be94ac91e89b293abccc731dc90185cf621cf4e5cc7baa8aee79f`.
Classification assertion was not reached; this is not the metadata RED.

Authorized read-only fixed-input investigation kept input raw SHA
`622a00603557b44d5aec41c49fdeb093aaf01d338d5a1ee73b5267a0541eeb99`
and every member byte fixed. Three cold Native processes, CPU39 as the original
receipt, all match two raw-stable public Ares outputs under the existing whole
stream generated-by-line identity policy. Three cold runs alone have one
variant; including the initial independently captured Native yields two.
Ares raw SHA both runs and the original failed case:
`ed03e3a1d25f4a6a35da152aa1918e0cf8d21713745de91aa698ab98d16d43dd`.
Original R1 .06 on af739's immutable binary also matches; input members exact.
Evidence: `${TMP_ROOT}/ares-apa-g1-bridges/nd-diagnosis/{fixed-input-audit.json,original-and-main-control.json}`;
first failure retained under `${TMP_ROOT}/ares-apa-g1-bridges/red-official`.

Coordinator explicitly ruled Native ND and authorized ONLY this .06 context to
use the existing `compare_exported` bounded Native retry path. Keep all observed
variants and the first failure; no new tolerance, filter, comparator, run-budget
change, chosen frozen authority or production fix. Every resulting PASS still
requires a complete ordered comparison against a genuine Native output. Re-run
RED to establish that only the incomplete-classification assertion fails before
harness GREEN. Other contexts retain their original strict path.

## Native-backed classification RED and harness GREEN checkpoint

Accepted admission run `fb395a3f-47e4-4b7f-bf19-579cfdcbfdff`,
`${TMP_ROOT}/ares-apa-g1-bridges/red-classification-official`: all nine full comparisons
and application/BR1/BR0 checks PASS, then only the classification assertion
fails (INCOMPLETE versus PASS). Independent RED commit `70145c3f4`; the authorized
pure-test main advance to `1b1c38eb9` was carried git-only to `c59d8132b` without
claiming new runtime evidence.

Harness implementation is key-local: six cases preserve old probe-min .5 and
probe-max 2.0 labels, fixed dependency activation, applied-value receipts before
Native reachability and full comparison, one bounded metadata row. Cold/hot
seeded and official directed runs each PASS both the nine-context classification
test and selected-key six-case unfiltered-value sweep. Evidence directories:
`${TMP_ROOT}/ares-apa-g1-bridges/{green-first-official,green-hot-official,green-cold-seeded,green-hot-seeded}`.
Inventory audit `other-plan-inventory-audit.json`: 650 rows in each tree,
only adaptive_pressure_advance_bridges differs; all other 649 rows exact.
Targeted strict Clippy exit0. These checkpoints do not substitute for the final
workspace/full controls and exact-hash combined review; no production PA,
geometry, writer, processor or comparator change.

Approval requested: this bounded test-harness contract and Native-backed
classification RED, followed by harness-only GREEN. No tests/implementation
start before approval.
