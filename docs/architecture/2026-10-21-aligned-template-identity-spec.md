# AlignedRectilinear: template identity is not the layer-angle override

Status: COORD accepted the read-only discovery and authorized this docs-only
SPEC followed by a separate input-only RED, on an independent branch from
main `47821c760d5ea86102a34bbc78c092bbaf381214`. GREEN is NOT authorized;
SPEC+RED require independent content review. CPU12, no FULL/integration window.

## Owning upstream rewrite slice

Pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/Fill/Fill.cpp:937,941,1136`: set
  `SurfaceFillParams.fixed_angle` from the actual sparse/solid rotation
  template's nonemptiness. An aligned pattern does not force this grouping bit.
- `Fill.cpp:288,316`: ordered-set comparison/equality retain this identity;
  numeric angle equality does not erase explicit-template provenance.
  `1245,1450` carry that bit into the filler.
- `Fill/FillRectilinear.hpp:38-46::FillAlignedRectilinear::_layer_angle`
  returns0 independently of template identity. `FillBase.cpp:308-312` calls
  the class layer-angle override when explicit/bridge/other gates allow it.
- GUI reachability: `PrintConfig.cpp:2074-2113` lists alignedrectilinear for
  top/bottom/internal-solid patterns; `4007-4016` defines the advanced Strength
  `solid_infill_rotate_template` String field. `GUI/Tab.cpp:2493` exposes it;
  `GUI/ConfigManipulation.cpp:687,1006-1009` handles the solid/template fields.

There are TWO source-owned concepts: explicit-template grouping identity and
an aligned class's zero alternation. The Rust destination is exclusively:

1. `ares-core/src/project_slice/group_fills/params/projection.rs:120-124`:
   preserve only the existing projected template-nonempty bit; do not OR the
   AlignedRectilinear pattern into it.
2. `ares-core/src/project_slice/fill_entities/monotonic.rs:65`: carry the
   aligned class's zero alternation via the existing `layer_angle_fixed`
   adapter. `fill/rectilinear/surface.rs:18-22,190-193` already separates
   that input from template `fixed_angle`.

These Ares adapters are temporary compatibility shells around the cited Fill
classes/parameters; this slice replaces a conflated handoff, not an Ares-owned
pipeline design. No new parameter family, helper, API or configuration is needed.

## Non-negotiable scope / queue separation

A future independently approved GREEN is only those two adapter changes.
**Do not remove fixed_angle from the comparator** or otherwise alter grouping,
first-owner machinery, Flow, kernels, union/clipping, phase/graph, emission,
M73/time/feedrate/E/wipe/retraction/footer behavior. No new allow/expect,
tolerance, filter, inverted assertion, Native golden, fallback or guard.

Freeze ironing and all concentric code/helpers: W4 owns concentric ironing.
This witness uses ordinary unpainted normal+modifier regions and aligned line
fill, not W1 painted, W2 brim/EFC, W4 concentric sparse, W5 spiral/processor/
lightning, or W6 seam occlusion/zero-area orientation. None of those queued
source boundaries is included. Tier1 remains browser WASM/Windows/macOS/Linux;
no filesystem/UI/terminal/OpenGL behavior enters core.

## Observed strict witness, not a later-motion substitute

**The FIRST strict failure involves M73/progress ordering** at byte1592/line69:
Native `G1 Z.2`, API/CLI `M73 P5 R15`; normalized lengths176519/168422.
Do not hide that early failure or report only a convenient later infill move.
M73 is a downstream witness, not the repair site.

Authored75-layer overhang, normal part plus one GUI modifier; actual geometry,
placement, transforms and every non-settings ZIP member are unchanged from
`tests/parity/fill-group-flow-ratio-key/equal-density.3mf`.
Top/bottom/internal-solid patterns are alignedrectilinear, global solid direction
45 degrees, global solid rotation template empty; modifier template45.
Both templates resolve to the same numeric angle, but upstream keeps separate
false/true grouping keys. This is a GUI setting interaction, not painted data.

Accepted canonical GDB snapshots at actual physical layer IDs0 and74 show:
- Native: two owner/region groups0/[0] fixed=false and1/[1] fixed=true.
- Rust post-coalesce: one owner0/[0,1] group fixed=true.
- Sampled angle0.7853981852531433, spacing/overlap/density, Flow width/height/
  nozzle and role speed match. Region0's identity bit is the observed mismatch.

Uniform aligned control has one Native false group and one Rust true group,
but complete PASS: uniform coverage hides the identity conflation. The target
makes it change real membership. Native snapshots are after group_fills; Rust
snapshots precede priority. Do NOT claim unmeasured exact geometry equivalence
across those stages, sole-cause proof, or guaranteed future GREEN closure.

## Frozen RED inputs and direct public seam

Use ONE positive and TWO sole-key controls, directly reusing the discovery ZIPs:

| Case | Only change relative to the positive | Discovery both modes |
|---|---|---|
|aligned-local45|positive: global empty / modifier45|strict DIVERGENT|
|aligned-global45-local45|global solid rotation template empty→45 only|strict PASS|
|aligned-local90|modifier solid rotation template45→90 only|strict PASS|

The global-template control has uniform explicit identity; the local90 control
has numerically different angles, so the normal angle key prevents merging.
Each control changes exactly one GUI key at one scope; all other settings and
model data stay frozen. Other discovery cases are evidence, not added RED scope.
No Native output or numerical Native constants are checked into tests/fixtures.
Input SHA identities and upstream source citations are allowed.

Register three thin, unconditional calls to the existing shared
`native_stream_case::check` (or the already approved case_runner). No new runner,
normalizer, comparison policy, observer framework or copied harness. The shared
check drives fresh canonical Native and a second replay, then public validated
API and CLI whole-output equality with the existing generator/date-only policy.
No production/core source edits, helper edits, old test/assertion edits, ignores
or weakened expectations in RED. Include frozen input-only3MFs and their README.

## Stage exits

1. Commit this SPEC alone from main47821c760, then separately commit input-only
   RED per COORD's explicit combined-stage authorization. Keep distinct commits.
2. Exact RED: one intended strict FAIL and two controls PASS per official/seeded
   mode. Existing W3 fifty shared checks remain PASS with retained raw triples
   and activity. Capture fresh producer command/status/input/output/binary
   evidence; Native repeats and API/CLI raw equality must be stable. Source
   admission/effective Native footer/activity and group identity remain
   independently auditable outside tests, without Native golden assertions.
3. Run fmt, locked workspace/all-target Clippy with-Dwarnings, locked core+wasm
   wasm32 gates after test registration. CPU12/jobs1/incremental0/threads1.
   No FULL, rebase, merge or push. Send SPEC+RED RR, then STOP for review.
4. No GREEN before new authority. Any separately approved bounded implementation
   must preserve approved RED/helpers/inputs and stop/report on any strict
   residual; adjacent repair or output normalization expansion is not allowed.

Accepted discovery, hashes, source/GUI citations, raw producers,24 Native +12
CLI +12 independent API runs and four canonical diagnostic streams:
`${EVIDENCE_ROOT}/w3-aligned-template-readonly-47821c760/README.md`.
Failed optimized readers and the initial76-vs75 priority-event labelling mistake
are retained/excluded there; accepted readers use actual generation layer args.

## Approved sparse-anchoring class-angle amendment

COORD independently accepted the read-only diagnosis at candidate
`26c54635079492678df1ea5e31af8ed6834c0115`. That three-file attempt closed
the official 53 selected checks but stopped on ten of 83 official neighbors;
it was not GREEN. Its explicit authorization also included the existing
`fill/rectilinear_multiline.rs:35` class-angle guard.

Extend the source-owned boundary to
`Fill.cpp:1394,1418,1441-1450,1497`,
`Layer::generate_sparse_infill_polylines_for_anchoring`: its filler factory
constructs the same `FillAlignedRectilinear`, whose class layer-angle is zero
(`FillRectilinear.hpp:44`, `FillBase.cpp:308-312`). The Rust destination is
`prepare_infill/bridge_over_infill/sparse_anchoring/linear.rs::append`.
This existing compatibility adapter must set `layer_angle_fixed` from
`pattern == ProcessInfillPattern::AlignedRectilinear`, replacing only its
literal false. Preserve `fixed_angle: fill.params.fixed_angle` unchanged.
The existing by-lines and multiline consumers already accept this class bit;
no kernel, dispatcher, helper or API change is needed.

All ten failed inputs actually reached this adapter as aligned count1 at
layer15/thickness1 with template=false, class=false and dont-alternate=false;
seven were count1 baselines before their requested multiline leg. Official
Native actual Aligned vtable returned class angle0 at corresponding idx15.
Normal aligned filling already supplied class=true. Thirteen observer complete
streams matched the canonical failures. This does not guarantee later closure.

This amendment supersedes the earlier two-adapter-only scope solely for the
authorized existing multiline guard and this one sparse-anchoring assignment.
Freeze all approved tests, inputs, SOURCE, helpers, comparator, ironing,
concentric, other algorithms, Flow and emission. Separate docs and one-line
production commits are required. Verify all 53 target/control checks and the
full 83 aligned/rectilinear/infill neighbors in both official and seeded modes,
plus fmt, locked workspace/all-target Clippy-Dwarnings and browser wasm32, on
CPU12. No FULL. Any residual means STOP, not adjacent repair.

Read-only source map, actual parameters/stacks/vtable return, canonical streams
and independently approved amendment RR:
`${EVIDENCE_ROOT}/w3-aligned-class-angle-readonly-26c546350/README.md`.
