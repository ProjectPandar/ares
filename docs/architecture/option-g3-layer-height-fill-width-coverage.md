# G3: bounded layer-height and LockedZag width coverage

Four test-harness domains:
`layer_height`, `initial_layer_print_height`, `skin_infill_line_width`, and
`skeleton_infill_line_width`. This slice makes no
changes to production validation, slicing, Flow or fill generation.

## Upstream boundary

Pinned OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

| Boundary | Source | Contract |
| --- | --- | --- |
| Normal layer height | `PrintConfig.cpp:771-777`, `PrintConfig.hpp:936` | `coFloat`, schema minimum 0, no explicit maximum |
| First layer height | `PrintConfig.cpp:3384-3391`, `PrintConfig.hpp:1540` | `coFloat`, schema minimum 0, no explicit maximum |
| Skin/skeleton widths | `PrintConfig.cpp:4064-4083`, `PrintConfig.hpp:1135-1136` | `coFloatOrPercent`, minimum 0, ratio over nozzle diameter, default 100% |
| Loaded configuration | `PrintConfig.cpp:10337-10351,10488-10517::validate` | Reject nonpositive heights; normal height also has the scaling-remainder predicate; width maximum is five times maximum nozzle diameter |
| Applied print validation | `Print.cpp:1543-1556,1639-1672::validate` | Height may equal the relevant minimum used nozzle diameter; nonzero absolute width must exceed layer height and be at most five times maximum used nozzle diameter |
| Layer parameters | `Slicing.cpp:69-146::slicing_parameters`, `:809-868::generate_object_layers` | Store configured normal/first heights, construct object layers; profile limits also incorporate configured normal height |
| Actual LockedZag flow | `Fill/Fill.cpp:990-1000::group_fills`, `Flow.cpp:21-34,129-143,200-206` | Nonbridge LockedZag builds skin/skeleton flows from region width and actual surface/layer height; literal nonpercent zero selects auto width, while zero percent is not that auto branch |

The schema's inherited `FLT_MAX` ceiling (`Config.hpp:2475-2479`) is not a
legal physical endpoint. `MAX_LINE_WIDTH_MULTIPLIER` is 5
(`libslic3r.h:68`). The normal-height scaling-remainder predicate is cited
as written; this spec does not infer a minimum positive height or claim a
reachable rejection from it without a Native witness.

## Existing Rust and harness boundary

Core already stores these options in the object/print/region source options.
Its existing counterparts include `options/validation/basic.rs`,
`options/validation/extrusion_width.rs`,
`print/validate/extrusion_width.rs::{applied,layer_heights,width}`, and
`project_slice/group_fills/params/locked.rs::append`. They remain unchanged.
Any newly observed production discrepancy blocks this harness slice and is
reported to its source owner.

The destination is `crates/ares-cli/tests/orca_parity/option_coverage/`:
`domains.rs`, key-local probes/context activation, width metadata, and dedicated
G3 E2E files. The harness is an admission adapter around the cited upstream
options, not an independent slicing pipeline. Reuse existing export,
application verification, OrcaRunner slicing, cache and full-output comparison.
Do not add a general context framework, production API, cache or Native build.

Current height probes sample 0.05/0.36 and remain INCOMPLETE. Their comment
requiring strictly less than 0.4 conflates height and other width constraints.
Locked widths already have seven cases each and real sparse-extrusion/full-byte
E2Es, but retain an unverified global-range omission. The intended PASS means
the explicit finite context below, never all nozzle/region combinations.
Latest full 650-domain observation remains `acf100550`; a targeted G3 run
must not be reported as a fresh full matrix.

## Included finite contracts

Use the existing centered cube10/Ender-3 fixture export path. Verify every
requested dependency survives Native export and the Ares effective configuration.
Record dependencies and deferred scope beside the domain result.

### Skin and skeleton widths

Retain the current one-nozzle 0.4mm, layer 0.2mm, relative extrusion,
LockedZag 40% context and all seven existing cases per key: literal zero auto,
positive near-lower width 0.20001mm, inclusive upper 2mm, seeded mm interior,
near-lower 50.001%, inclusive upper 500%, and seeded percent interior.
Preserve actual sparse-extrusion observability and the assertion that different
effective widths change extrusion. Full ordered output equality remains mandatory.

Add a one-nozzle 0.6mm, layer 0.3mm control with 0.30001mm/3mm and
50.001%/500% requests, plus literal zero auto. This witnesses that the physical
upper endpoint follows nozzle diameter rather than a hardcoded 2mm. It is a
dedicated dependency control; the generic domain sweep retains its declared
0.4mm/0.2mm context. Verify the altered nozzle and layer height explicitly.

