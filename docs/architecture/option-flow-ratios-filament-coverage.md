# Bounded flow-ratio and filament-diameter option coverage

Status: proposed worker6 harness slice, following G3. Latest full observation
is [the current option audit](2026-10-09-option-full-current-audit.md): these
four domains are INCOMPLETE. This proposal does not claim a fresh 650-domain
execution. G2 belongs to worker3; APA bridge options belong to worker1.

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
`project_slice/perimeters/flow.rs::resolve_fill_bridge_flow` and thick bridge
resolution, `project_slice/group_fills/projection.rs` role projection,
`extrusions/options/accessors.rs` filament-area and role multipliers,
`extrusions/options/hardware.rs`, and existing options validation. No production
API, cache recipe, comparison normalization, routing, or general context
framework is authorized. A real Ares/Native difference is retained and reported
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
comes first and remains a real RED requiring coordinator review.

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
native_environment: optional visible SKIPPED, required FAIL. All Native output
and receipts remain outside the repository. No Native rebuilds.

Deferred: nullable/inherited filament entries, multiple filaments/nozzles,
SEMM/tool ordering, object/region/range overrides, pellet coefficient conversion,
support flows, arbitrary bridge geometries and densities, all combinations of
print/role ratios, volumetric E modes and extreme positive numeric feasibility.
APA and `adaptive_pressure_advance_bridges` are explicitly excluded. A bounded
PASS closes only the declared finite contract; it does not certify every
schema value or constitute a fresh full-option audit.
