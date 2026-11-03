# Equivalent ironing-region span collection

Status: docs-only SPEC, then separately committed registered public RED.
Base3fbad615e334a283596d0e6acff921f1d02804c6; new independent branch
worker6/ironing-equivalent-span-spec. CPU15, Cargo jobs1, test threads1; no FULL.
Production GREEN is NOT authorized by this SPEC+RED task.
Earlier approved4chains20e41 ->8dd ->360e9 ->0fb8 remain clean/frozen separately;
their existing W6 integration order/window authorization does not change.

## Upstream owner / exact Rust boundary

This is ONE source-cited libslic3r Layer::make_ironing rewrite slice, NOT an
Ares grouping pipeline, new infill architecture or independently designed policy.

Authority: `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1507-1700`, especially local
`IroningParams`, its operator< / operator== and equivalent-region [i,j) loop.
The downstream fill call/emission at1702-1735 is cited only to locate the seam;
its implementation is explicitly deferred/frozen.

Rust destination is ONLY `project_slice/fill_entities/ironing`'s span decision and
areas collection boundary: spans.rs::append_spans, areas.rs::ironing_areas and
strictly necessary Input/parameter adaptation in enclosing ironing.rs. The
current sorted per-record append is a temporary compatibility shell, not the
source operation. Replace that shell with native-equivalent span collection,
feeding the SAME existing clip/filler boundary. No new public API/framework.

## Included source contract / non-negotiable architecture

1. Collect/sort eligible ironing region parameters as Layer::make_ironing does.
   IroningParams operator< compares, in ORDER: extruder, just_infill, line_spacing,
   height, speed, angle, fixed_angle, inset. Pattern is NOT an ordering field.
2. operator== compares extruder, just_infill, line_spacing, height, speed, angle,
   fixed_angle, PATTERN and inset. layerm is NOT an equivalence field. Do not
   substitute region ID, geometry adjacency, approximate equality, epsilon,
   rendered angle, or an Ares-only tie breaker for this source contract.
   Upstream line_spacing/height/speed/angle/inset are DOUBLE parameter values;
   do not conflate the key with the later float filler angle. Any necessary
   key-type correction belongs only at this span decision boundary, not a broad
   float or filler rewrite. No new generic sorting algorithm/tie-policy feature.
3. After sorting, consume contiguous EQUIVALENT IroningParams as [i,j), then
   i=j (1636-1644,1734-1735). Do NOT append/fill each equal record separately.
4. Collect areas across ALL span members, preserving EACH member's existing
   ironing_type/top/bottom/internal-solid selection (1652-1688). Scalar equality
   does not replace per-member surface classification or region options.
5. Apply the cited union_safety_offset when j>i+1 OR internal infill was added
   (1690-1697), before the EXISTING shared layer-outline inset/intersection.
   A single-region span retains source behavior. Do not merge arbitrary
   neighboring regions or discard holes, replace whole geometry, guess widths,
   or invent a new union/offset implementation.
6. Feed the source-equivalent collected area to existing downstream filler;
   preserve source first-layerm ownership/region IDs and existing collection
   consumers. Strictly necessary local span/areas adapters only, no consumer
   workaround or independent emission pipeline. Existing per-record Input is
   compatibility scaffolding, not an architectural contract to preserve at the
   expense of Layer::make_ironing parity.
7. No legacy fallback, ignored residual, unsupported guard, input workaround,
   fixture/test/helper/normalizer edit during future GREEN. If any strict
   residual remains, STOP and request review instead of broadening scope.
8. Core stays platform-neutral WASM/browser, Windows, macOS, Linux. No native
   process, file I/O, dependency, UI/viewer or terminal behavior added to core.

## Explicitly deferred / frozen adjacent behavior

Filler generation, rectilinear/monotonic traversal and linking, generic
Fill/Clipper/offset/union algorithms, geometric tolerances, motion/travel,
GCodeProcessor, M73/time estimates, header/footer and exporter are ALL deferred.
No timing correction even though the earliest output symptom is a time header.
No generic solid/sparse grouping, combine-solid, Arachne, aligned-rectilinear
geometry-template identity, painted/side-cut, brim/EFC, spiral, seam occlusion,
zero-area winding, placeholder-double or instance-name sanitizer changes.
No unit tests added/modified; the sole new public seam is the Native pair below.
Existing state/consumers and all approved RED fixtures/SPEC/helpers must freeze.

## Approved read-only discovery / actual first producer witness

Authority root:
`${EVIDENCE_ROOT}/w6-3fb-ironing-modifier-angle-ocd0lki0/`.
DISCOVERY-RR.md, verification.json, verify-discovery.py and full streams/probes.

