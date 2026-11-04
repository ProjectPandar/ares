# Classic narrow-wall-hole sandwich eligibility guard

## Upstream boundary and clarified scope

Pinned beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`PerimeterGenerator.cpp`:

- `PerimeterGeneratorLoop::is_narrow_wall_hole:72` defaults false.
- `:1713-1718` computes this flag only for InnerOuterInner wall sequence.
  Reuse the first-shell `reach(room)`/`out_of_reach` operators: the room
  for inner walls is `ext_perimeter_spacing2 + min_spacing / 2.`.
  `is_narrow_wall_hole = narrow_holes_ordered && out_of_reach(...)`.
  Reach uses only the unreduced normal first offsets, not smaller-width
  offsets or the region left after top-wall reduction. All first-shell
  holes, normal then smaller-width, are classified against that reach.
- `:1861-1863` defines the narrow island using `contours.front()`:
  exactly one root contour, one child and that child's narrow flag.
- `:1867` adds `!narrow_wall_hole_island` to the existing InnerOuterInner
  branch, preserving physical `layer_id > 0` and outer-first/brim
  precedence. A narrow island retains hole, contour, then gap-fill order.

The clarification explicitly includes this **eligibility guard** at1859-1867.
The actual sandwich reorder body at1868+ is already ported and is **unchanged**.
Do not rewrite/reverse its permutations, change gap ordering, or claim a
larger sandwich algorithm port. This supersedes the earlier broad deferral
of the guard, not the constraint to leave its existing body alone.

## Rust destination and carrier

Under `crates/ares-core/src/project_slice/perimeters/classic/`:

- Extend the existing source-compatible first-shell reach classifier in
  `onion/thin_holes.rs` and its per-record flow/sequence inputs with the
  **separate narrow flag**. Do not alter accepted thin-hole semantics.
- Carry the property alongside its corresponding first-shell hole through
  onion/hierarchy, `hierarchy::PerimeterGeneratorLoop` and
  `traversal::TraversalSeed`. Contours/deeper holes default false, and the
  narrow flag stays false outside InnerOuterInner. No new option/API.
- At `perimeter_append.rs` consume the current surface's nested root
  topology and raw narrow bit before the existing reorder decision, not
  printed endpoints or a collection entity-count proxy. Add the exact
  narrow-island condition to skip the existing sandwich branch only.
- Keep `sandwich_reorder`'s executable body and its call's ordering
  operations unchanged. Existing carriers are temporary compatibility
  shells until they carry this upstream property; no legacy fallback.

Reuse normal first-shell geometry, existing scaled flow operands,
`min_spacing`, offset2/bbox clipping/open intersection and EPS/casts exactly.
No speculative kernel, geometry widening, count heuristic or global reverse.
Preserve gap fill, thin-wall classification, nesting, inset IDs, seam
placement, wall direction, physical layer mapping and other consumers.
Core stays browser-WASM/Windows/macOS/Linux portable without I/O/UI.

## Citation-only correction boundary

Existing `perimeter_append.rs::reorder_walls` comments cite older
`PerimeterGenerator.cpp:1467` and1470-1560. On pinned beta the outer-first
branch is1864-1865 and sandwich eligibility/body begins1867-1868. Those
comments on the touched guard must be corrected when implementing it.
No unrelated documentation cleanup or sandwich-body edit belongs here;
any nearby citation update must be checked against this pinned source.

## Real public RED candidate

Input `tests/parity/classic-narrow-island-guard/narrow-tube.3mf`, SHA256
`01b634386258e99d65c8de97f2c47578917388c3b259a32128a5e09125bba372`.
Its deterministic input-only generator uses a closed oriented faceted
circular tube with outer/inner radii10/8.55mm, thickness1.45mm, height0.8mm.
The exact InnerOuterInner wire token is `inner-outer-inner wall`
(`PrintConfig.cpp:366`), verified before this corrected candidate.
Inherited standalone Ender settings are explicitly set to Classic,
InnerOuterInner, three requested walls, uniform0.45mm line widths,
0.2mm layers, no top/bottom shells/one-wall reduction/brim/support/fuzzy
skin/overhang reversal/arc fitting, and enabled residual gap fill.
This admits ordinary physical layers above zero where the guard matters.
The generator verifies mesh manifold orientation/positive volume and
records all authored changes/member hashes, with no Native G-code input.

The Native runtime must actually print hole then contour then gap on a
later layer while current Ares enters sandwich ordering. Printed role
labels alone do not distinguish two outer-wall loops or prove a private
flag. Qualify actual extrusion runs/spatial nesting and the first ordered
difference externally; no fixed Native coordinate/count golden in tests.
If first-shell geometry or an earlier unrelated difference prevents this
candidate from qualifying, stop and preserve that result rather than
silently changing thickness/inputs/producer or expanding production scope.

