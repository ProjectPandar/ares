# Cubic sparse-anchoring filler dispatch

## Upstream boundary

This slice rewrites the Cubic dispatch inside
`Layer::generate_sparse_infill_polylines_for_anchoring`,
`src/libslic3r/Fill/Fill.cpp:1561–1687`, at upstream revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`.
Its downstream consumer is `PrintObject::bridge_over_infill`:
`PrintObject.cpp:3895–3942` constructs expansion boundaries and selects the
bridge direction; `:3480–3550` implements the geometry-based direction vote.

The anchoring function calls `Fill::new_from_type(surface_fill.params.pattern)`.
For `ipCubic`, `FillBase.cpp:85` returns **FillCubic**, not FillAdaptive.
`FillCubic::fill_surface` (`FillRectilinear.cpp:3603–3618`) dispatches:

- `params.multiline > 1`: `fill_surface_trapezoidal(..., 3)`;
- otherwise: `fill_surface_by_multilines` with the three Cubic sweeps,
  using `dx = sqrt(0.5) * z` and shifts `{dx, -dx, dx}`.

AdaptiveCubic and SupportCubic have a different filler boundary. Their octrees
and noncrossing multiline router are not dependencies of this Cubic slice.

## Parameter construction

`Fill.cpp:1564–1571` obtains the existing grouped fills and admits only
`stInternal`. The anchoring path retains the grouping result, region identity,
ordered surfaces and per-surface geometry; it does not regroup the object.

`Fill.cpp:1609–1622` sets the object's bounding box, object-relative layer id,
current print Z, region-specific alternation option, `surface_fill.params.angle`
and `fixed_angle` on the filler. These are the actual supporting layer's values,
not the bridge layer's nominal direction.

`Fill.cpp:1646–1669` constructs `FillParams` from that same grouped fill:

- density is `float(0.01 * surface_fill.params.density)`, converting the grouped
  percentage to a fraction with the upstream double multiplication and float cast;
- multiline is `surface_fill.params.multiline`, without replacing it with one;
- anchor lengths, resolution, actual layer height, pattern options and extrusion
  role are forwarded; `dont_adjust` and `use_arachne` are false.

Angle is a filler property, not a replacement density or multiline parameter.
`Fill::_infill_direction` (`FillBase.cpp:386–430`, quarter turn at `:429`)
supplies the frame's quarter turn;
the Rust filler parameters already fold this into their angle. It must occur
once, with the existing float evaluation order.

For each expolygon, `Fill.cpp:1671–1683` retains the existing per-body box/octree
selection, resets spacing from `surface_fill.params.spacing`, assigns the
surface geometry and calls the selected filler's `fill_surface`. The new filler
starts with overlap zero (`FillBase.hpp:219–220`); this is not the ordinary
extrusion fill's positive perimeter overlap. The result is raw anchor polylines,
not extrusion entities or a subsequent extrusion-chain joining pass.

## Rust destination and replacement

The private destination is the `ProcessInfillPattern::Cubic` arm of
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/sparse_anchoring/arms.rs:134–165`.
Its unconditional generic `fill_multiline_surface` call is a temporary
compatibility shell, not an independent Cubic anchoring algorithm.

Replace that unconditional dispatch with the upstream FillCubic selector:
use the existing `fill::cubic_trapezoidal::fill_surface` for multiline greater
than one and retain the existing three-sweep multiline call for the other
branch. Do not reimplement either generator or change their shared math.
`project_slice/fill_entities/cubic.rs::polylines` already expresses the same
upstream selector for ordinary Cubic output; its extrusion materialization and
end-joining tail must not be called for anchoring.

Retain the anchoring arm's spacing, zero overlap, frame angle, object reference,
fractional density, multiline, anchor lengths and sorting behavior. Forward the
existing supporting `layer_index`, print Z `z`, and the grouped representative
surface's `thickness_layers` into
`fill::cubic_trapezoidal::SurfaceGeometry { layer_id: layer_index,
thickness_layers: fill.representative.thickness_layers, print_z: z }`.
Do not derive layer identity from rounded Z or substitute the bridge
candidate's layer. Preserve ordered expolygon iteration and append order.