Existing validation/modifier-wall-loops-classic.3mf: one cube plus existing
modifier (no new geometry). Classic/unpainted; interlocking/brim/skirt/EFC/arc OFF;
shared top ironing/rectilinear, global angle0, spacing0.1, inset0.2. Existing
modifier wall_loops4 is common. Sole GUI field modifier.ironing_angle0/17 differs;
only that XML metadata value changes, all JSON/otherXML/ZIP geometry identical.
Native exports are input-only. PrintConfig.cpp4351-4359 exposes legal GUI angle.

Native2 stable FIRST in BOTH official/seeded, successful real public byte API/CLI,
raw pairs equal. Target0 COMPLETE FAIL/control17 COMPLETE PASS in both modes.
FIRST strict NON-M73 canonical difference byte109,line3,col59 is total-time
header12m11s vs12m12s. It remains in the strict comparator, NOT filtered or fixed.
Diagnostic-only first unequal ordered motion index4884/rawline8840 BOTH:
Native X129.209 Y127.004 vsAres X126.882 Y128.435 (travel liftZ10.4).
Earlier4884motion records match; printZ is10, not10.4. This diagnostic is NOT an
alternate comparator or parity success claim.

Four VALID normally exiting read-only GDB observers preserve COMPLETE streams:
- Native TARGET actual filler receives ONE4-point whole-face region.
- Rust TARGET receives TWO distinct live region records,1half-face area EACH,
  same fill angle/spacing, split atX0; no equal-span area coalescence.
- Native CONTROL receives TWO half-face regions, and Rust CONTROL matches its
  actual bounds/fillangles/spacings EXACTLY plus complete-stream PASS.
- Native local by_extruder/i/j were optimized out: span mechanism is
  SOURCE-INFERRED, not a claimed sampled runtime key/index. Actual filler
  cardinality/bounds above are observed. All invalid/stopped/late-post-move
  samples explicitly excluded; no inferior calls/memory/register writes.

The first divergent producer is pre-filler equivalent-span area collection,
not M73/processor, fixed-angle math or a requested rectilinear correction.
Native double-vs-Rust float key precision is not independently established as
this witness's cause: TARGET angles are IDENTICAL, actual filler angles match.
Source-complete span comparison must still follow the cited IroningParams types.
No implementation/counterfactual GREEN was performed in discovery.

## Frozen Native-first registered RED matrix

Use the already qualified Native exports BYTE-IDENTICALLY, no regeneration:

| Role | tests/parity/ironing-equivalent-span/ | SHA256 |
|---|---|---|
| Equal-angle target0 | equal-angle-target.3mf | `08ff42b871ae8cca22dfb529a059a5e5f2a2a633b73640dac87735b65d9679ab` |
| Distinct-angle control17 | distinct-angle-control.3mf | `e9109b5d0a2e959c154133f7a3d63d94bdc5e9c952b0bdf6fc7c65ff2f7de8cb` |

Two DIRECT unchanged native_stream_case::check ->NativeCase calls with meaningful
labels, source citations and input SHA freezes, assert_two_tools=false (single
extruder). Dynamic live Native expected bytes, NO hardcoded Native G-code,
coordinates, bounds, time, area/call counts or expected-error assertion in tests.
No generated G-code/golden output committed, only input3MFs.

## Sequence / verification / STOP criteria

1. Commit docs-only SPEC, then input/test-only RED; production/helper/golden
   unchanged. All changed text<400LOC, conventional commits/requested coauthor.
2. Exact RED official AND seeded EACH target COMPLETE FAIL/control COMPLETE PASS;
   Native2 stable BEFORE actual public byte API+real CLI. Verify API success,
   CLI0, BOTH COMPLETE ordered Native/API+Native/CLI and actual raw files.
3. Preserve M73, motion/time, comments and metadata/footer in the unchanged
   generator-only comparator. Target panic is BEFORE final raw assertion and
   post-adapter: independently audit raw equality, do NOT claim that assertion ran.
4. fmt and strict workspace/all-target clippy PASS; scope/chronology/input-identity
   proof and REVIEW-REQUEST, then STOP BEFORE GREEN, no FULL/rebase/push/merge.
5. Only after separate authorization may a production-only span/areas correction
   run frozen pair and appropriate ironing neighbors plus fmt/clippy/WASM gates.
   No source/fixture/normalizer expansion to conceal residuals.
6. Existing4chains and W6 integration order unchanged; no automatic fifth-chain
   or main integration credit until its own authorization and exact-tip gates.
