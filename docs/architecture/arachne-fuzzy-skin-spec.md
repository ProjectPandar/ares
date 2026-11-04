# Arachne fuzzy skin: proposed junction rewrite slice

Status: accepted. SPEC 308583e3a and group-dependency decision 791a9fbe9; RED first.

## Execution split and order

The stochastic oracle distinction requires two slices:

- F1: single-group whole coverage, all noise variants and three modes, Ripple,
  raw source eligibility, Context/draw sequence and actual slice_z. Only F1's
  small production boundary lands after its own public RED and alternate merge.
- F2: compatible multi-group effect masks, identical-effect merge/priority and
  LineSplit ordered splits. Separate fixtures/RED and production GREEN.

Neither slice inherits acceptance from the other. Until alternate extra wall
is merged, only
Native closure/min-width preflight and F1 fixtures/public RED may
proceed.
No F1 partial implementation may silently treat multiple groups as one, discard
non-owner configurations or add a new Unsupported guard. Any reachable
multi-group dependency preventing a clean F1 boundary needs an explicit decision.

F1 GREEN development may proceed before the alternate extra wall
merge.
documented in [F1/F2 boundary addendum](arachne-fuzzy-f1-group-boundary-addendum.md)
before production integration. Its order is:
commit/report F2 public Native RED, commit F1 noise/junction production, commit
F2 masks/LineSplit production, then full parity once on the complete
stack. F1 and F2 merge together; no partial F1 merge. The earlier independent
F1 GREEN sequencing is superseded by this dependency decision.

## Upstream ownership and source identity

OrcaSlicer v2.4.2 owns this behavior:

| File/function | Source lines | Included contract |
| --- | --- | --- |
| `PerimeterGenerator.cpp::process_arachne` | 2093–2095,2125 | group fuzzy configurations before walls; active contour fuzzy disables arc-fitting simplification reduction |
| `PerimeterGenerator.cpp::traverse_extrusions` | 378–387 | mutate Arachne junctions before clipping/conversion; open extrusion counts as contour |
| same | 418–435 | fuzzy contour/hole flags participate in odd-layer overhang reversal before steep detection |
| `Feature/FuzzySkin/FuzzySkin.cpp::group_region_by_fuzzify` | 415–473 | full compatible-member config equality, scaled values, active flags, single-group whole coverage and safety-offset multi-group masks |
| `should_fuzzify` | 476–492 | type, first layer, inset and contour/hole eligibility |
| `PrintObject.cpp::region_config_from_model_volume` | 3664–3709, especially3707–3708 | after object/volume/material/range overrides, thickness<0.001 or point distance<0.01 normalizes region fuzzy type to None |
| `get_noise_module`, `random_value` | 26–65 | uniform noise or existing libnoise modules; random spacing consumes mt19937 draws even for coherent noise |
| `fuzzy_extrusion_line` | 345–412 | random resampling, three modes, destination width/index, small-output padding and endpoint repair |
| `fuzzy_extrusion_line_ripple` | 223–291 | adjacent-junction arc resampling, deterministic phase, destination width/index; separate dispatch from three-mode random branch |
| ripple phase/anchor helpers | 89–156 | layer-group shift and leftmost Y=0 crossing, fallback/closest-arc order |
| `same_fuzzy_effect`, `collect_merged_fuzzy_regions` | 502–558 | eligibility first, merge identical effects without artificial boundaries |
| Arachne `apply_fuzzy_skin` overload | 685–814 | whole coverage or exclusive masks, ordered line splits, endpoint isolation and closure reconstruction |
| `FuzzySkin.hpp::fuzzy_extrusion_line` | 12 | whole-line call defaults `closed=true`; split-segment call explicitly uses false |
| `Algorithm/LineSplit.hpp::SplitLineJunction`, `split_line` | 9–64 | source-index identity of old/new intersection junctions |
| `Algorithm/LineSplit.cpp::do_split_line` | 64–314 | ordered source-aware region splits, not an unordered intersection-only replacement |
| `Arachne/utils/ExtrusionJunction.hpp` | 19–40 | position, scaled integer width and perimeter index |

Local checked source SHA256s:
`PerimeterGenerator.cpp`: `45e0a6c56df0acf4e66ae82ae7a4aa4792dc558fb263a12b1211438de47b7de6`;
`Feature/FuzzySkin/FuzzySkin.cpp`: `6508c0a268fc15f1a2e6742c7e5c6e03c9a41ee4f24f73dfe70375a45f73e20c`.
The local source includes the seed-replay initialization
at random_value:28; record official versus replay producer identity separately.
Do not claim that official accepts ORCA_REPLAY_SEED or has identical entropy.
The ignored OrcaSlicer symlink stays outside Git.

## Exact Rust destination

