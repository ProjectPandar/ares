# SEMM ramming profile sweep integration — approval spec

Status: **PROPOSED; no heavy task before approval**.
Date:2026-10-11. Base main:6e4c6c843 (test-only advance, not production).
Next priority from [gap audit](option-coverage-gap-2026-10-11.md).

## Source-owned goal, not a new ramming implementation

Integrate bounded `filament_ramming_parameters` coverage into
`option_coverage::domains` / `sweep`, reusing the existing SEMM fixture recipe.
The current generic coStrings plan falls through `domains.rs` to a None-valued
baseline: it cannot activate two logical filaments and their unload profiles.
The key is **not globally untested**: existing
`runner/filament_map_ramming.rs::semm_ramming_profile_changes_tool_change_block`
(:138-235) proves Native toolchange/profile-swap activation and compares the
original distinct-profile case to Ares. It does not itself register an authored
sweep domain or compare both original and swapped inputs through validated API
and real CLI. Preserve that test and every current assertion.

This slice is test/accounting only unless a newly admitted witness identifies
an actual upstream port mismatch. Any production correction needs separate
owner approval naming the specific upstream operation and Rust destination.
No invented optimizer, tower geometry algorithm, extra validation/fallback,
material scheduling/build, Native golden or acceptance tolerance is authorized.
Worker2's wipe-tower production ownership remains unchanged.

## Exact upstream owners / destination boundary

| Upstream source | Included behavior | Rust destination / observation |
| --- | --- | --- |
| PrintConfig.cpp:2833-2840 coStrings definition | Default serialized ramming profile, one string per logical filament | Existing typed filament option; CLI tests supply explicit two-row vector |
| Print.cpp:3581-3628 `WipeTower2` construction / set_extruder / toolchange planning | Route real two-filament SEMM toolchanges to the active tower | Existing `ares-core/project_slice/gcode_emit/wipe_tower` compatibility shell; no alternative builder |
| GCode/WipeTower2.cpp:1248-1250 | m_semm and m_enable_filament_ramming gates from config | Existing tower construction/gates, observed via Native motion |
| WipeTower2.cpp:1357-1390 `set_extruder` | Per-filament SEMM loading/cooling fields, first two percentages, speed floats until stream stops at pipe | Existing `wipe_tower/filament.rs:104-122` reader; preserve source f32 semantics |
| WipeTower2.cpp:1643-1657,1707-1713 unload path | Width/step multipliers, gated ramming, speed ×0.25s → volume → E-motion | Existing tower unload/ramming emission; any mismatch gets its own operation proposal |
| WipeTower2.cpp:2228-2232 plan_toolchange sizing | Accumulate0.25s flow samples into ramming length/depth when enabled | Existing tower planning, not an Ares-owned allocation policy |

The BBL `GCode/WipeTower.cpp:1591-1601` lookalike reader is inside `#if0`.
Do not cite it as the active parser or use an H2D dual-hotend fixture as a SEMM
substitute. Nozzle flush dataset belongs to a different CLI predictor boundary
and is deferred to the following spec, not bundled into this one.

## Established fixture and admission before tests

Use the existing **Ender-3 Standard0.4, two cube10 objects, object→filament1/2**
recipe from `runner/filament_map_ramming.rs`; two distinct PLA colours,
filament_map `[1,1]`, one physical nozzle, filament_is_support `[0,0]`.
Preserve stock single-extruder commands and loading/cooling parameters.
Prime tower anchored150/150 inside220×220 bed; no support/raft/TPU, fixed
seed/generator metadata and unchanged geometry/tool ordering.

Offline retained original input at
`/tmp/worker3-master-full-496-v1-seeded-cand-artifacts/case-PhFtBr/input.3mf`
shows `single_extruder_multi_material=1`, `enable_filament_ramming=1`,
`enable_prime_tower=1`, `purge_in_prime_tower=1`, both requested profiles,
filament_map1/1, one0.4 nozzle. This is a **historical input recipe**, not a
fresh admission/golden or proof for the new matrix. Record its hash/source if
reused, then run fresh Native against the exact rewritten bytes after approval.
Do not build a second exporter: retain the admitted two-object3MF and patch
only permitted config fields using existing CLI test fixture conventions.

Required application proof before any Native comparison: two profile strings
and both logical filaments survive input construction; per-object material
assignments retained; all other ZIP entry bytes identical; whole settings equal
except the declared profile vector and, for off controls, the one gate.
If export/case construction collapses or normalizes a request, stop with
APPLICATION-MISMATCH. Do not count a defaulted/inactive value as coverage.

## Explicit proposed profile strings

D is the PrintConfig default, unchanged:

```text
120 100 6.6 6.8 7.2 7.6 7.9 8.2 8.7 9.4 9.9 10.0| 0.05 6.6 0.45 6.8 0.95 7.8 1.45 8.3 1.95 9.7 2.45 10 2.95 7.6 3.45 7.6 3.95 7.6 4.45 7.6 4.95 7.6
```

S is the already-used shortened runner profile:

```text
100 100 1 1|0.05 1 1
```

W keeps D's width/step percentages but S's speed samples:

```text
120 100 1 1|0.05 1 1
```

W is a **new, unadmitted request**. It isolates the100→120 width multiplier
from the speed samples. No malformed/empty/nil/negative/zero-percentage profile
is proposed. Free-form coStrings has no finite global min/max/full-domain
claim: this is only the declared bounded set in the admitted context.

## Proposed finite matrix: five active requests + two gate controls

