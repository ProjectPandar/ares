# ZAA non-alternating object-fill direction SPEC

Docs only, no RED or production edits. Builds on the modifier slice-Z
chain ending f500b9b3b601f2b8276b452b9e2b226fc890a40e.
Upstream: OrcaSlicer8500fcdccaa10b5099ac20d252af3a7c560046f1.

## Owning upstream boundary

PrintConfig.cpp:4397-4402 defines the region boolean
zaa_dont_alternate_fill_direction, default false. This is a fill-generator
runtime control, distinct from fixed_angle and from the contour ray query.

Fill/Fill.cpp::Layer::make_fills:1235-1242 constructs the generator for each
SurfaceFill, assigns its layer ID, and sets:
`dont_alternate_fill_direction = rcfg.zaa_enabled &&
rcfg.zaa_dont_alternate_fill_direction` using the fill's region_id. The
secondary filler construction at1440-1447 applies the same regional rule
while adjusting layer ID for raft layers. Neither assignment changes the
grouped angle, fixed-angle value, bridge angle or fill grouping identity.

Fill/FillBase.cpp::Fill::_infill_direction:275-323 starts with the stored
angle, selects bridge_angle when nonnegative, otherwise considers layer
alternation only with a defined layer ID and !fixed_angle. Lines311-313
skip _layer_angle when dont_alternate_fill_direction is set. The final
float pi/2 addition and bounding-box reference remain unchanged.

Adjacent source: Fill.cpp::Layer::make_ironing:1507,1631 installs the same
flag for ironing. Fill subclasses may override direction/layer-angle
behavior; this is not permission to rotate all pattern outputs globally.

## Measured public discrepancy, before implementation

Read-only discovery at f500b9b3b, official and seeded,
used the existing admitted classic rectilinear sloped-roof input, with
zaa_enabled=1, minZ0.05 and minimize_perimeter_height35. Only the requested
boolean changes between the false control and true probe. No Native
artifacts were committed and no comparator was changed.

| Probe | Official | Seeded | Public API/CLI |
| --- | --- | --- | --- |
| dont_alternate=0 | PASS | PASS | success, raw bytes equal |
| dont_alternate=1 | DIVERGENT | DIVERGENT | success, raw bytes equal |

Both Native exports accept true and record both enabled booleans in the
complete config footer. Existing generator-only ordered comparison first
differs at byte4306/line155 (M73 placement;117262 vs116476 normalized bytes).
There is also a geometric difference:1529 Native vs1513 Ares positive XY
extrusion moves; the first different extrusion is Internal solid infill,
Native `G1 X113.553 Y105.98 E.03085` versus Ares
`G1 X114.02 Y112.214 E.0933`. Native geometry changes when the boolean is
switched; Ares extrusion geometry stays identical. Thus the witness is
active and is not merely config footer or progress accounting.

The input SHA for true is
74e654dac354007fb5eac279efdbc39e168f8ec8544d870fb628fb4c06fb1218.

## Exact Rust destination and bounded behavior

The runtime destination is the existing object-fill generation boundary:

1. project_slice/fill_entities/generate.rs::generate_layer resolves the
   canonical region belonging to each SurfaceFill::region_id and derives
   the two-option conjunction. Read the actual fill region, including an
   already-resolved modifier region; never substitute a first region or
   a project-wide option. Preserve existing region IDs and fill grouping.
2. fill_entities/monotonic.rs::append and expolygon_polylines carry this
   per-call bit to the current rectilinear geometry entry. Its present
   MonotonicFillParams construction already carries layer_index,
   thickness_layers, fixed_angle and bridge_angle. Add the distinct source
   control to this existing carrier, rather than rewriting fixed_angle.
   fill_entities/types.rs::SurfaceCall and surface.rs may need mechanical
   caller plumbing if that is the smallest shared route; support callers
   retain their upstream default false, without applying object options.
3. fill/rectilinear/surface.rs::infill_direction is the existing port of
   Fill::_infill_direction. Preserve its current float calculation and
   bridge/fixed-angle precedence; inhibit only its layer-alternation term
   under the regional conjunction. Carry the bit consistently through
   rectilinear_multiline.rs where the same source direction calculation
   is repeated, without changing the pattern's own multiline geometry.

Existing Ares fill_entities adapters are temporary compatibility shells
around Layer::make_fills and Fill, not an independently designed pipeline.
This slice fills a missing source-owned runtime parameter in those shells.
Do not add an option guard, fallback, public API, alternate renderer, generic
pattern dispatch framework, or change group_fills comparison/grouping keys.
No ray-query, contour-Z deformation or travel/motion changes belong here.
Use scalar per-call plumbing; no additional per-point allocation or copy.