Own the junction/config/group transaction in new private
`crates/ares-core/src/project_slice/perimeters/arachne/fuzzy_skin.rs` and focused
children for junction noise/ripple and region splitting as needed below400LOC.
Replace the fuzzy rejection in `arachne/config.rs` and `arachne/traverse.rs`
only after an admitted public RED has been committed and reported.
Use `arachne/materialize.rs` and `project_slice/perimeters.rs` to pass the
actual layer slice_z, source config-group view and mutable SlicerContext into
the existing Arachne traversal. The Context must not be reconstructed per wall.
Keep traversal ordering, variable-width conversion, entity roles and G-code
emission as downstream upstream-owned consumers, not new Ares pipeline stages.

Existing `PerimeterGroup.members`, source region surfaces and record_groups
own compatibility membership. Expose only the needed private read view from
`perimeters/types.rs`/groups; iterate actual members in their source order.
The fixed singleton compatible_region_ids field is not evidence that a group
has one fuzzy configuration: groups.rs can merge compatible region members,
and compatibility deliberately omits fuzzy settings. Group by the full upstream
FuzzySkinConfig equality (`PerimeterGenerator.hpp:13–42`), then merge same
effects only after per-loop eligibility. Do not select an owner/first region
as the fuzzy config for all merged surfaces.
If the existing geometry splitter does not retain LineSplit source indices,
port its required private source-index split boundary in geometry with the
above file/function citations; do not reuse overhang width interpolation as
though it were the same contract. FuzzySkin.cpp:781–786 copies the source
junction width/index for an inserted crossing and changes its position.

## Existing scaffold and included behavior

`perimeters/FuzzySkinConfig` already maps type/noise parameters but omits the
Arachne mode. Its thickness>=0.001 and point-distance>=0.01 condition reflects
the upstream region normalization owner, not an independently invented skip:
PrintObject.cpp::region_config_from_model_volume:3707–3708 resets region type
to None below these thresholds before should_fuzzify. The initial SPEC's
"independent shortcut" interpretation was corrected by Native zero/sub-threshold
inactive controls and the source normalization call chain during F1 preflight.
Preserve this normalization before config grouping/active flags, not only inside
the later sample generator. should_fuzzify itself remains the source type/layer/
inset/contour predicate applied to the normalized config. Global footer settings
need not show the effective per-region type reset.
`PrintConfig.cpp:3561–3579` permits thickness0..2 and point distance0.01..5;
out-of-schema values remain public validation errors, not internal guards.

The Classic polygon fuzzifier and public Point2 prototype are compatibility
shells outside the Arachne junction owner. Do not convert junctions into a
polygon, fuzzify it and restore constant width. Reuse verified scaled arithmetic,
libnoise evaluation and Mt19937 distribution primitives where their contracts
match; the old independent Point2 hash randomness is not an Arachne oracle.

Include all six existing noise variants: Classic, Perlin, Billow, RidgedMulti,
Voronoi and Ripple. Non-Ripple modes Displacement/Extrusion/Combined must follow
FuzzySkin.cpp:381–391, including its literal min_extrusion_width=0.01 in the
scaled-coordinate expression and conversion to integer junction width; do not
reinterpret the literal as0.01mm or introduce a different width floor.
Ripple branches at348–352 before this mode switch and preserves j1 width/index;
the mode setting must not manufacture a Ripple extrusion-width modulation.

Preserve scaled thickness/step quantization, separately truncated interpolated
base and normal offset, sample-spacing leftover across segments, padding order,
closed-input endpoint position/width repair, and every junction perimeter index.
Open wall contour eligibility is `!is_closed || is_contour`; whole-line fuzzy
calls still use the upstream default closed parameter. Split segments explicitly
use closed=false and restore non-fuzzy boundary endpoints as the source does.
Do not infer different endpoint rules from the upstream parameter's name.

Include mixed compatible fuzzy configurations: safety masks, effect merge,
later-region overlap priority, source-ordered split/rebuild, unmodified outside
segments and upstream closure behavior. Existing admitted multi-volume inputs
must not receive a new fuzzy-specific guard. Imported fuzzy painting schema and
unrelated modifier loaders remain deferred, not silently claimed implemented.

Consume group has_fuzzy_skin/has_fuzzy_hole for simplification and reversal.
In the overhang reversal branch, preserve the exact source has_fuzzy_skin
conjunction before its contour/hole cases (PerimeterGenerator.cpp:422–428),
not an independently designed "any fuzzy wall reverses" rule. General overhang
detection/clipping guard removal is a separate later slice; this slice only
supplies the fuzzy flags and already-supported consumers it requires.

## Random context and oracle contract

