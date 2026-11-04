# G2: bounded extruder clearance and nozzle-height coverage

Harness-only slice for
`extruder_clearance_height_to_lid`, `extruder_clearance_height_to_rod`,
`extruder_clearance_radius`, and `nozzle_height`.
Follow the G3 workflow, not its nozzle/width physical limits. This
spec introduces no production change.

## Source-owned boundary

Pinned OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

| Owner | Source | Included contract |
| --- | --- | --- |
| Rod clearance | `PrintConfig.cpp:2215-2222` | `coFloat`, mm, schema minimum 0, default 40 |
| Lid clearance | `PrintConfig.cpp:2225-2232` | `coFloat`, mm, schema minimum 0, default 120 |
| Clearance radius | `PrintConfig.cpp:2234-2240` | `coFloat`, mm, schema minimum 0, default 40 |
| Nozzle-tip height | `PrintConfig.cpp:2242-2248` | `coFloat`, mm, schema minimum 0, default 2.5 |
| Loaded validation | `PrintConfig.cpp:10436-10447::validate` | All four must be strictly positive; zero is not an accepted schema endpoint |
| CLI loaded-validation phase | `OrcaSlicer.cpp:3647-3653`, `Utils.hpp:43` | Nonempty validation map exits `CLI_INVALID_VALUES_IN_3MF=-18`; retain actual stage/channels/result |
| Applied print dispatch | `Print.cpp:1284-1306::Print::validate` | Sequential clearance runs for by-object printing with multiple objects/instances; smooth timelapse and wrapping detection have earlier guards |
| Horizontal clearance | `Print.cpp:640-722::sequential_print_clearance_valid` | Offset convex hulls, compare actual intersection; ordinary offset is `0.5*radius + skirt_offset - 0.1` |
| Short-object predicate | `Print.hpp:42,1120-1122` | Every object height is strictly below scaled `nozzle_height`; short offset uses half `MAX_OUTER_NOZZLE_DIAMETER=4`, not configured clearance radius |
| Vertical clearance | `Print.cpp:734-735,833-930` | Active ordering is model object index; future Y overlap selects rod, otherwise lid; last instance starts with printable height; reject when selected limit is strictly below object `max_z` |
| Skirt dependency | `Print.cpp:3777-3797::object_skirt_offset` | Per-object skirt/draft-shield can alter offsets and depend on nozzle height; zero skirts isolate this slice |
| CLI applied rejection | `OrcaSlicer.cpp:6061-6094`, `Utils.hpp:65` | Sequential collision is `CLI_OBJECT_COLLISION_IN_SEQ_PRINT=-63`, not a generic invalid-number rejection |

All four inherit the schema ceiling from `Config.hpp:2475-2479`; there is no
explicit physical maximum. `FLT_MAX`, historical probes 250/400, and a selected
bed's dimensions must not be presented as universal legal upper endpoints.
The upstream option named radius is consumed with the factor **0.5** above;
do not substitute an independently designed geometric interpretation.

The old arrangement-score sorter inside `Print.cpp` is under `#if 0`.
Do not claim Native rearranges tall objects to rescue a rejection. Rod overlap
is checked against **later** instances, after shrinking the current inflated
Y bounds. A single-object by-layer export alone cannot prove these consumers.
Object height (short predicate) and `max_z` (vertical rejection) are distinct;
the contexts below deliberately use zero Z offset so both are 10mm.

## Rust destination and current gap

Production owners already exist:

- `ares-core/src/options/validation/dimensions.rs::SliceOptions::validate_extruder_clearance_options`;
- `ares-core/src/print/validate/sequential_clearance.rs::validate`, with its
  `skirt.rs` helper and existing project-slice validation entry.

These remain untouched. This is a test-harness admission slice around upstream
`PrintConfig::validate` and `Print` consumers, not an Ares-owned pipeline or a
new validator, registry, cache, geometry engine or public API.

The current full audit is actual `acf100550`, not current-head execution:
`2026-10-09-option-full-current-audit.md` lists all four as INCOMPLETE.
`option_coverage/domains/probes.rs:145-148` authors 0.5/250 for the three
clearances and 1/400 for nozzle height; these are samples, not proven bounds.
`domains.rs::plan` retains the authored-numeric omission and false
`provably_legal`. Existing matching sample bytes do not justify changing it.

New logic belongs in **`crates/ares-cli/tests/orca_parity/option_coverage/clearance.rs`**
and its `clearance/geometry.rs` child (the combined module exceeds 400 LOC):
key-local plan authoring, explicit context/deferred descriptions, runtime
boundary controls and classification assertions. `domains.rs` and `tests.rs`
receive only minimal wiring and the four keys' retained expectation updates.
Reuse existing export/application verification, primitive cube input, project
transform helpers, native_environment and strict full-stream comparison.
Do not grow a general context framework or alter shared normalization/activation.

G3 height work in this directory is a separate slice; resolve only the
G2 wiring and never change G3 requests, metadata, assertions or results.
No plate-origin/cache changes belong to this slice.

