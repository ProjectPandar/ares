# PrintObject::combine_infill: scaled-width clearance arithmetic

Latest status: COORD accepted #49 and coordinated W4, authorizing a separate
SPEC revision then caller-only GREEN3 as specified in the final amendment.
Candidates c810ef7d3 and 81318b506 failed strict exits and are not independently
integrable. Earlier review-only stage descriptions below remain historical;
the final amendment records the current bounded authority, not GREEN acceptance.
Original SPEC on `worker3/combine-solid-clearance-spec`, based on
merged main `231bd11ccae4ac26304aa89772a1e5c4b2fd532c`.
COORD accepted the six-case Native discovery and source boundary and authorized
this SPEC followed by a separate RED. GREEN requires independent RED review;
no implementation or FULL is authorized by this document. Integration is
COORD-owned, queued after W2; no worker rebase/merge/push/window is implied.

## Owning upstream rewrite slice

Pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/PrintObject.cpp:4175-4180`,
  `PrintObject::combine_infill`: at100% use `stInternalSolid` and configured
  `internal_solid_infill_pattern` rather than sparse internal surfaces.
- `4252-4264`: `float clearance_offset` adds `0.5f` times the retained
  perimeter Flow's `scaled_width()` and pattern-dependent `1.5f`/`0.5f`
  times the retained solid-infill Flow's `scaled_width()`. The existing
  rectilinear/monotonic/grid/lateral-lattice/line/honeycomb/lateral-honeycomb
  predicate chooses1.5f; other configured patterns choose0.5f.
- `src/libslic3r/Flow.hpp:61-62`: width is float;
  `scaled_width()` returns `coord_t(scale_(m_width))`, truncating the scaled
  width **before** multiplication. `libslic3r.h:93` scales by division.
  Ordinary C++ arithmetic converts these coord_t operands to float for the
  float multiplications and addition; do not replace that order with f64 math.
- `PrintObject.cpp:4265-4291`: offset/intersection subtraction and combined
  surface transport consume this clearance. They are adjacent, deferred
  behavior, not permission to port kernels or rewrite surface algorithms.

Rust destination is exclusively the clearance calculation in
`crates/ares-core/src/project_slice/prepare_infill/combine_infill/process.rs`
`combine_group`, currently205-208. It presently sums raw retained f32 widths
in f64 millimeters, divides the combined value by `scale.factor()`, then
casts the final delta to f32. Replace only this arithmetic with the source
order: existing `Flow::scaled_width(scale)` integer truncation, conversion
to f32, f32 multiplication and addition. The existing helper in
`project_slice/perimeters/types.rs:17-18` already implements the truncation;
do not change Flow construction, transport or that helper.

The existing `combine_group` record/geometry machinery is the compatibility
shell around this upstream operation, not an Ares-owned infill design.
This slice replaces its mismatched arithmetic, not the surrounding pipeline.

## First inconsistent input, observed before offset

Single-region authored75-layer overhang, same geometry/placement/profile,
layer0.2 and nozzle0.4. Existing canonical binaries/DWARF and read-only GDB,
no Native source instrumentation or rebuild:

| Configured solid pattern | Native actual delta | Rust actual delta |
|---|---|---|
| monotonic,1.5f branch |899998.0|900000.0|
| concentric,0.5f branch |449999.0|450000.0|

For each pattern the first8 pre-offset intersection contours and holes match
exactly, including coordinate order. Rust local tops are2,4,...16/count2;
Native's matching calls use Miter and both miter limits are3.0. Thus the first
observed different offset input is the clearance scalar, not its geometry.
Successful diagnostic Native complete streams match canonical references
under generator/date identity normalization; CLI diagnostic streams are
raw-identical to canonical CLI. Rectilinear shares the cited1.5f source
branch; its strict witness is independently recorded, not a claimed GDB run.

Ares retained widths are observed as0.44999998807907104. Source integer
scaling truncates that value to449999; applying the two float factors gives
899998 or449999, agreeing with the actual Native arguments. The current
late-rounding calculation yields900000 or450000. This source consistency
calculation diagnoses the mismatch, not a replacement acceptance oracle.
Direct Native-width probes without captures are not claimed as width
provenance. Failed pointer-typedef decoders are preserved/excluded; the
accepted reader strips typedefs and verifies Point16-byte layout.

## Frozen Native matrix and public seam

Reuse `tests/parity/combine-infill-region-view/combined-single.3mf` directly
for sparse15-on: its discovery SHA is identical, not a new baseline ZIP.
Freeze only the five other existing discovery inputs, without regenerating
geometry or changing non-settings ZIP members. Only GUI density, combination,
internal-solid pattern and maximum combination height change.

| Case | Density / combination / solid pattern / maximum | Both modes |
|---|---|---|
|sparse15-on|15% /1 /monotonic /100%|PASS control|
|solid100-off|100% /0 /monotonic /100%|PASS control|
|solid100-on|100% /1 /monotonic /100%|DIVERGENT target|
|solid100-rect|100% /1 /rectilinear /100%|DIVERGENT sibling|
|solid100-concentric|100% /1 /concentric /100%|DIVERGENT sibling|
|solid100-one-layer|100% /1 /monotonic /0.2|PASS control|

Discovery has24 fresh Native streams and12 CLI outputs, two Native repeats
per input/mode, stable repeat/cross-mode identities. Strict first differences:

| Target | Byte / line | Native | CLI | Lengths |
|---|---|---|---|---|
|monotonic|3555 /158|M73 P5 R15|G1 X113.987 Y111.832 E.01673|199325/199271|
|rectilinear|3584 /159|M73 P5 R15|G1 X108.168 Y106.013 E.2555|199729/199675|
|concentric|11014 /487|G1 F2154|G1 F2160|165913/165913|

M73/feedrate are downstream witnesses, not repair sites. First changed
monotonic/rectilinear motion is physical51/Internal solid; concentric's first
changed motion is physical5/Inner wall feedrate. Do not filter these effects.

The confirmed test seam is fresh canonical Native versus public byte-in/
options-to-byte-output API and CLI, not private numerical unit tests. RED must
reuse main `NativeCase` or the existing W3 `case_runner`/`complete` lifecycle;
no copied runner, second normalizer or assertion policy. Reusing W3 preserves
both fresh repeats and their producer records before complete assertions.
Admission asserts one model part/no modifier and GUI values; effective
Native footer and positive solid/sparse activity demonstrate active inputs.
No previous assertion, case/input or helper behavior may be weakened.

## Non-negotiable decisions and stage exits

1. SPEC commit is documentation only, parented on231bd11cc. Per COORD's
   discovery approval, RED may follow as a separate commit; no production
   edit before its independent review.
2. RED freezes the five existing target/control ZIPs and reuses the identical
   sparse control. Register six complete-output cases through shared lifecycle.
   Require3 intended strict FAIL/3 PASS per official/seeded mode, all prior44
   selected cases PASS, repeat stability/API-CLI raw equality and retained
   existing Native/API/CLI hashes/assertions. Preserve command/status/binary/
   input/output receipt links and repeatable artifacts; report RED to COORD.
3. Only after RED approval, GREEN changes **process.rs clearance arithmetic
   only**, using truncated Flow scaled_widths and float operations in upstream
   order. Freeze SPEC/RED/helpers/inputs. Keep the current pattern predicate,
   per-region views, actual layer-id0 skip, height cap/count/area thresholds,
   intersection and surface rewrite/transport, Flow values/keys unchanged.
4. Any strict residual means **STOP/report**. Explicitly defer offset kernels,
   batching/per-component union/miter/topology, phase/graph/bbox, bridge angles,
   max-height arithmetic, Flow grouping fields, E/wipe/retraction/feedrate/
   cooling/time/progress. No adjacent port or unsupported-feature guard.
5. Gates: `cargo fmt --all --check`; locked workspace/all-target Clippy with
   `-D warnings`; locked core+wasm browser wasm32 check. CPU12 selected work,
   `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0`; no FULL until COORD releases the
   after-W2 integration window. Later acceptance needs then-current main,
   preserved per-stage range-diff and exact clean final-hash fresh official+
   seeded FULL with zero FAIL/skipped; refresh if main advances.

Core remains platform-neutral for browser WASM, Windows, macOS and Linux.
No new API/configuration, filesystem/terminal/UI/OpenGL behavior, fallback,
feature flag, new allow/expect/ignore, tolerance, inverted assertion, Native
G-code golden or output filtering. Only existing generator/date identity
normalization; all M73/time/feedrate/E/wipe/retraction/footer bytes significant.

Accepted discovery and producer/localization audits:
`${EVIDENCE_ROOT}/w3-combine-solid-readonly-231bd11cc/README.md`.
The prior minimum-area7/7 PASS and combine-height6/6 PASS matrices remain
bounded negative evidence; they do not authorize any adjacent change.

## Read-only residual amendment: physical height is not extrusion height

COORD approved RED d09e00530 and the one-expression clearance port only.
Candidate c810ef7d3 changes process.rs3+/4−, with all tests/inputs/helpers
frozen. fmt/all-target Clippy/browser wasm32 checks pass, but official selected
runtime remains47PASS/3strictFAIL (185.692s), with the same first differences
as RED. Monotonic/rectilinear raw outputs are unchanged; concentric changes
without closing its first feedrate difference. The strict stop fired before
seeded; no FULL or broader implementation followed. C810 is **not GREEN**.

COORD then authorized one read-only diagnosis and this documentation amendment.
Existing candidate/official binaries only; no production/test/input edit,
Native rebuild/source instrumentation, seeded execution or FULL.

### Clearance and the complete combine boundary are now excluded

For both monotonic and concentric, actual candidate deltas equal Native
899998/449999 at the original offset call. Not merely geometry-equivalent:
all36 offset calls' input/output vertices, holes and ordering match exactly.
All108 subsequent difference/intersection inputs/outputs also match exactly,
including safety=No. All75 layers' pre/post-combine surfaces match in geometry,
ordering, kind, thickness, thickness_layers, bridge_angle and extra_perimeters.
Rust's final snapshot is before group74, whose selected-kind upper surface
is empty: the cited intersection-empty guard makes that final group a no-op.
For monotonic, all38 later `split_solid_surface`/Rust narrow-split inputs and
normal/narrow outputs also match exactly. These are bounded negative findings,
not permission to modify kernels or a universal parity claim.

Every accepted diagnostic full stream equals its respective canonical output
(Native generator/date identity only; candidate CLI raw-identical). Official
symbol-table/ABI reads are validated by these exact snapshots. Existing DWARF
was read statically for layout only, **without running a seeded inferior**.
Optimized-out/inline-frame/source-line reader failures are retained and excluded
from accepted evidence; no missing observation is presented as equivalence.

### Next observed different generator input

At the first sampled Arachne construction in each fixture:

| Handoff | Native actual layer_height | Candidate actual layer_height |
|---|---|---|
|monotonic routed to ConcentricInternal|0.20000000000000007|0.39999999999999997|
|configured concentric|0.20000000000000007|0.39999999999999997|

The same sampled calls have **identical raw outline vertices/order**, outer/
inner spacing364159, inset count (6 internal,23 configured), and wall inset0.
Sixteen calls per path are retained, not just one scalar print. This is a
separate downstream input mismatch, not another clearance delta, nozzle fork,
progress/E/feedrate repair or proof that changing it alone closes the RED.
Other unmeasured Arachne policy inputs are not declared universally equivalent.

Owning upstream source, pinned8500fcd:

- `Fill/Fill.cpp:1290,1485`: `FillParams::layer_height` is assigned
  `layerm.layer()->height`, the **physical layer** height, independently of
  the surface's combined extrusion Flow height.
- `Fill/FillConcentricInternal.cpp:37` and `Fill/FillConcentric.cpp:100`:
  pass `params.layer_height` to the `Arachne::WallToolPaths` constructor.
- `Arachne/WallToolPaths.cpp:69-75`: retain that layer_height;
  `516-517` use it when deriving extrusion width from line spacing. It is
  an active geometry input, not just an emitted HEIGHT comment.

Current Rust mismatch:

- `project_slice/fill_entities/concentric.rs` internal and configured-full-
  density entrypoints scale `fill.params.flow.height`/`params.flow.height`
  into the generator's layer_height argument (currently48 and170).
- `concentric/generate.rs:94` converts that integer back to millimeters for
  `RawWallToolPathConfig.layer_height`; this passes accumulated0.4 extrusion
  height where upstream passes physical0.2. It also unnecessarily quantizes
  that distinct physical-height input if reused for ordinary layers.
- `arachne/wall_toolpaths.rs:68-71` consumes the supplied height for the
  width-from-spacing calculation, matching the cited downstream source owner.

### Proposed next slice — review only, no implementation authorization

Restore only the upstream **FillParams physical-layer-height handoff** through
existing `fill_entities/generate.rs` (already owns `PlannedLayer.height`), both
concentric entrypoints and their shared generator into RawWallToolPathConfig.
These existing Ares adapters are compatibility shells around the cited Fill
and WallToolPaths input boundary; do not invent a new pipeline or parameter
family. Keep physical layer height separate from retained extrusion Flow/
surface thickness: do not change Flow.height, combined thickness/count,
spacing/width/volume, kernel/beading math, phase/graph or emitted HEIGHT/E/time.

This is not permission to extend c810 or quietly replace its failed scope.
COORD must independently approve the revised source boundary and RED strategy
before any further production edit. Approved six GUI cases and shared strict
lifecycle remain frozen; new controls or RED changes require separate approval.
No claim of dual-mode GREEN or final acceptance is made; seeded/FULL remain
unrun. Any later bounded attempt with a strict residual must again STOP/report.

Residual diagnosis and exact snapshots:
`${EVIDENCE_ROOT}/w3-combine-solid-residual-c810ef7d3-readonly/README.md`.

## Approved GREEN3 amendment: configured Arachne nearest-point ties

COORD accepted the #49 read-only conclusion, independently verified Point.cpp
last-tie behavior and the existing pub(crate) helper, and coordinated W4.
Authorization: commit this SPEC revision separately, then change only the
standard Arachne finalize caller to reuse that helper and delete orphan
nearest_to. Helper/tests/inputs/ironing/union remain frozen. This is not GREEN
acceptance or a FULL/rebase/integration window.

### Stage result and read-only correction

Authorized GREEN2 transports physical height through the three approved adapters
and includes exactly ten approved fixture-argument adaptations. fmt/all-target
Clippy/browser wasm32 pass. Official selected50 is49PASS/1FAIL (138.158s): both
monotonic/rectilinear tests close strictly; concentric remains byte11014/line487,
Native `G1 F2154`, Ares `G1 F2160`. Seeded, post-change neighbors/core units and
FULL were not started after that failing bounded batch. Candidate is NOT GREEN.

The feedrate is a **downstream cooling witness**, not proof that paths agree.
At physical layer5/Z1 the affected inner wall has identical captured width
0.44999998807907104, height0.20000000298023224, geometric mm3/mm
0.08141592890024185, print ratio1, filament ratio0.98 and max volumetric speed12.
The resulting cap150.39929071112397mm/s exceeds the actual40mm/s role speed.
Native writer and both pre-cooling blocks have F2400. It is neither a bridge
role nor a different overhang/Flow/volume speed cap at that wall.

Both cooling inputs enable cooling, target8s/min10mm/s, allow outer slowdown,
and have equal initial position. The first36 parsed cooling records have equal
numeric fields. Record36 (zero-based) is an already different subsequent
concentric travel: Native `G1 X111.492 Y111.316 F9000`, Ares
`G1 X111.316 Y111.492 F9000`. Subsequent rotated/cropped runs have different
length/time values. Thus the same layer's wall is slowed to35.9012451171875 vs
36.006507873535156mm/s, producing F2154/F2160. Do not repair cooling or feedrate
formatting to compensate for these different route inputs.

### Earliest localized differing decision and source ownership

Pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `Fill/FillConcentric.cpp:73-146`: configured solid **ThickPolylines/Arachne**
  overload of `_fill_surface_single`, not its ordinary Polylines overload.
  At113/120/122, `last_pos` is passed to `Point::nearest_point_index` before
  clipping130 and shortest-traverse reorder139.
- `Point.cpp:190-196` forwards Points to the pointer scan199-220. The X/Y
  pruning tests208/213 skip only **strictly greater** distance; assignment215
  therefore retains the **last** equal nonzero minimum. The near-zero early
  exit218 remains part of upstream behavior.
- Rust `project_slice/fill_entities/concentric/finalize.rs:16` calls private
  `nearest_to`44-50; `.min_by_key`48 retains the **first** equal minimum.
  This adapter is a temporary compatibility shell around the cited upstream
  start-selection operation, not an Ares-owned routing design.

Read-only Native GDB returns index7 for captured reference(3194483,3194483)
and an ordered point list also present exactly in the captured Rust generator
input. Its indices6=(2846770,3056100) and7=(3056100,2846770) both have exact
nonzero squared distance140054185058. Native returns7; the frozen Rust selector
returns6. Closed points AND endpoint widths are true, excluding the extra
closedness condition for this witness. Captured pre-cooling travels actually
contain Native X113.056/Y112.847 versus Ares X112.847/Y113.056. The Rust index is
identified from captured inputs plus the unchanged selector and confirmed by
actual route text; it is not presented as a captured Rust return register.

Five accepted diagnostic complete streams equal their respective canonical
streams (Native generator/date identity only, CLI raw exact). Failed/partial
readers are retained and excluded. No Native build, source instrumentation,
inferior function call or seeded runtime occurred. Static seeded DWARF type
reads do not execute a seeded inferior. No universal toolpath parity or proof
that changing this one choice closes the entire RED is claimed.

### W4 overlap and coordinated caller-only boundary

W4's Step0 owns ordinary Polylines/ironing `FillConcentric.cpp:9-71`, including
`union_pt_chained_outside_in`43 and nearest-start49. W3's measured call is the
separate ThickPolylines overload120 after Arachne. W3 has **not** measured or
modified W4's union/order boundary, or FillConcentricInternal in this diagnosis.

There IS shared upstream semantic ownership: both call `Point::nearest_point_index`.
Moreover existing Rust `fill/concentric_sparse.rs:23-44::nearest_point_index`
already ports last-equal selection, used by the plain-ring path127 and ironing
through `fill_entities/ironing.rs:136`. The configured Arachne adapter instead
has the duplicate incompatible private selector. This semantic overlap has been coordinated with W4 by COORD. Do not alter
the shared helper or W4's worktree; if W4 later changes the same file, handle
any rebase conflict explicitly rather than overwriting its work.

The authorized Rust destination is exclusively
`fill_entities/concentric/finalize.rs`: reuse the existing source-ported
`fill::concentric_sparse::nearest_point_index` at the standard Arachne call and
remove the now-orphan private `nearest_to`, rather than introduce another helper.
Leave that shared helper itself, the internal-origin selector, distance helper,
ring union, Arachne/beading, closedness gate, clipping, shortest-traverse order,
Flow/physical height, extrusion, speeds/cooling/timing/M73 unchanged. This approved source-owner boundary permits only the named caller change;
no kernel, helper or adjacent repair is included.

GREEN3 must preserve the existing six GUI cases and
strict helpers/inputs, all prior44 controls, the ten authorized mechanical unit
adaptations, and complete-byte assertions. Run the frozen six cases plus prior44 controls on the exact clean tip in
fresh official+seeded selected checks, plus concentric/ironing/Arachne
neighbors, with unchanged
producer identities and the three required gates. Any remaining strict residual
again means STOP/report. No tolerance, filter, golden, inverted assertion,
allow/ignore, fallback, extra config, pipeline feature, FULL or integration is
authorized. FULL/rebase/push remain COORD-owned window decisions.

Read-only localization, producer/canonical checks and W4 boundary:
`${EVIDENCE_ROOT}/w3-concentric-speed-residual-81318b506-readonly/README.md`.