The dedicated option-coverage leaf runs Native2 fresh stable exports,
validated public API1 and CLI1, retaining the complete strict ordered-byte
comparison. ROOT, runners, comparator and all previous registrations stay
unchanged. Only canonical generator lines normalize; M73, XYZ/E order,
travel/feed, TYPE/WIDTH, wipe/retract, labels, terminals, config/footer,
whitespace and trailing bytes remain required. An external M73-command-only
diagnostic retains maps/all removed records/full contexts; no Fword filter
or filtered acceptance. No raw Native reach/flag observation is claimed.

## Corrected candidate and actual Native premise

The prior 1.5mm candidate used an invalid wall-sequence spelling. Its
Native footer reverted to InnerOuter, the premise callback stopped before
API/CLI, and no RED credit was claimed. In the corrected candidate only
the exact enum token and inner radius8.5→8.55 changed; all other settings
and ZIP members except the mesh are byte-identical.

The corrected run completed with Native2 fresh stable complete
exports, validated API1 and CLI1, raw API/CLI identical. At physical
Z0.4,0.6,0.8, Native has exactly two nested closed Outer-wall boundary
runs followed by one closed Gap-infill run, with no Inner-wall runs:
CW child hole, CCW enclosing contour, then gap. Ares instead prints
CCW contour, CW hole, then gap. At Z0.2 both retain hole→contour→gap,
consistent with the physical-layer gate. The complete emitted island
order, ring points, containment, winding and line ranges are retained
externally; labels alone were not used to identify the hole. No private
Native PG tree/flag operand observation is claimed.

The full strict first difference is byte1898/original85: Native
`G1 X108.712 Y101.32 E.02857`, API/CLI `M73 P49 R0`. M73 stays in the
comparator. The external M73-command-only diagnostic first differs at
original304 in both streams, diagnostic289/byte7532/column7: Native
`G1 X118.68 Y111.288 Z.8`, API/CLI `G1 X112.375 Y100.518 Z.8`.
These are travels selecting different first walls on the next layer;
subsequent full runs prove the order above. No fixed coordinates/counts
are test goldens, and no other record class was omitted externally.

## Literal port and strict GREEN qualification

The public RED predates the production port. The existing reach classifier is byte-identical: onion
configuration supplies the separate optional inner-wall room only for
InnerOuterInner, and onion calls that classifier on the same unreduced
normal first offset. Narrow flags follow normal/smaller hole order into
PG loops and seeds, false on contours/deeper holes and other sequences.

The current surface's nested first-depth roots implement exactly the
one-root/one-child/child-narrow predicate. The guard skips both collection
reverse and the sandwich call at physical layers above zero; outer-first
and brim precedence are unchanged. `sandwich_reorder` is byte-exact,
including its comments. Only the touched guard's stale citations change:
outer-first1864-1865 and eligible sandwich1867-1870/reverse1868.
Accepted thin-hole classifier/orientation, geometry kernels and other
adjacent behavior remain unchanged.

A frozen 124-check existing Classic/perimeter/wall-sequence/thin/narrow/
seam family ran before and after the port. **54 PASS / 70 FAIL → 55 PASS /
69 FAIL**, zero ignored, exactly one FAIL→PASS (this public target), zero
prior-PASS to FAIL. Guard/premise results remain visible, not blanket
Native-parity credit.

Per phase, the dedicated target has two fresh stable Native exports and
one validated API/CLI export, raw API/CLI equal. The AFTER strict complete
output is **GREEN**, not just an externally filtered prefix. Native,
Native replay, API and CLI equal 51,593 bytes / 1,781 lines after the sole
canonical generator normalization, SHA256
`567116c7cab69982a93b4b4b5affb87b4a0fbd2b6b100900f5b5d35a9834d159`.
All 46 M73 commands, ordered motion, roles/width/feed, wipe/retract,
labels, config/footer, terminal whitespace and trailing bytes remain.
There is **no remaining full or non-M73 first difference** on this input.

All four physical layers now have identical Native/API closed CW hole,
closed CCW enclosing contour, then closed Gap run, with exact saved
extrusion-run/point/line identity. This verifies the full island order,
including the reverse eligibility, not merely the first loop or TYPE
labels. No private Native flag trace or new runtime probe was used.

Tests, fixture/generator, ROOT, runners and comparator are unchanged from
the RED.

## Included, deferred and exit criteria

Include only narrow classification, carrier and eligibility guard. The
existing sandwich body/permutation, accepted thin-hole orientation, Flow,
geometry kernels, Arachne/brim/overhang logic, seam and G-code serializer,
M73 estimator and config footer are separate and unchanged. No private
operand trace or further runtime probes are part of this one-candidate RED.

The ARD and corrected fresh RED preceded the declared literal port. Its
strict GREEN and paired family evidence satisfy this slice's runtime gate;
remaining 69 family failures are out of scope. Any later change needs its
own source-cited boundary; do not grow this guard into a sandwich-body rewrite or a
new geometry/estimator/config policy.