Application-check correction: upstream `ConfigOptionFloat::serialize`
(`Config.hpp:777-781`) uses stream number formatting, so the requested decimal
`"0.00001"` exports as `"1e-05"`. These are the same finite f64 value. The first
GREEN four-key sweep failed string application checks before comparing these
four streams; retain that attempt. `clearance.rs` now verifies exact finite f64
application for its four coFloat keys, with no epsilon; `sweep.rs` has only a
key-local dispatch hook. The other 646 plans retain the shared verifier.
Requested wires and strict whole-G-code comparison are unchanged.

## Declared finite coverage, not a global numeric-domain claim

### A. Canonical selected-key sweep

Retain the Ender-3 0.4mm smoke fixture's existing one-object/by-layer context
and existing value labels. Preserve all eight old requests, adding one explicit
positive near-zero request `0.00001` per key (not a smallest-positive claim):

- Rod, lid, radius: `0.00001`, `0.5`, `250` mm each.
- Nozzle height: `0.00001`, `1`, `400` mm.

There are **12 authored successful requests**, conditional on actual Native
acceptance and complete byte equality. Verify the option, resolved print
sequence, nozzle diameter and model geometry in the exported/effective input;
record the actual baseline dependencies. Do not silently edit generic smoke
defaults to make a request pass. An unexpected Native rewrite/rejection or
Ares mismatch stops the slice and is reported.

The by-layer context establishes loaded acceptance, option application and
complete serialization/emission parity. It does **not** activate sequential
collision semantics. The consumer witnesses in B are mandatory before bounded
classification can become PASS. Metadata must distinguish these scopes, the
open lower bound, and the unproved global upper bound.

Add eight independent loaded-rejection witnesses: `0` and `-0.00001` for each
key, with the other three kept positive. Preserve the Native loaded validation
map, actual export/slice phase, exit/status, stdout/stderr and result record.
Compare the same option values at the existing public Rust loaded-validation
boundary. If Native export rejects before any 3MF exists, state that interface
boundary explicitly: do not fabricate a Native-generated 3MF or call the Rust
options API an identical Native argv interface.

### B. Explicit sequential consumer contexts

Use the existing cube10 input twice, two separate model objects, one 0.4mm
nozzle, no raft/support, no per-object skirts (`skirt_loops=0`), no draft shield,
no prime tower, traditional timelapse, wrapping disabled, and by-object sequence.
Use the existing by-object fixture's M73 setting unchanged and compare the
**entire** output, not only extrusion or object-transition snippets.

Apply fixed transforms through existing project helpers after unchanged Native
export; both consumers receive the same resulting project bytes. Verify
object count/order/names, transforms, 10mm heights and `max_z`, actual bounds,
zero Z offset and effective dependencies. The ordinary bed comfortably contains
the stated layouts. No automatic re-arrangement, scene repair or fallback may
replace a failed application check.

| Target | Fixed context | Source-derived accept candidates | Rejection candidates |
| --- | --- | --- | --- |
| Lid | Cube centers `(60,60)`, `(60,90)`; radius 10, rod 40, nozzle height 2.5; future Y bounds disjoint | lid 10, 10.00001 | lid 9.99999 |
| Rod | Centers `(60,60)`, `(90,60)`; radius 10, lid 120, nozzle height 2.5; future Y bounds overlap but hulls do not intersect | rod 10, 10.00001 | rod 9.99999 |
| Radius | Centers `(60,60)`, `(90,60)`; 20mm raw X edge gap; lid 120, rod 40, nozzle height 2.5; objects not short | radius 20.19999, 20.2 | radius 20.20001 |
| Nozzle height | Centers `(60,60)`, `(75,60)`; 5mm raw X edge gap; radius 20, lid 120, rod 40 | nozzle height 10.00001: all objects short, offset 1.9mm | nozzle height 9.99999 and 10: ordinary offset 9.9mm |

These are **Native witness candidates**, not already observed PASS/rejection
results. For radius, two offsets meet at the derived 20.2mm value; actual
scaled/float/Clipper touching behavior must be witnessed, not asserted from
algebra alone. Retain source numeric conversions and the 0.1mm shrink; no
comparator tolerance or substitute endpoint is permitted. If the touching
candidate behaves differently, report it before changing the spec or requests.

For vertical equality, the source compares `limit < max_z`; for shortness,
it compares `height < nozzle_height`. Therefore equality has opposite effects
in those two consumers. The close-layout nozzle-height controls must genuinely
switch horizontal acceptance, not merely change a config footer. Radius/lid/
rod controls must hit their named consumers, not an earlier unrelated guard.

Add a last-instance dependency control: first cube 10mm, last object 20mm
(using an existing Z-scale transform), radius 10, disjoint Y, lid 10, rod 40,
printable height above 20. It should accept with the tall object last; swapping
model order should reject the first 20mm object's lid limit. Verify actual
order and geometry; do not rely on the disabled source sorter.