The change stays in private, platform-neutral core geometry. It adds no public
API, dependency, filesystem, terminal, native runtime or platform-specific path.

## Producer-entry prerequisite

Unequal anchors at the angle consumer establish an input difference, not its
cause. Before attributing that difference to the selector, compare initialized
Native and Rust entries for the same supporting object, layer, region and fill:

1. Identify the corresponding `SurfaceFill` in upstream `group_fills` output and
   the existing Rust grouped-fill handoff, including surface type, region id,
   pattern, ordered membership, expolygons and representative thickness.
2. Compare the effective configuration used by that group, including nozzle/flow
   inputs, density, multiline, angle, fixed angle and alternation settings.
3. Compare filler-facing spacing, density conversion, overlap, anchor lengths,
   object reference/bounding box, print Z and object-relative layer id.
4. Only with equal producer entries, observe the selected upstream FillCubic
   branch and the Rust call and compare their ordered anchor outputs. A static
   dispatch contrast alone is insufficient to assign the cause.

If entries differ earlier, locate that upstream producer boundary instead of
compensating in the filler or bridge-angle vote. Grouping, region identity and
configuration changes require their own source boundary; they are not included
here. Candidate-area and boundary equality does not imply grouped-fill equality.

## Included and deferred behavior

Included: the Cubic anchoring selector, its existing parameter conversion,
minimal supporting-layer/thickness forwarding, and ordered raw-polyline handoff.
The single-line branch remains the upstream branch, not an alternate fallback.

Deferred: changes to grouping, configuration resolution, body preparation, belt
transforms, bounding-box ownership, cache identity, all other infill patterns,
Adaptive/Support octree selection, bridge candidate filtering, automatic-angle
voting, anchored-area reconstruction, support ordering, retraction, timing and
configuration output. An unrelated complete-output mismatch is not repaired by
this slice or waived by a geometry-only comparison.

## Tests

The existing `runner::lateral_multiline_overlap::cubic_m2` is the target.
Keep its input fixture, Native invocation, validated API call, CLI call and
complete ordered generator-line-only comparisons unchanged. Do not create a
second test or copy the fixture. Preserve the actual baseline test exits and
complete output from both Native modes.

Use the existing `runner::cubic_trapezoidal` count-two, count-three and holed
count-two complete-output cases as controls. Its existing count-one
motion/progress assertion is only a diagnostic control: retain and separately
report its complete-output comparator result, without promoting the narrower
assertion to complete-output acceptance. Compare complete before/after API and
CLI bytes for unchanged controls.

Capture same-input Native repetitions and require generator-line-only equality;
retain raw validated API/CLI equality and complete ordered comparisons for the
target. CONFIG, headers, time and M73 remain strict. Record the actual test exits
and all remaining complete differences. Anchor/angle comparisons diagnose the
producer but do not replace the public-byte test.

Temporary diagnostics must leave no source changes in the implementation.
Retain repeatable captures bound to input, catalog, source and executable
identities. Run rustfmt, strict workspace/all-target Clippy and the browser WASM
build checks; keep changed production modules within 400 lines.

## Failure modes

1. Matching bridge-candidate geometry conceals different grouped producer inputs.
2. Region membership, effective options or object-relative layer ids are paired
   incorrectly, assigning an earlier producer difference to the filler.
3. The percentage-to-fraction conversion changes precision or is applied twice.
4. Multiline greater than one still invokes the generic three-sweep generator.
5. The single-line branch is changed unnecessarily or routed through another pattern.
6. The quarter-turn frame rotation is omitted, duplicated or evaluated differently.
7. Print Z is used to reconstruct layer identity, or representative thickness is lost.
8. Ordinary extrusion overlap, joining or entity materialization contaminates anchors.
9. Expolygon iteration or raw-polyline append order changes independently of geometry.
10. Unequal anchors are compensated by altering the automatic bridge direction.
11. A motion-only diagnostic hides strict header, time, M73 or CONFIG differences.
12. Temporary diagnostics, a new public boundary, or unrelated pattern changes remain.