Keep existing Native rejection controls intact: 0.2mm and 50% are too small,
2.01mm and 501% exceed the upper bound, and 0% reaches the negative-spacing
Flow failure. Preserve stage, exit status, diagnostic and evidence assertions.
These are rejection witnesses, not successful full-output comparisons, and do
not count toward the successful case denominator. Literal zero and zero percent
must never be merged in metadata or application checks.

### Normal and initial layer height

Use one 0.4mm nozzle, no raft/support, and keep the other height at 0.2mm.
Explicitly set ordinary nonbridge widths to 0.6mm and thick bridge settings
to isolate the height predicate from unrelated thin-width rejection. Retain
the bridge-width cap and verify these settings; do not globally relax validation.

For each key, request 0.05, 0.2, 0.36, 0.39999 and 0.4mm. Verify requested /
effective heights, real emitted layer heights/Z schedule, and complete Native /
Ares output equality. The 0.4 case is the inclusive nozzle boundary. Keep the
existing pure validation controls unchanged, including their documented coupled
width rejection for the unmodified normal-height equality input.

Add runtime Native rejection witnesses for 0 and 0.40001mm. Record the actual
loaded/applied failure stage, code and diagnostic separately from successful
comparisons. Pair the same merged input/options with the existing public Ares
validation boundary where possible; do not introduce a production entry point.
An unexpected Native rewrite/rejection or disagreement is reported before any
scope change. No smallest-positive-float claim or exhaustive open-lower-bound
coverage follows from these finite cases.

## TDD and expectation changes

Implement vertical sub-slices: the two LockedZag widths, then the two heights.
Each starts with a separately committed Native-backed RED: exercise admitted
requests at runtime, verify effective values and full ordered output, then
assert the bounded domain classification is PASS. Existing INCOMPLETE metadata
must fail that last assertion. If bytes differ first, preserve that real RED and
report it; do not promote the classification around a divergent stream.

GREEN changes only these four keys' authoring, context metadata and bounded
classification. Preserve every other domain's generated plan. No tolerance,
footer-only comparator, extra normalization, ignore, probe or rejection masking.
Native artifacts remain outside the repository, through existing cache paths.
Missing dependencies use shared native_environment: optional visible SKIPPED,
required FAIL.

Explicit expectation updates, retained as assertions:

- In `numeric_metadata_tests.rs`, change only the `layer_height` and
  `skin_infill_line_width` negative locks from `!provably_legal` / omission
  present to bounded-positive classification / omission absent. Preserve the
  `filament_map`, `filament_self_index` locks and printable-area positive control.
- In `locked_width_plans_cover_context_edges_and_seeded_interiors`, retain all
  edge/interior checks and replace only the negative classification expectation
  with the declared bounded-positive expectation and explicit deferred context.
- Keep `limited_width_contexts_stay_incomplete_when_all_samples_match` active
  and nonvacuous: use an explicitly unproven limited-plan control and assert
  matching samples still yield INCOMPLETE. Do not leave an empty filtered loop.

## Acceptance and deferred scope

For each sub-slice: required cold/hot directed tests under seeded and official
producers; unfiltered selected-key sweeps account for every authored successful
request; negative witnesses retain their evidence. Fresh seeded/official
base/candidate control comparisons verify common Ares raw bytes unchanged and
list all status changes. Default environment full workspace, strict workspace
clippy, fmt and wasm check must pass. Report RED logs and all metadata assertion
changes, plus an inventory diff proving the other domains unchanged.

Deferred: mixed nozzles/SEMM/tool ordering (G4), raft/support nozzle selection,
object/region overrides and layer ranges, adaptive/painted layer height,
bridge LockedZag flow reuse, every FloatOrPercent parser/tolerance edge,
malformed values, all printer geometries and arbitrary Flow feasibility.
G2 clearance/height domains and printer_technology remain separate slices.
Any unexpected Native admission or production parity failure requires reporting;
this spec does not permit reducing requests or repairing core behavior.

## Width slice conclusion

RED `8c0179910` and GREEN `dc968c79c` establish the two width domains' declared
bounded scope. Each canonical seven-request set and the ten-request second
context produce actual sparse extrusion and complete Native/Ares equality.
Runtime Native retains all five rejection controls per key, including the
distinction between literal-zero auto width and zero-percent Flow failure.
Seeded and official cold/hot runs also execute both unfiltered domain sweeps:
14 generated, 14 strictly compared, zero rejected; both scoped rows are PASS.

