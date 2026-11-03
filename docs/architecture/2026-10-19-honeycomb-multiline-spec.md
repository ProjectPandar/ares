# Regular Honeycomb multiline anchoring surface contraction

Status: SPEC-AMENDMENT-REVIEW-REQUEST; original SPEC and RED approved;
GREEN paused pending review of the localized owner and destination.
Current base: `fb9a4ecd5`; approved chain: `9300e3c74` → `da72d0703`.

## Native-first finding

Using the existing input-only `tests/parity/cube10.stl` and the live
`honeycomb_multiline` harness, with `sparse_infill_pattern=honeycomb`,
`fill_multiline=2`, `sparse_infill_angle=45`, and `sparse_infill_density=20%`,
Native was admitted and the strict whole-file comparison diverged in both
modes. Official evidence is under
`${EVIDENCE_ROOT}/w3-honeycomb-official2`; seeded evidence is under
`${EVIDENCE_ROOT}/w3-honeycomb-seeded`. Each mode had three fresh
Native runs; the target remained divergent on all three. The first difference
is byte 83637 (line 3727): Native `G1 X114.003 Y111.205 E.08267`, Ares
`G1 X114.003 Y111.205 E.08266`; neighboring motion is identical and both
outputs are 121942 bytes. The controls `fill_multiline=3` at 45 degrees and
`fill_multiline=2/3` at 0 degrees with 40% density passed in both modes.

## First differing intermediate and upstream owner

Localization used CPU12 GDB reads, without production probes. The auxiliary
Native diagnostic build is not an oracle; strict admission/equality continues
to use official and seeded Native runners. Diagnostic evidence is under
`${EVIDENCE_ROOT}/w3-honey-native-entry/gdb.log` and
`${EVIDENCE_ROOT}/w3-honey-current-red-gdb.log` (current RED's
freshly rebuilt test executable). Native diagnostic-source `Fill.cpp`,
`FillBase.cpp`, and `FillHoneycomb.cpp` compare byte-identically to the pinned
upstream checkout `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

At layer 42, the first Honeycomb call is the sparse anchoring projection. Both
sides carry spacing `0.40707963705062866`, density `0.20000000298023224`,
multiline 2, and the same layer/angle inputs. The first differing intermediate
is the expolygon handed to `FillHoneycomb::_fill_surface_single`:

- Native contour: `(4019693,4019693)`, `(-4019688,4019693)`,
  `(-4019688,-4019688)`, `(4019693,-4019688)`.
- Ares contour: `(4223233,4223233)`, `(-4223228,4223233)`,
  `(-4223228,-4223228)`, `(4223233,-4223228)`.
- The later emitted-fill call agrees at contour bounds `-3612599..3612604`.
  Thus the observed mismatch is in anchoring parameter/domain assembly,
  not evidence for changing the Honeycomb lattice or actual-fill contraction.

The owning source sequence is
`OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1394-1504`:
`Layer::generate_sparse_infill_polylines_for_anchoring` creates a filler with
zero overlap (`FillBase.hpp:187`), resets its spacing, and calls
`f->fill_surface` at line 1497. Regular `ipHoneycomb` uses the base
`Fill::fill_surface` (`FillBase.cpp:105-117`), which offsets by
`float(scale_(overlap - 0.5 * spacing))` at line 108 before calling
`FillHoneycomb::_fill_surface_single` (`FillHoneycomb.cpp:9-80`). Ares's
Honeycomb arm in `prepare_infill/bridge_over_infill/sparse_anchoring/arms.rs`
bypasses that base wrapper and passes the uncontracted surface directly.

Downstream diagnostics agree with the geometric ownership: the layer-43
Internal Bridge input surface and spacing match, but its bridge angle is
Native `1.8325960636138916` versus Ares f32 `1.8325961828231812`.
The emitted bridge segment lengths are respectively `1699529` and `1699527`
in integer coordinates, while both flows have
`mm3_per_mm = 0.11938051879405975`. Supporting logs are
`w3-honey-native-rect4/gdb.log`, `w3-honey-native-localize/gdb2.log`, and
`w3-honey-current-red-gdb.log` under the same evidence root.
`PrintObject.cpp:2848-2936` consumes these sparse anchor lines to determine
the bridge angle. This does not authorize changing that detector, Flow,
rectilinear geometry, or E formatting; final causal verification belongs to
GREEN's unchanged strict whole-file test.

## Ares destination and scope

The production destination is the regular Honeycomb arm in
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/sparse_anchoring/arms.rs`:
restore the upstream base-fill half-spacing contraction before invoking the
existing `fill/honeycomb.rs` kernel, independently for each offset component.
The existing anchoring arm is a temporary compatibility shell around
`Layer::generate_sparse_infill_polylines_for_anchoring`; it must preserve
that upstream wrapper rather than invent an Ares-specific anchoring rule.
The already-contracted actual-fill wrapper in
`project_slice/fill_entities/honeycomb.rs` remains unchanged.

Included behavior is only regular Honeycomb's zero-overlap base-fill surface
contraction in sparse anchoring, using the existing Clipper offset primitive
and native float-scaled offset. Deferred behavior includes changes to lattice
arithmetic, multiline offset/connection helpers, simplification, automatic
bridge-angle selection, Flow, E arithmetic/rounding, lateral overlap removal,
ThreeDHoneycomb, SupportCubic, TPMS-D, ironing, and other pattern arms.
If restoring the measured contraction does not satisfy the unchanged target
and controls, STOP and localize again; do not adjust E or widen this scope.

The approved RED must use authored input fixtures and runtime Native output,
strict whole-file equality, and controls that assert equality. It will cover
the discovered `multiline=2, angle=45, density=20%` target plus controls for
multiline 3 at the same angle and multiline 2/3 at angle 0 and density 40%.
No Native artifacts, hardcoded output, retries, inverted assertions, guards,
or comparator weakening are allowed. Both official and seeded runs remain
CPU12-only with fresh target/artifact/cache roots.