| Label | Profiles for logicalF0/F1 | enable_filament_ramming | Obligation |
| --- | --- | --- | --- |
| R0 | `[D,D]` |1| Default/default baseline; real tower toolchanges and E-motion |
| R1 | `[D,S]` |1| Existing distinct-profile recipe, now full validated API+CLI comparison |
| R2 | `[S,D]` |1| Swap only profiles vsR1; Native ramming E-motion changes, both full streams compared |
| R3 | `[S,S]` |1| Symmetric short-profile control; accepted/sliced with both materials |
| R4 | `[D,W]` |1| Width multiplier isolation vsR1 while speed samples/step stay fixed forF1 |
| C1 | `[D,S]` |0| SameR1 except one feature gate; no enabled ramming moves |
| C2 | `[S,D]` |0| SameR2 except gate; no Ramming comments or unload ramming E fragment; profile-dependent later wiping XY remains upstream behavior |

These are proposed requests, **not newly executed Native facts**. All seven
must be freshly admitted; if Native rejects tower footprint, crashes, or fails
to activate the intended path, preserve every stream/receipt and request a
revised spec. Do not silently change tower placement, models, purge volume,
profile strings or expected outcome to rescue the case.

R1/R2 profile swap must show a non-header extrusion-motion difference, not
merely config serialization or a different comment. R1/R4 must demonstrate
Native width-dependent ramming path/E motion.

Native admission at actual664b387d8 found both off cases succeed but differ
in795 extrusion-motion XY lines. Approved correction: C1/C2 require no Ramming
comments and no unload ramming E fragment, **not equal overall motion**.
WipeTower2.cpp:1643-1645 computes profile line_width/y_step unconditionally
before do_ramming (:1655; local guard :1657); :1834-1844 retains y_step in the
post-unload set_position even when ramming is off. Subsequent wiping therefore
retains a profile-dependent XY offset. Preserve this upstream fact and the
original failed equality audit; input/profile/gate values remain unchanged.
Motion extraction is only supplementary activation proof. Every individual
Native/API/CLI acceptance comparison remains the **entire ordered GCode**,
including config headers; no sections are dropped to compare off controls.

## Test-first implementation after approval

1. Sequential fresh Native admission under load<60/jobs2/threads2. Before each
   start, last exact exclusive START must have END; watch new exact markers,
   kill owned process groups and retain VOID on exclusivity. Pin standard
   producer/seed/CPU and record actual repo/input/binary hashes.
2. Commit admitted input3MF/provenance only (never Native output goldens), then
   bounded E2Es at validated public byte API and real CLI seams. Write tests
   before any production correction. Compare original and swapped cases fully,
   plus every other declared active/off request. Direct GREEN is acceptable;
   report it, do not force RED.
3. Register authored cases + domain metadata for `filament_ramming_parameters`
   through existing domains/sweep special dispatch, as with the prior identity
   slice. Every required case carries Some(profile vector), explicit gate and
   contextual proof; full7-case execution is the bounded PASS obligation.
   No new admission feature flag or generalized fixture/scheduling framework.
   Required Native absence fails closed; filtered partial domains keep existing
   provably_legal=false/INCOMPLETE behavior; no None-baseline completion credit.
4. Keep existing `runner/filament_map_ramming.rs` and all its original Native
   toolchange/block count/swap/Ares comparison assertions. Prefer input-only
   fixture reuse, leaving that runner test unchanged. Any unavoidable mechanical
   input-helper move must be separately authorized/compilable and preserve
   every assertion; no duplicate production or exporter path.
5. Run seven named cases plus selected authored sweep with standard seeded and
   official references, fresh per-mode artifacts/caches, load<60/jobs2/threads2.
   If divergence appears, report exact first difference and upstream owner;
   no production change until a separate operation-specific owner approval.
6. Engineering under load<30/jobs2/threads2: workspace nextest --no-fail-fast,
   5400s; strict workspace/alltargets/allfeatures Clippy, fmt, core/WASM.
   Full or any test-only carry requires new explicit owner authority; do not
   inherit the prior6e4 full waiver automatically. No push without exact approval.

Repeatable artifacts per case: input/hash + full-settings/ZIP control proof,
Native producer/command/exit/signal/stdout/stderr/result, complete Native,
validated-API and CLI GCode, real CLI binary hash/channels, whole-comparison
outcome and Native activation facts. Existing generator-only normalization and
wall-clock result exemptions remain exactly unchanged. Preserve all failures,
build failures and interrupted/VOID attempts, with actual execution hashes.

## Exit criteria / boundaries

- All7 admitted requests pass both public seams and whole-output Native
  comparison; R1/R2 and R1/R4 have causal Native motion evidence; C1/C2 have
  no ramming comments/unload ramming E fragment while retaining residual XY dependence.
- Authored sweep reports the declared finite context only, no global domain
  completion claim; existing assertions/acceptance unchanged.
- CLI tests/fixtures own Native/filesystem/process behavior; core stays platform
  neutral and browser-WASM capable. No UI/OpenGL/renderer or postprocessor added.
- All touched/new files<400LOC; no add/modify formatting pairs; existing Ares
  tower scaffolds remain temporary shells around cited WipeTower2 operations.

Deferred: arbitrary malformed profile text/GUI curve semantics after pipe,
invalid bounds, all filament counts/vector broadcasts, non-SEMM/multitool
ramming, tower types/geometry generalization, mixed diameter/materials,
Auto mapping/TPU, cooling algorithm changes, nozzle dataset predictor sweep,
worker5 inherits and Native crash clusters. No new Ares ramming policy.

**Approval requested:** this7-request Native admission/test-first sweep
integration only. Until approved, remain offline/0heavy/0production changes.