The 650-row plan inventory changes only these two classifications; all authored
case values and the other 648 rows are unchanged. Width metadata names the
physical context and deferred behavior. The negative metadata lock for skin
width becomes a bounded-positive assertion, alongside skeleton width; normal
layer height, filament map/self-index and the printable-area control retain
their previous expectations. The explicitly limited/unproven countercontrols
still assert INCOMPLETE and cannot become an empty loop.

Fresh base/candidate controls preserve every common Ares raw stream: 533
seeded keys and 522 official keys. Each producer's sole existing test-state
change is the selected width sweep FAIL to PASS; the additional second-context
test passes. No production source, comparator, cache or input fixture changes.
This is not a fresh 650-domain execution or a claim about all Flow contexts.
The two layer-height domains have their separate Native-backed slice below.

## Height slice conclusion

RED exercises all five requests per height key before
asserting their classification. All ten complete Native/Ares streams match;
only the two INCOMPLETE-to-PASS expectations fail. In particular, runtime
Native accepts both 0.4mm nozzle-equality requests, and preserves 0.39999mm.
The emitted HEIGHT/Z sequences are recorded with each complete comparison.

GREEN changes only the two height domains' finite authoring and
classification. Their dedicated activation sets one 0.4mm nozzle, ordinary
widths 0.6mm, bridge width 0.4mm, thick bridges, no support/raft, and the other
height 0.2mm. Both dedicated comparisons and the unfiltered selected sweep
verify the exported dependencies. The 650-plan inventory has exactly two
changed rows; all other 648 remain identical. This closes the declared finite
scope, not all physical layer heights or the entire 650-domain matrix.

Runtime rejection witnesses remain separate from successful comparison counts.
For each key, zero fails Native Export with -18 and its exact
`key: invalid value 0.000000` diagnostic; 0.40001mm fails Slice with -51 and
`Layer height cannot exceed nozzle diameter.`. On Unix the captured statuses
are 238 and 205. Existing public Ares validation and validated slicing reject
the corresponding override with the same diagnostic. No production guard,
validation relaxation or narrower request set is introduced.

Seeded and official required cold/hot directed runs each pass all five tests,
including the two-domain sweep's ten strict comparisons and four rejection
witnesses. Cold runs have 42 misses and two within-run hits from reusing the
successful 0.2mm baselines; hot runs have 44 hits. Every common Ares raw stream
is unchanged across RED and GREEN, both producers, and the assertion-helper
extraction needed to pass strict clippy. That extraction preserves both public
rejection checks and their exact error-type/message assertions.

The `layer_height` negative metadata lock becomes a bounded-positive assertion
alongside `initial_layer_print_height`. Filament map/self-index negative locks,
the two LockedZag width assertions, and the printable-area positive control
remain unchanged. The existing coupled-width/nozzle validation controls also
remain active and unchanged. Native outputs, cache records and execution
reports are not committed.

Fresh base/GREEN controls give 863/864 to 867/867 PASS for
both seeded and official. Their 530 seeded and 519 official common input
keys have unchanged Ares raw SHA sets. The only existing test-state change
is the selected height sweep FAIL to PASS; the three new height tests pass.
Default environment workspace passes 8237/8237, and strict clippy, fmt and
wasm check pass.

Official concentric Native M73 ND (accepted independently of tree): normalized Native SHA `5da48f589ee04c6ec39a2e07ed5d55fd14fe90b2b77a4ba12d2d844c8731d908` / `7af8c7aaad37fc66287511c7284318241a7a9523d34ad57d17f658a8ad8b71a5`; first difference line149 (`G1 X111.401 Y106.013 E.11356` / `M73 P10 R7`), all seven Ares raw captures identical.

An external comparison script originally compared raw
SHA lists including repeated captures. One base capture versus two identical
candidate Ares captures therefore falsely reported a byte change. Its corrected
audit compares distinct raw SHA sets and retains capture counts and Native
variants separately. No repository comparator changes. Independent cold base/candidate
concentric repeats use budget one: base PASS/PASS, candidate PASS/DIVERGENT,
same producer components and identical 3MF member bytes. Ares independently
slices the Native-exported 3MF (`artifacts.rs:131-140`); it never consumes the
reference G-code, and Native does not consume candidate code. Native's original
candidate capture diverges, then its unchanged-project retry matches in full.
Full Native diff also changes final top-surface concentric traversal and a tiny
center-path survivor; M73 placement is the earliest difference, not the only
difference. Their upstream root cause is not established or labeled UB.