Included: ordinary object Rectilinear, AlignedRectilinear, Monotonic and
MonotonicLine fills sharing the existing rectilinear direction entry,
including internal-solid/top/bottom paths routed there. Preserve
thickness_layers/layer-ID semantics, existing fixed-angle/template and
bridge overrides. The conjunction must be false when ZAA is disabled.
The complete-output witness remains the admitted classic single-instance
roof; no unmeasured multi-object/modifier coverage is claimed.

DEFERRED: Fill.cpp:1440-1447 bridge-over-infill sparse anchoring, including
the rectilinear patterns in prepare_infill/bridge_over_infill/
sparse_anchoring/linear.rs. Its filler also receives the regional flag
upstream, but this slice retains false at that separate Rust call site.
Anchoring is the next slice and requires its own active public Native RED
before production changes; ordinary-fill parity does not cover it.

DEFERRED: locked_zag and other dedicated object-pattern implementations
whose upstream filler receives the flag at Fill.cpp:1235-1242, including
fill_entities/locked_zag.rs and Line, Honeycomb, lattice, Grid/Triangles/
Stars, TPMS and plane paths. Their explicit false initialization preserves
the existing behavior; this slice does not port their regional flag.
ZigZag/CrossZag already routed through the shared monotonic adapter receive
its scalar, without claiming dedicated-pattern ZAA coverage.

Also deferred: object ironing and
support/raft generator option domains, multi-object/instance and Arachne
ZAA fixtures. In particular Fill.cpp:1631 is a separately identified
ironing follow-up, not silently claimed by the ordinary-fill witness.
infills/rotation.rs is a different existing adapter; this slice must not
rewrite it speculatively when the witnessed path is fill_entities ->
fill/rectilinear. No claim of complete boolean/pattern matrix coverage.
If the bounded path is insufficient for the public RED, STOP and report
before broadening the scope or changing any assertion.

## Public Native RED design: input-only fixture

Add the public E2E RED in its own test-only commit before production GREEN.
Use OrcaRunner::from_env().unwrap() with the repository's
`let Some(runner) = ... else { return; };` convention. Required-oracle
gates set ARES_REQUIRE_ORCA=1 so absence cannot silently pass the gate.

Reuse the existing admitted sloped-roof input geometry and profile setup
from compatible_regions/zaa_contour_z.rs, without executing that older
whole-output assertion as fixture preparation. Construct only input bytes;
do not use an Ares slice result to choose or rewrite the fixture. Add only
the requested ZAA options to the exported 3MF settings; retain unchanged
ZIP-member hashes and verify loaded public options. No Native products or
expected Native G-code are checked into the repository.

The RED contains a true case and false control with ZAA enabled in both.
For each mode, run each input through Native, public validated slicing API
and the CLI. Require Native success, complete config footer and the
requested boolean value; require API and CLI success and raw equality.
Compare whole output with the existing parity::compare_case and
compare_ordered_bytes_generator_only comparators; both cases assert PASS.
The expected RED result is false PASS / true FAIL at whole-output parity,
after all admission/activity assertions have passed. Do not assert an
expected DIVERGENT status or weaken the existing comparator.

Record a repeatable activity witness: true Native Internal solid infill
extrusion stream differs from false, and its direction/coordinates are
nontrivial; retain input/native/API/CLI SHA and input.3mf plus outputs in
external artifacts. Repeat true/false with fresh Native caches to establish
stable normalized Native geometry before accepting the RED. Raw Native
header timestamps need not match; use only the existing normalization.
Keep public fixture construction separate from comparison so GREEN can
satisfy unchanged assertions. Unexpected Native rejection, inactivity or
non-repeatability is STOP, not a new numeric or admission policy.

## GREEN gates

Port only the runtime bit after accepted RED. Re-run true/false
both official and seeded on the exact hash; preserve the false control's
raw Ares bytes. Add an input-only disabled-ZAA/flag-true control before
production if needed to pin the two-option conjunction. Existing ZAA
modifier and sloped-roof directed controls must stay green; bridge and
fixed-angle controls must retain their source semantics.

All Native runs use a fresh Native cache/artifact root per run. Gates: fmt, strict clippy for core
and the CLI test target, wasm32 check; every file <=400 lines. No guards,
regions[0], debug macros, suppressions, ignored tests, comparator changes,
assertion weakening or unsolicited production edit.
