# Task 22O.45 — CrossHatch fill surface

## Status

Implemented and verified. This source-owned dependency-first crate-private
CrossHatch fill transaction remains intentionally unwired: public prepared
project slicing still consumes and disposes O43, returns
`ProjectSlicingIncomplete`, and emits no O45-derived G-code. Final independent
source/specification and standards reviews unconditionally approve this
completed documentation and gate state.

## Goal and upstream boundary

Port the KSR-reached public CrossHatch fill transaction from OrcaSlicer 2.4.2
commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/Fill/FillBase.cpp:105-119`, for the public
  `Fill::fill_surface` half-spacing offset, ordered offset-component traversal,
  and accumulated owned result;
- `FillCrossHatch.hpp:12-25` and complete
  `FillCrossHatch.cpp:28-232`, for the CrossHatch pattern and protected
  per-component implementation;
- `FillBase.hpp:48-61,100,112-124,173-174,182-202,220,224`, for the source
  state, default-zero overlap, and relevant `FillParams` fields;
- `FillBase.cpp:1820-1823,1827-1829`, for the KSR-active predicate and
  `chain_or_connect_infill` branch into the already-ported O44 connector; and
- `FillBase.cpp:2712-2715`, for KSR multiline one's exact no-op.

The Rust destination is a crate-private `fill::cross_hatch` module. It borrows
one unoffset ExPolygon surface and returns final inset, generated, clipped,
short-remnant-filtered, boundary-connected, and rotation-restored polylines or
the first checked `ClipperError`.

```rust
pub(crate) struct CrossHatchFillParams {
    pub(crate) z: f64,
    pub(crate) spacing: f64,
    pub(crate) overlap: f64,
    pub(crate) angle: f32,
    pub(crate) density: f32,
    pub(crate) multiline: i32,
    pub(crate) anchor_length: f32,
    pub(crate) anchor_length_max: f32,
    pub(crate) dont_sort: bool,
}