Successful controls require complete Native/Ares byte equality. Collision
controls compare the same exported project at public `ares_core::slice_project_plate_validated`
and retain the existing typed `SequentialPrintCollision` diagnostic. The owning
entry is `ares_core::slice_project_plate_validated`, also used by the CLI;
`slice_project` explicitly disables print validation and is not a rejection oracle.
Capture
Native's -63 phase/status/text separately from loaded -18 rejection; no G-code
is expected from a pre-emission validation rejection. Rejections are not added
to the successful comparison denominator or relabeled as successful slicing.

Per-object skirt/draft-shield branches are **deferred**, not covered by setting
skirts to zero. Record this next to the context rather than claiming complete
coverage of every nozzle-height/radius consumer.

## RED then harness-only GREEN

Proceed one key at a time. Commit a Native-backed RED that:

1. Verifies canonical application, named acceptance/rejection boundaries and
   relevant geometric dependency controls at runtime.
2. Strictly compares all successful full streams, retaining raw channels,
   statuses, effective inputs and durable artifacts (not committed).
3. Only after those assertions succeed, asserts the declared bounded plan is
   classified PASS; current INCOMPLETE metadata must fail this final assertion.

A genuine Ares byte/validation difference is a production RED: preserve it and
report before touching any core owner. Do not change classification around it.
No invented RED, dropped request, assertion deletion, successful-prefix/footer
comparison, Native artifact fixture, tolerance or retry-to-PASS.

GREEN changes only four G2 plans' bounded classification/context metadata and
minimal wiring. `PASS` means the stated finite context and consumer witnesses,
not all real printer geometries or every positive IEEE float. Keep an explicit,
nonempty unproved-plan countercontrol: matching samples with unproved context
must still classify INCOMPLETE. Preserve unrelated negative metadata locks.
Inventory diff against the post-G3 baseline must prove the other **646** plans
unchanged, including their requests, dependencies, omissions and classifications.

## Exit criteria and exclusions

- Required seeded and official cold directed runs plus cache-hit controls;
  Native nonzero/signal fails closed, no crash retry/skip. Optional absence is
  a shared visible SKIP before artifact-root access; required absence fails.
- Unfiltered four-key sweeps account for every canonical authored request;
  rejection witnesses and sequential controls have separate counts.
- Preserve actual commits, executable/input SHA bindings and all failed attempts.
  Cache hits are reuse, not new Native stability observations.
- Fresh seeded/official BASE–CAND controls, isolated Cargo targets, raw Ares
  common-byte comparison and exact-name failure-subset reporting; baseline
  failures do not prevent candidate pairing. No generic Native ND exemption.
- Default workspace via nextest, strict workspace/all-targets/all-features
  clippy, rustfmt and WASM compilation.
  No production edit is made; report any genuine disparity.

Deferred: arbitrary meshes/bed exclusions, rotations and nonzero Z origins,
object height compensation, support/raft protrusions, variable layer schedules,
per-object skirts and draft shield, multiple instances/order permutations beyond
the named control, mixed nozzles/SEMM/toolhead offsets, printer-specific physical
safety maxima, malformed float strings/NaN/infinity, SLA and UI arrangement.
No global boundedness claim, self-designed slicing feature or platform-specific
core behavior is introduced. This spec is not a fresh 650-domain audit or proof
of all-printer/default safety. The observed finite witnesses below are narrower.

## Observed verification

No production file changed. GREEN changes only four-key admission,
its finite context, the key-local exact-f64 application proof and test wiring.
Near-zero requests remain `"0.00001"`; no G-code numeric/ordering/footer tolerance
was added.

Seeded and official cold and same-cache-hit directed runs each ran 8 tests, all
PASS. Per stage: 12 canonical full-G-code matches, eight loaded -18 map witnesses;
14 geometric controls comprise eight full-G-code matches and six validated API
plus CLI exit 193/-63, diagnostic, stdout/stderr/result and no-output matches.
The separate selected-key sweep generated/strictly compared all 12 requests,
four plans PASS, zero rejected. Matching samples with explicitly unproved context
still classify INCOMPLETE. Inventory contains 650 plans; the other 646 plans are
exactly unchanged.

Full isolated BASE–CAND controls: seeded 882 → 889 PASS; official BASE had one
FAIL
(`runner::compatible_regions::infill_first::modifier_infill_first_preserves_region_wall_phases`)
and CAND 889 PASS, so the candidate failure set is a baseline subset; no ND
exemption was needed. Seeded 705 and official 667 common full identity streams
are unchanged (only entire generated-by lines ignored); each has 20
candidate-only streams: 12 canonical plus eight geometric accepts.

An early geometric apparent discrepancy used unchecked `slice_project`; its
success did not test print validation. That claim was retracted, no production
RED was manufactured, and the corrected validated API/CLI run passed all 14
controls. No rejection is counted as successful slicing, and no stock-signal case
is readmitted by this slice.
