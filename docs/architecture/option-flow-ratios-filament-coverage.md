# Bounded flow-ratio and filament-diameter option coverage

All four bounded contracts are verified in separate RED→GREEN slices.
Latest full observation
is [the current option audit](2026-10-09-option-full-current-audit.md): these
four domains are INCOMPLETE. This proposal does not claim a fresh 650-domain
execution. G2 and APA bridge options are separate slices.

## Upstream boundary

OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/`:

| Option or consumer | Source | Contract |
| --- | --- | --- |
| `bridge_flow` | `PrintConfig.cpp:1327-1337`, `PrintConfig.hpp:1084` | coFloat, schema 0..2, default 1; loaded validation rejects nonpositive values at cpp:10427-10429 |
| `internal_bridge_flow` | `PrintConfig.cpp:1354-1364`, `PrintConfig.hpp:1086` | coFloat, schema 0..2, default 1; loaded validation rejects nonpositive values at cpp:10432-10434 |
| `filament_flow_ratio` | `PrintConfig.cpp:2315-2325`, `PrintConfig.hpp:1309` | Nullable coFloats, schema 0..2, default [1]; loaded validation rejects nonpositive entries at cpp:10450-10454 |
| `filament_diameter` | `PrintConfig.cpp:2606-2611`, `PrintConfig.hpp:1325` | coFloats, schema minimum 0, default [1.75], no explicit maximum; loaded validation rejects entries below 1 at cpp:10354-10358 |
| Bridge geometry | `LayerRegion.cpp:31-58::bridging_flow`, `Flow.hpp:104-106`, `Flow.cpp:166-206,219-231` | Thick bridge diameter scales by sqrt(bridge_flow); thin bridge changes cross section while keeping spacing. Circular versus rounded-rectangle cross sections determine mm3/mm |
| Perimeter consumer | `LayerRegion.cpp:133-140`, `PerimeterGenerator.cpp:1166,2107` | Both classic and Arachne receive overhang flow built by bridging_flow and use its mm3/mm |
| Bridge fill consumer | `Fill/Fill.cpp:904-910,954-958,1342-1345` | Distinguishes external/internal bridge extrusion roles, selects thick branch per surface type, and applies internal bridge density |
| Effective extrusion | `GCode.cpp:6467-6478,6510-6513::_extrude` | Geometric volume times print and filament ratios; internal bridge additionally multiplies internal_bridge_flow; conversion to filament length cancels the duplicate cached filament ratio |
| Filament conversion | `Extruder.cpp:16-18,148-165`, `Extruder.hpp:61` | Cache ratio / cross section, get_at logical filament lookup, area = diameter * diameter * 0.25 * PI |

`internal_bridge_flow` is not substituted for `bridge_flow` in
LayerRegion::bridging_flow: the latter constructs bridge geometry, while the
former is a role-specific GCode multiplier. Tests must isolate these consumers
rather than assume that both options have identical geometric effects.
Schema endpoints alone do not establish physical feasibility. Diameter's
inherited storage maximum is not an authored physical endpoint.

## Rust destination and scope

Only `crates/ares-cli/tests/orca_parity/option_coverage/`: `domains.rs`,
`domains/probes.rs`, key-local activation and sweep application checks, and
dedicated flow E2Es/context modules, split below 400 LOC. Reuse VendorProfiles,
runtime OrcaRunner export/slice, application verification, external cache and
the existing complete ordered-byte generator-only comparison. The harness is
an admission adapter around the cited upstream boundary.

Existing core counterparts remain unchanged:
`project_slice/perimeters/flow/fill.rs::resolve_fill_bridge_flow` and
`perimeters/flow.rs` thick bridge resolution,
`project_slice/group_fills/params/projection.rs` role projection,
`extrusions/options/accessors.rs` filament-area and role multipliers,
`extrusions/options/hardware.rs`, and existing options validation. No production
API, cache recipe, comparison normalization, routing, or general context
framework is introduced. A real Ares/Native difference is retained and reported
before classification promotion or any production work.

## Included finite contracts

Single Ender-3 nozzle 0.4mm, one filament, ordinary/initial height 0.2mm,
relative E, no support/raft/prime tower, ordinary print flow ratio 1 and other
optional role ratios disabled. Verify all declared dependencies after Native
export and in Ares effective settings, including filament vector cardinality.
Keep the existing standard templates, M73 and footer comparison intact.

| Domain | Successful authored requests | Canonical activation |
| --- | --- | --- |
| bridge_flow | 0.5, 1, 1.5, 2 | Existing `tests/parity/overhang.stl`, external Bridge paths, internal_bridge_flow=1, filament ratio=1, diameter=1.75, thick_bridges=1 |
| internal_bridge_flow | 0.5, 1, 1.5, 2 | Existing cube10, rectilinear sparse infill 15%, three top/bottom shell layers, zero shell-thickness overrides, thick_internal_bridges=1, bridge_flow=1, filament ratio=1, diameter=1.75 |
| filament_flow_ratio | [0.5], [1], [1.5], [2] | Existing cube10 with ordinary perimeter/infill extrusion, diameter=[1.75], bridge ratios=1 |
| filament_diameter | [1], [1.75], [2.85], [3] | Same ordinary cube10 context, filament_flow_ratio=[1], bridge ratios=1 |

Native must actually emit positive XY extrusion in the relevant role for every
successful request. Require nonempty external Bridge paths for bridge_flow,
Internal Bridge paths for internal_bridge_flow, and ordinary extrusion for the
two filament keys. Compare role extrusion observations across requests to
prove the option affects consumption, in addition to verifying export values
and complete bytes. Do not replace whole output equality with that observation
or assume identical path coordinates when flow changes geometry.

Add dedicated thin-bridge controls for the same four requests per bridge key,
changing only its corresponding thick flag to 0 and verifying the flag and
actual role. Canonical sweeps keep the declared thick context. If a fixture
fails to activate a role, report the missing witness; do not count an unchanged
CONFIG_BLOCK/footer as proof or silently reduce the request set.

Runtime rejection witnesses: zero for each ratio and diameter 0.99999. Record
actual Native stage, status and diagnostic separately; pair the same options
with existing public Ares validation/validated slicing. Diameter 1 is the
inclusive loaded-validation boundary and must actually be admitted at runtime.
Unexpected rejection, rewriting, nullable behavior or parity mismatch blocks
the affected sub-slice and is reported. No claim about a smallest positive
ratio or diameter upper endpoint follows from these finite requests.

## RED, GREEN and expectation updates

Two vertical sub-slices: bridge ratios, then filament ratio/diameter. Each has
a separately committed runtime Native-backed RED before harness GREEN. RED
executes all requested successful full-byte comparisons and activation checks,
then fails the existing INCOMPLETE classification expectation. A byte failure
comes first and remains a real RED reported explicitly.

GREEN promotes only the proven keys and records the bounded context/deferred
scope in summary metadata. Selected-key sweeps must execute every authored
request without value filters. Bridge sweep selects the declared input rather
than using an unactivated cube for external bridge flow. Source wire shapes
remain scalar for process ratios and one-element vectors for filament keys.

Preserve existing metadata assertions. Add positive bounded expectations for
the four proven keys; leave all unrelated key locks unchanged. Existing
`upstream_rejection_does_not_mask_compared_case_parity` and
`rejected_unexecuted_and_baseline_only_domains_are_incomplete` currently borrow
bridge_flow's unproven plan. Retain their rejection/FAIL and partial-comparison
semantics using an explicit unproven copy where needed, so promotion cannot
make these negative controls vacuous. Report exact before/after assertions and
the 650-plan diff; only the current sub-slice's keys may change.

## Acceptance and deferred behavior

Required seeded/official cold and hot directed tests, rejection witnesses and
unfiltered selected sweeps; fresh base/candidate seeded and official controls
with common Ares raw hashes and all status changes listed. Default environment
workspace, strict clippy, fmt and wasm check. Native dependencies use shared
native_environment: optional visible SKIPPED, required FAIL. No Native output is
committed. No Native rebuilds.

Deferred: nullable/inherited filament entries, multiple filaments/nozzles,
SEMM/tool ordering, object/region/range overrides, pellet coefficient conversion,
support flows, arbitrary bridge geometries and densities, all combinations of
print/role ratios, volumetric E modes and extreme positive numeric feasibility.
APA and `adaptive_pressure_advance_bridges` are explicitly excluded. A bounded
PASS closes only the declared finite contract; it does not certify every
schema value or constitute a fresh full-option audit.

## Bridge-ratio conclusion

RED `3ab5db697` executes all 16 thick/thin successful requests against runtime
Native complete output, witnesses positive extrusion in the intended role,
then fails the four INCOMPLETE classification assertions. Its fifth test
passes the two zero-value rejection witnesses: Native Export returns -18
(Unix status 238), matching both public Ares validation and validated slicing.
Earlier test-observer failures are not claimed as RED evidence. GREEN `b8da6c998` promotes only these two bounded plans;
no production code, Native artifacts, or comparison normalization changes.

Metadata expectations add `provably_legal=true`, `omission=None`, and exact
requests `[0.5, 1, 1.5, 2]` for `bridge_flow` and `internal_bridge_flow`.
Existing partial/rejection negative controls and all prior expectation loops
remain unchanged. The 650-plan comparison changes exactly these two keys;
the other 648 plans are identical. This is inventory comparison, not a fresh
execution of all 650 domains.

Required seeded and official cold/hot directed runs each pass 9/9 tests:
24 complete streams per run, 96 total equal, with activation and rejection
witnesses. For each producer, cold has 48 cache misses and two within-run hits;
hot has 50 hits. Fresh full controls on baseline `016dfc86f` produce:

| Control | Base | Candidate | Common Ares raw SHA sets |
| --- | --- | --- | --- |
| seeded | 871/872 PASS | 877/877 PASS | 541 keys, unchanged |
| official | 871/872 PASS | 877/877 PASS | 530 keys, unchanged |

In both controls the only existing-test change is
`option_coverage::sweep::option_coverage` FAIL→PASS.
The five added tests PASS: external bridge thick, external bridge thin,
internal bridge thick, internal bridge thin, and zero bridge-ratio rejection.
There are no candidate-only failures, common Native generator-normalized SHA
changes, or common manifest-status changes. Official base/candidate used fresh
separate caches. Default-environment workspace passes 8248/8248 (five existing skipped);
strict workspace clippy, fmt, and core/wasm wasm32 checks pass.

Filament ratio and diameter remain
unpromoted at the bridge slice's boundary; their subsequent evidence follows.

## Filament-ratio and diameter conclusion

RED `030d375ce` runs all eight successful requests against runtime Native,
including diameter 1mm, and witnesses positive ordinary outer-wall extrusion
that changes across requests. Every complete output matches before the two
INCOMPLETE classification assertions fail. The third test passes: Native
rejects ratio `[0]` and diameter `[0.99999]` at Export with -18 (Unix 238);
diagnostics match both public Ares validation and validated slicing. No
request is rejected unexpectedly or rewritten to a different numeric value.
The RED uses a fresh cache; no timeout occurs.

GREEN `073a12d43` changes only these two harness plans and their sweep hook.
Their metadata expectations add `provably_legal=true`, `omission=None`, and
exact one-element vectors: ratio `[0.5]`, `[1]`, `[1.5]`, `[2]`; diameter
`[1]`, `[1.75]`, `[2.85]`, `[3]`. All prior expectation loops and negative
controls remain unchanged. The 650-plan comparison changes exactly these two
keys, leaving the other 648 identical, including the already completed bridge
plans. This does not claim a fresh full-option execution.

Required seeded/official cold and hot directed runs each pass 7/7 tests,
with 16 complete comparisons per round (64 total equal). Each producer's cold
cache has 32 misses and two within-run hits; hot has 34 hits. Fresh full
controls compare baseline `1151db4bc` with GREEN:

| Control | Base | Candidate | Common Ares raw SHA sets |
| --- | --- | --- | --- |
| seeded | 881/882 PASS | 885/885 PASS | 568 keys, unchanged |
| official | 880/882 PASS | 885/885 PASS | 557 keys, unchanged |

Both controls change sweep FAIL→PASS and add three PASS tests: filament ratio
bounded output, filament diameter bounded output, and invalid filament
ratio/diameter rejection. Official additionally changes
`runner::support_annotations::overlapping_support_blockers_preserve_complete_output`
FAIL→PASS. Its identical input bytes and producer/request components yield
identical complete Ares raw output; only Native selects a different internal
solid-infill entry point at line 508. See the Native-only observation in
[the existing nondeterminism record](2026-10-08-nearest-seam-gaussian-validation.md).
Strict candidate-failure-subset checks pass unchanged in both controls.
There are no candidate-only failures or Ares byte differences. Seeded has no
Native content changes; official has this one Native-only change.

Full pairs use separate fresh caches. Default-environment workspace passes
8256/8256 (five existing skipped); strict workspace clippy, fmt, and core/wasm
wasm32 checks pass. Production code, cache recipes, input fixtures and
comparison normalization are unchanged; no Native output is committed.

Completion closes
only the finite single-filament contracts above; all listed adjacent behavior
remains deferred.