Extend the existing [ARD-0068](ard-0068-seeded-slicer-context.md) Context ownership
to real Arachne junction consumption: task-local Mt19937, explicit seed,
continuous source-order draws and independent jobs. Pass slice_z for coherent
noise from the actual layer, rather than print_z or a guessed layer height.
The current materialize-before-Classic shell must not double-consume draws or
reset at per-record/per-wall boundaries. Demonstrate traversal/draw equivalence
with Native before accepting any necessary transaction-order adjustment.
Mixed Classic/Arachne records are retained controls, not permitted collateral
changes. No global RNG, thread-local ambient RNG, file I/O or OS entropy in core.

Proposed E2E evidence split, following the existing seeded-context distinction:

- Official and seeded: deterministic Ripple and inactive controls require
  unchanged full ordered-byte equality against validated core API and real CLI.
- Seeded replay: Classic/coherent random-spacing variants and all three modes
  require strict full-output equality; seed0 repeat/isolation and explicit seed1
  probes use independently identified replay producers and the public Context.
- Official active stochastic cases: Native admission plus source-derived public
  geometry/role/width witnesses and raw output identity are diagnostic evidence,
  explicitly not strict Native parity PASS. Do not normalize random coordinates,
  retry until chance matches, or claim a shared reproducible random stream.

This stochastic oracle boundary is accepted with 308583e3a,
consistent with existing ARD-0068. If all active
official stochastic cases must be raw-byte identical without a reproducible
producer/seed, report that gate unresolved; never weaken the strict comparator
or present the diagnostic projection as full parity.

## Public test-first plan and failure inventory

Write deterministic input-only 3MFs and public Native/API/
CLI tests first, and commit actual RED separately before production.
Use a small finite matrix that shares meshes but keeps these failures witnessed:

1. Closed prism with External/Hole/All/AllWalls and disabled/none controls;
   hollow and two-wall inputs distinguish contours, holes and inset eligibility.
2. First-layer false/true and thickness0/sub-threshold inactive controls, active
   arc fitting: distinguish upstream region normalization from raw global footer
   settings, and preserve the resulting eligibility/simplification flags.
3. Narrow neck/open Arachne wall and varying-width model: preserve width/index,
   source open/closed handling, junction padding and endpoint/loop connectivity.
4. Ripple phase0/nonzero, grouped layer offset, anchor crossing/fallback and each
   mode: deterministic emitted motion equality and unchanged width semantics.
5. Seeded Classic/coherent noise, each mode, seeds0/1: spacing/amplitude draw order,
   real slice_z, extrusion amount/width formulas and repeat/concurrent isolation.
6. Compatible multi-volume regions with equal and differing fuzzy effects,
   overlapping masks and untouched portions: no double-fuzz, first-region leak,
   artificial effect boundary or missing outside segments.
7. Existing supported overhang-reverse input with fuzzy contour/hole flags and
   first-layer/inactive controls; general clipping changes remain out of this slice.

Before declaring RED, preflight Native admission and actual open/closed path
behavior. Ripple223–291 does not contain the random branch's endpoint repair;
PerimeterGenerator.cpp:534–540 still asserts closed-loop connectivity. The
closure decision below resolves the demonstrated release/debug
discrepancy; preserve every assertion not demonstrated broken and never invent
a closing segment.
Preflight modes near low-width boundaries: keep the exact scaled integer floor;
if Native admits behavior needing a downstream variable-width consumer repair,
cite that owner and amend the scope rather than adding an arbitrary clamp.
No speculative unit test, implementation-derived expected RNG or saved Native
golden replaces the public artifact comparison. Finite F1 RED excludes F2
mask/split acceptance; no multi-group result is presented as F1 GREEN.

The old `active_fuzzy_skin_fails_closed` unit in project_slice/tests/perimeters/
arachne/traverse.rs asserts precisely the rejection to be removed. After public
RED, record a source-cited retirement and
retire it alone in a separate commit citing source and RED hash.
Every other assertion and existing fixture/comparator stays untouched.

## Gates, exclusions and sequencing

F1 closure addendum:
PerimeterGenerator.cpp:534–540's disconnected-path TODO and NDEBUG release
semantics make the Native-admitted open Ripple loop authoritative. Evidence:
`closure-width-audit.json`.
F1's production commit removes the mirrored closing debug assertion broken by
that input; adjacent-joint assertion removal additionally requires demonstrated
breakage. Preserve all other assertions, no invented closure/retag. Cite those
source lines, NDEBUG and evidence JSON in the production commit message.
The two Native Flow::spacing rejection cases(-100/156) leave the parity matrix;
they remain diagnostic preflight evidence and authorize no new Ares guard.

Included only the source-owned fuzzy configuration/group/junction behavior and
its required context/consumer seams above. Defer Arachne beading/stitch changes,
new painting import, public/Classic fuzzy feature expansion and general overhang
guard removal. No Ares-authored pipeline, new Unsupported guard, fallback or
success with omitted fuzzy geometry is allowed.

All changed/new files <=400LOC; fresh Native cache and artifact
root per run. Strict clippy, fmt and
WASM32 core/adapter must pass.