pub(crate) fn fill_surface(
    surface: &ExPolygon,
    params: CrossHatchFillParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError>;
```

The record deliberately combines only the `Fill` and `FillParams` fields read
by this source path. It does not accept degrees, percentages, option records,
flows, layer IDs, `thickness_layers`, or `_infill_direction`. The protected
CrossHatch override does not read its thickness or direction arguments. A
later source-cited caller will project these already-resolved values before
calling this seam.

This is the smallest public source-owned deep boundary. Exposing only
`_fill_surface_single` would leak an already-offset component and omit active
base-class behavior. Exposing a raw CrossHatch lattice would leak a private
one-consumer intermediate and omit clipping, connection, and rotation.
Activating `PrintObject.cpp:2725-2761` now would instead require complete
`Fill.cpp::group_fills`, angle/template projection, nominal sparse-flow
resolution, adaptive/Lightning decisions, and lower-layer-map ownership.

## Direct dependencies

- `ClipperUtils.hpp:19,27,345` and `ClipperUtils.cpp:568-570` for singleton
  ExPolygon offset with the default Miter join and miter limit 3;
- `ClipperUtils.cpp:207-223,837-845,926-927` for open-polyline subject /
  closed-polygon clip intersection using NonZero fill and recursive open
  PolyTree extraction order;
- `Point.hpp:187-203,216-218,707-720`, `MultiPoint.hpp:27-33`,
  `MultiPoint.cpp:21-34,48-55,89-92`, `ExPolygon.hpp:300-306`,
  `ExPolygon.cpp:29-40`, `Line.hpp:152-164`, `BoundingBox.hpp:15-40,221-223`,
  and `BoundingBox.cpp:69-81,172-179` for
  round-away-from-zero point construction, translation, rotation, polyline
  length, bbox merge, and negative-coordinate grid alignment; and
- `libslic3r.h:42-46,52,60-70,92-96` for `coord_t`, `coordf_t`, `EPSILON`,
  scale factors, continuous `scale_`, and `SCALED_EPSILON`.

O45 reuses O44's crate-private `fill::connect::connect_infill` exactly. It
constructs `FillConnectionParams` internally so the later CrossHatch caller
does not depend on the connector's implementation seam.

The only new shared geometry operation is the source-shaped open intersection
already supported by Ares' private Clipper worker:

```rust
pub(crate) fn intersection_open_polylines(
    subject: &[Polyline],
    clip: &[Polygon],
) -> Result<Vec<Polyline>, ClipperError>;
```

It must add open subjects, then closed clips, then execute Intersection with
NonZero subject and clip rules and return open paths without the closed-path
recombination used by Ares' current `intersection_pl`. Rotation, translation,
contour bbox/grid alignment, contour-plus-hole flattening, and polyline length
remain local to CrossHatch instead of widening generic geometry APIs for one
consumer.

## Trusted contract

The crate-private seam trusts the upstream caller invariants: a valid nonempty
ExPolygon contour, finite positive spacing and density, finite z/overlap/angle,
positive representable line spacing and period, representable counts and
fixed coordinates, `multiline == 1`, and
`anchor_length_max >= 0.05f` with O44's anchor invariants. Do not add
additional validation, fallback, clamping, or defensive copies; the
source-required low-density repeat-ratio clamp remains included. The
implementation may use only narrow branch-selection debug assertions for
multiline one and the active anchor branch; it must not mirror the whole
trusted invariant list as defensive checks.

Checked fixed-coordinate construction, addition, rotation, and Clipper work
remain geometry boundaries. The first range or Clipper failure returns
`ClipperError::CoordinateOutOfRange` and publishes no partial output. The
borrowed source surface remains unchanged.

## Required behavior

For trusted input, Ares must preserve this exact order and arithmetic:

1. compute the public-wrapper offset delta as
   `f32((overlap - 0.5 * spacing) / scale.factor())`: complete the subtraction,
   multiplication, and division in f64, then cast once to f32; call singleton
   `offset_expolygon` with Miter/3 and preserve returned ExPolygon order;
2. if the singleton offset returns no components, return `Ok(Vec::new())`.
   Otherwise process offset components serially. Each component contributes
   paths to the final vector in component order. An error in a later component
   discards all earlier working output;
3. for each owned component, evaluate the source comparison exactly as
   `f64::from(angle.abs()) >= 1e-4_f64`; do not narrow EPSILON to f32. When it
   succeeds, negate the signed f32 angle and only then widen it to f64, rotate
   contour then holes using separate f64 cosine and sine evaluations, and round
   each completed coordinate halves away from zero;
4. derive the bbox from the rotated outer contour only. Compute
   `density_adjusted` by f32 `density / multiline` before widening it to f64;
   compute `line_spacing` by truncating
   `(spacing / factor) / density_adjusted` to i64;
5. when the f32 density widened to f64 is strictly below `0.999`, convert that
   already-truncated i64 spacing to f64, multiply by `1.08`, and truncate again
   to i64. Do not combine the two truncations or round either result;
6. multiply final line spacing by four in integer space to obtain the distinct
   `alignment_cell`. Align the bbox minimum downward on that cell: positive
   coordinates use truncating integer division;
   negative coordinates use
   `((coord - alignment_cell + 1) / alignment_cell) * alignment_cell`.
   Merge only that aligned point into the bbox, leaving the original maximum;
   then compute width and height as i64 `max - min` and widen those completed
   differences to f64 for pattern generation;
7. set repeat ratio to f64 one unless widened density is strictly below `0.3`.
   In the low-density branch evaluate `-5 * density` in f32, evaluate the f32
   exponential, widen it, then compute and clamp `1.0 - expf` in f64 to
   `[0.2, 1.0]`;
8. widen final i64 line spacing to the f64 `pattern_grid`, distinct from the
   four-times-larger alignment cell. Compute `z_scaled = z / scale.factor()`
   in f64, then generate the Z phase using only f64 scaled-coordinate values:
   `trans = pattern_grid * 0.4`,
   `repeat = pattern_grid * repeat_ratio`, add
   `repeat / 2 + trans` to `z_scaled`, set `period = trans + repeat`, and
   calculate the nonnegative remainder with
   `z_scaled - floor(z_scaled / period) * period`. For negative shifted z, the
   quotient must still use floor rather than truncation or fmod. Set
   `trans_z = remainder - repeat`;
9. narrow `fmod(z_scaled, period * 2) - (period - 1)` to source-width i32 by truncation
   toward zero before selecting direction `-1` for `phase <= 0`, otherwise
   `+1`. A raw positive fractional phase that narrows to zero selects `-1`;
10. choose repeat only for `trans_z < 0`. At exact zero choose transform.
    Transform progress uses `fmod(trans_z, trans) / trans`; values strictly
    below `0.5` use `(progress + 0.1) * 2` with the current direction, while
    equality and above use `(1.1 - progress) * 2` with the opposite direction;
11. set transform grid `G = pattern_grid * 2` before every transform
    calculation. For effective progress `p`, set
    `offset = p * (1.0 / 8.0) * G` and generate one-cycle points in exact order
    `(.25G-offset,+offset)`, `(.25G+offset,+offset)`,
    `(.75G-offset,-offset)`, `(.75G+offset,-offset)`, then construct fixed
    points with halves-away rounding. Use `G` for cycle count
    `trunc_i32(width / G + 2)`, row count `trunc_i32(height / G + 2)`, cycle
    translations `(i*G,0)`, odd-row translations `(0,i*G)`, and even-row
    translations `(-.5G,(i+.5)G)`. Copy and translate the fixed four-point
    cycle in increasing cycle index and concatenate those copies into one base
    row polyline. Emit copies of that full base row for odd rows in increasing
    row index, then emit even-row copies in increasing row index. Direction
    zero remains on the nonnegative horizontal branch, while negative
    direction swaps width and height before counts and swaps every final x/y
    pair afterward;
12. repeat layers use `trunc_i32(height / pattern_grid + 1)` rows and emit
    two-point paths `(0, pattern_grid*i)` to `(width, pattern_grid*i)` in
    increasing index order. Apply the same negative-direction width/height and
    final coordinate swaps. Preserve every path and point order; do not sort
    or canonicalize;
13. translate generated paths by the merged bbox minimum with checked integer
    addition. KSR `multiline == 1` then takes the exact early return at
    `FillBase.cpp:2714-2715`; no offset or path rewrite occurs;
14. flatten the rotated ExPolygon as contour followed by holes, preserving
    winding and order. Intersect generated open polylines as subjects against
    those closed polygons as clips, preserving returned open-path and point
    order without recombination;
15. compute each clipped polyline length as the sum of source `Line::length`
    values, with fixed-coordinate endpoint subtraction before f64 norm. Stable-
    erase only paths whose length is strictly less than continuous f64
    `(0.8 * spacing) / factor`; exact equality survives;
16. if clipping or strict filtering leaves no paths for a component, skip O44
    and rotate-back for that component and continue with the next component.
    If paths remain, call O44 directly with the rotated component boundary,
    original unscaled f64 spacing, f32 anchor and maximum, source i32 multiline,
    bool `dont_sort`, and explicit scale. The selected O45 contract never calls
    `chain_polylines` or the legacy Ares anchor helper;
17. rotate only that component's connected output back by the positive f32
    angle under the same `abs(angle) >= EPSILON` rule, then append it to the
    public result. Earlier components must never be rotated twice; and
18. return the exact owned vector. No fallback, sorting, path canonicalization,
    swallowed error, filesystem read, platform branch, or lifecycle mutation is
    permitted.

The KSR density and spacing witness is especially sensitive to operation
order. With spacing f32 bits `0x3ed06cbe` promoted to
`0.40707963705062866`, density bits `0x3e19999a`, and multiline one, Normal
line spacing is `2_713_864` before and `2_930_973` after the multiplier;
LargeBed is `271_386` then `293_096`. Applying `1.08` before the first
truncation would produce the wrong LargeBed value `293_097`.

A small public-seam geometry vector must make the raw pattern order observable:
with effective `pattern_grid=100`, width 250, height 450, and transform progress
0.2, nonnegative direction contributes eight 12-point paths while negative
direction contributes six 16-point paths; the corresponding repeat geometry
contains five horizontal paths or three vertical paths. Exact public
`fill_surface` output must freeze the resulting point/path order rather than
testing a private generator helper directly.

The selected public contract fixes multiline to one, so widening before or
after division by one is observationally identical. The source f32 operation
remains normative, but O45 does not invent a private out-of-contract
multiline-nine helper solely to distinguish an unobservable implementation
choice.

## KSR path

The committed fixture reaches CrossHatch anchor generation on lower planned
layer indices
`[14,29,30,31,40,44,59,64,69,74,81,84,89,104,115,124,135,254]`.
Its direct O45 tuple is:

- Normal scale;
- nominal sparse-flow spacing `0.40707963705062866` mm, promoted from f32 bits
  `0x3ed06cbe`;
- density `0.15000000596046448`, f32 bits `0x3e19999a`;
- f32 angle `0.7853981852531433`, bits `0x3f490fdb`;
- f32 anchor `1.6283185482025146`, bits `0x3fd06cbe`;
- f32 anchor maximum 20, bits `0x41a00000`;
- multiline one, default `dont_sort=false`, and `overlap=0`.

The public oracle uses bit-exact accumulated planned `print_z` rather than
nominal decimal heights: lower 44 is `0x4022000000000000`, lower 31 is
`0x401999999999999d`, lower 40 is `0x4020666666666668`, and lower 14 is
`0x4008000000000001`.

The inherited default overlap is zero in the anchor caller; the archive's
15% `infill_wall_overlap` is not assigned to this `Fill` field. Therefore the
KSR public-wrapper offset delta is f32 `-203_539.8125` at Normal scale, bits
`0xc846c4f4`, and `-20_353.982421875` at LargeBed, bits `0xc69f03f7`.

At density 15%, `-5 * density` is exactly f32 `-0.75`, f32 `exp` is
`0.47236654162406921` (bits `0x3ef1da07`), and the widened repeat ratio is
`0.52763345837593079` (f64 bits `0x3fe0e25f90000000`). For Normal scale,
the transform size is `1_172_389.2`, repeat size
`1_546_479.420396477`, and period `2_718_868.620396477`.

O45 direct tests use synthetic ExPolygons and representative KSR layer z
values. They do not claim that the current prepared-project graph already
projects nominal sparse flow, `group_fills`, or the lower-layer map.

## Deferred behavior and scaffolds

Explicitly deferred from O45 are:

- `FillBase.cpp:1824-1826` and
  `ShortestPath.cpp:1968-1996::chain_polylines` for
  `anchor_length_max < 0.05f`;
- `FillBase.cpp:2717-2782` for multiline 2 through 10 and its Clipper2 round
  open-path offset. Ares' Clipper6 offset is not a fallback;
- generic `_infill_direction` at `FillBase.cpp:275-320`, because CrossHatch
  ignores the value passed to its override;
- `Fill.cpp:25-1504`, including rotation-template resolution,
  nominal sparse-flow projection, `group_fills`, adaptive/Lightning generator
  wiring, and the public anchor-generation caller;
- `PrintObject.cpp:2725-2761`, its transaction-local lower-layer anchor map,
  and all later bridge depth, direction, commit, fill, extrusion, motion,
  G-code, CLI, and UI behavior; and
- every other Fill pattern and any general Rust Fill trait/class hierarchy.

The old `infills.rs`, `infills/rotation.rs`, `InfillPath`, and legacy
`InfillOptions` remain temporary compatibility scaffolds and are not called or
modified. They use simplified scanlines, host sorting, two-point output,
incorrect CrossHatch alternation, and a collapsed anchor/max value; none can
substitute for this source boundary. O45 adds no `PreparedPost...` wrapper and
does not compute then discard paths.

Once O45 calls O44, remove O44's temporary module-level dead-code expectation.
Because O45 itself remains unwired, place at most one narrow reasoned
`#[cfg_attr(not(test), expect(dead_code, reason = "..."))]` on the CrossHatch
source module rather than fabricating a production caller.

## Acceptance

Before implementation, a disposable pinned-Orca harness must call the public
inherited `FillCrossHatch::fill_surface`, not protected helpers. It must freeze
exact ordered output for Normal and LargeBed, use the KSR parameter tuple,
cover repeat, forward transform, backward transform, both directions, a
negative bbox, a hole, and more than one inset component, and prove its O44
endpoint/arc keys have no comparator-equivalent records. Debug and Release
must agree and repeated runs must be byte-identical. The harness source,
commands, normalization, hashes, and restored pinned tree belong in the
matching plan.

The first compiling Rust stub must produce a genuine nonempty exact-output RED.
Focused literal tests must then distinguish:

- public f64-completed/f32-cast half-spacing offset and inset-component order;
- KSR line spacing, two-stage truncation, and both density thresholds using
  adjacent f32 bit patterns;
- f32 `exp`, low-density clamp, negative grid floor, and both scales;
- fractional phase-to-i32 truncation, negative-z floor remainder,
  `trans_z == 0`, `progress == 0.5`, and transform/repeat direction swaps;
- halves-away point construction, angle threshold neighbors, checked
  translation/rotation, hole winding/order, and exact open clipping;
- strict short-remnant equality, per-component rotation suffix behavior, and
  exact O44 connection rather than raw/chained paths; and
- empty public inset and empty-after-filter success, plus the initial offset
  failure, every otherwise reachable checked boundary failure, and at least one
  genuinely later-component failure proving no partial result or input
  mutation. Tests must not add a production fault-injection seam solely to
  manufacture unreachable failures.

The offset cast-order test must not rely only on KSR's zero overlap, for which
an early-f32 mutant happens to produce the same bits. Its Normal vector uses
spacing `0.1` and overlap `0.0501004999`, yielding f32 delta
`100.49990081787109` (`0x42c8fff3`), while an early-f32 mutant yields
`100.50088500976562`. Its LargeBed vector uses spacing `0.1` and overlap
`0.0510049999619`, yielding exact f32 `100.5` (`0x42c90000`), while the mutant
yields `100.49976348876953`. Literal rectangle output must distinguish the
resulting Clipper rounding on both scales.

At minimum, reversible mutations must make the following observably RED and
then restore production byte-for-byte: skipping the public offset; early f32
offset arithmetic; single-stage 1.08 multiplication;
host/truncating point conversion; truncating instead of floor remainder;
moving phase comparison before i32 narrowing; changing either strict branch;
closing the clipping subjects; changing the short-path erase predicate from
`<` to `<=`; bypassing O44; and rotating the entire accumulated output after
every component. Mutations that call O44 for an empty component or turn an
empty inset into an error must also be observably RED.

O45 CrossHatch unit/oracle/error tests use literal in-process geometry and never
read Orca source, helper files, the committed fixture, the golden G-code, or the
filesystem. The separate existing lifecycle and ignored golden integration
regressions may use their committed inputs; the lifecycle regression proves
O45 remains unwired and public slicing still disposes O43 at the incomplete
terminal.

Every Rust source file stays below 400 physical lines. Final verification
requires focused Nextest, relevant Clipper/O44/O43 bands, workspace Nextest,
rustfmt, workspace all-target/all-feature warning-denying Clippy,
ares-core/ares-wasm wasm32 checks, diff/whitespace/LOC/include/fixture-read
audits, the unchanged normalized golden progress probe, pinned-Orca source
restoration, and unconditional independent source/specification and standards
reviews.

## Completion evidence

The compiling empty-output stub produced the intended exact nonempty public-
oracle RED, and the transform stub produced its own branch RED. Final focused
O45 Nextest passes 34/34 (`083dc9db-5ad2-48a2-9612-ed1b2e39af68`). The
Clipper/open-intersection/O44 dependency band passes 305/305
(`8cfd30f0-bd9f-4402-b2ec-e1fa6339ab57`), and the O24-O26/O40-O45 predecessor
band passes 228/228 (`daf0e79f-dc26-4943-a7d2-a2e80b4691e8`).

The main public Orca harness remains
`dc41ed54fba644b589c41d4208847347b2c5e7626367660b1fd547d843ce542f`;
its byte-identical Debug/Release normalized output is
`17b755322c8d1e586e29145836f04ea728f4fdd846cce965430f8af1fea8691f`,
and its evidence record is
`831221270b56383b5a5cf1a1d25da94e937be07462a751f1941e60ec193cbe93`.
The supplemental raw-pattern-order harness is
`e07a31cedb92637b35750e6ac2b287a5dbddc644b6924ea14085502a5e92411e`
with byte-identical Debug/Release output
`bda674683e3990477401aeba3dcb3deec1a817f98d1fae049bc9b73744071f84`.
The current arithmetic harness is
`24040248e57f2dadb2aae060e1c32ecd357ffe57f57127453ea28d5ab4362200`
with stdout
`42434f1fad069e70c09e5538da1e173e2ce8919fe225c4b5fff8897608b10ea7`.
The supplemental public f32-repeat-ratio harness is
`5cf7c7847b079ff8d71b9240856ffd21f6ce3d1701ad5f1c12d8566a71ba7d84`;
its byte-identical repeated Debug/Release normalized output is
`e9b62afdc6fe0f7b03e4baf86d9c0e13e4692398f5ac89b6d8850bc82bd01aa2`.
The LargeBed oracle input was corrected to preserve source
`scaled<coord_t>` truncation toward zero; all four LargeBed public cases pass
without a production repair.

Eight arithmetic mutants were observably RED: early-f32 offset (`450edfd2`),
single-stage 1.08 (`ad7d22e6`), truncating point conversion (`5e5c4daf`),
floor-to-truncating remainder (`ca0f7ff5`), raw phase comparison
(`5f8ebdca`), non-strict `trans_z` (`24e1fe07`), non-strict progress
(`a9a5e4f7`), and strict-remnant reversal (`fdbc73aa`). Six composition mutants
were also RED: skipped public offset
(`392a2e7c-bfbc-4486-aca0-d58333d30749`), closed/recombined clipping subjects
(`11372d71-86bd-488e-bf86-e7c8906b3418`), bypassed O44
(`039b09c9-0bb3-4f31-9916-0de7d1604a0e`), whole-accumulator rotate-back
(`07f8a4bd-4687-4e36-a41b-8d109da26268`), O44 on an empty component
(`bc2558dd-0384-431f-b64f-2fb62bf9e532`), and empty inset as error
(`ab792c8b-97ed-4a67-9317-d200e97e11b8`). Two additional public-seam mutants
that widened f32 repeat-ratio arithmetic were RED: f32 product followed by
f64 `exp` (`f2038fcc-86eb-4e2d-9988-b9bd477c0186`) and full f64
multiplication/`exp` (`c3ecc38a-67c9-42de-9f43-ab6d35d28385`). Production
was restored to
`369d5c44a09822b05c6ef16770bc1431c61d2160a4cf28166bd58a1d5e7f46c4`
for `cross_hatch.rs`,
`e1cd61932b98e248152c75e862f736b5b0b32c755ed3e08363059856c426cb3a`
for `pattern.rs`,
`7a26f837fb94aed660e92354aa9338c3fb686d2d9fcaed56f64d3f82bff9b54a`
for `transform.rs`, and
`b8b385224223a2702b63e40732012e2d2f74abfb584bbd87419cdb3a1c816201`
for `geometry/clipper/polyline.rs` before final gates.

Workspace Nextest passes 6,235/6,235 with 30 slow and two skipped
(`818f7790-9db4-41d2-9206-ebb4f969f8a4`). Rustfmt, workspace
all-target/all-feature warning-denying Clippy,
and wasm32 checks for core and the browser adapter pass. The ignored normalized
golden probe remains the expected RED (`9f4804f9`) at the unchanged missing
`--options` boundary. O45 therefore verifies only the crate-private dependency;
it does not activate a public lifecycle stage or claim a G-code change. Final
independent source/specification and standards reviews unconditionally
approve.
