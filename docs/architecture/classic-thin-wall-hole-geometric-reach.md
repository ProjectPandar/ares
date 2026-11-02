# Classic thin-wall-hole geometric reach

Status: literal thin-hole classification/transport/orientation port qualified,
explicitly NON-GREEN pending complete parity and source-port review. The
approved public RED remains unchanged; no complete GREEN is claimed.

## Upstream owner

OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`PerimeterGenerator.cpp`:

- `PerimeterGeneratorLoop::is_thin_wall_hole:68` is an explicit false-by-default
  loop property, distinct from contour/depth/smaller-width status.
- `:1507` defines the gap-fill minimum as
  `0.2 * min(perimeter_width, ext_perimeter_width) * (1-INSET_OVERLAP_TOLERANCE)`.
- `:1699-1717` classifies first-shell holes after `last` receives normal
  offsets, before top-wall reduction. Its reach is flattened
  `offset2_ex(last, -float(room), float(room)+float(SCALED_EPSILON))`,
  with `room=0.5*(ext_perimeter_spacing2+gap_fill_min_width)`.
  Intersect the hole's closed split polyline with reach clipped to the
  EPS-inflated hole bounding box. Only an empty intersection means thin.
- `:297-307::traverse_loops`, especially303, requests CCW exactly when
  `(wall_direction == CounterClockwise) ==
  (loop.is_contour || loop.is_thin_wall_hole)`.
  It no longer propagates a topology-reversal flag or reverses accumulated
  frame output as a thin-hole side effect.

Old `8500fcdccaa10b5099ac20d252af3a7c560046f1:252-267` instead uses one
contour with one child hole and no grandchildren as the recursive reversal
proxy. Rust still implements that old predicate and collection reversal.
This slice rewrites those upstream concepts, not an Ares winding policy.

## Rust destination and non-negotiable behavior

Under `crates/ares-core/src/project_slice/perimeters/classic/`:

- Compute classification from the **unreduced normal first offset**,
  available as `top_split::prepare_surface`'s `first.normal` and
  `onion::IterationInput.normal_first_offset`. Never use post-top-split
  `remaining`, the effective perimeter count or final tree topology.
- Carry the bit alongside the corresponding first-shell hole through
  existing onion/hierarchy materialization, `hierarchy::PerimeterGeneratorLoop`
  and `traversal::TraversalSeed`, then consume it in
  `entity_collections/orientation.rs::orient_loop`.
- Delete `entity_collections/traverse.rs:120-124`'s old recursive proxy,
  its frame flag propagation and the obsolete frame output reversal
  at179-181. Do not preserve them as a fallback or another orientation mode.
- Reuse the existing geometry primitives and flow operands. Preserve
  upstream float casts, EPS addition, bbox clipping, intersection emptiness,
  hole identity and ordering. No independently designed reach kernel.

Current loop/seed metadata is only a compatibility shell until it carries
the cited beta property. No blanket hole reverse, seam-winner/score fix,
coordinate/count oracle, serializer change, extra option or public API.
Do not transplant Arachne's separate `pg_extrusions.size()==2` predicate
into Classic. Core remains browser-WASM/Windows/macOS/Linux portable,
without filesystem/UI/terminal behavior or a new crate.

## Included and deferred

Include the thin-hole property, its geometric first-shell classifier,
Classic orientation consumer and removal of the superseded topology proxy.
The adjacent narrow-hole flag at1718 and InnerOuterInner sandwich guard
at1859-1867 are **deferred**; this witness uses InnerOuter. Do not imply
their parity or silently add their behavior to this slice.

Arachne, brim eligibility, top-wall reduction policy, onion counts,
mesh slicing, paint, seam scoring/visibility, Flow math, G-code formatting,
accepted spiral/gap ports, M73 estimator and config footer remain separate.

## Exact public RED

Base accepted source stack `94c678d2252abb1b55fc9f3930e13c80eecdb146`.
Reuse unchanged `tests/parity/spiral-lift-arc-enabled/sequential.3mf`,
SHA256 `baa6c8d352d5e62c400cc450d50de643ce868fcc619a71352748f89aabe7aab1`.
No fixture copy/setting edit or minimization variant is needed.

The dedicated option-coverage leaf runs fresh ordinary Native2, verifies
stable generator-only equality, then validated public API1 and CLI1.
Native settings premise: Classic, ccw, two walls, one-wall-top enabled,
InnerOuter and the real cube-with-hole object. No coordinate/count pins.
ROOT, shared runner, comparator and previous/retired registrations stay
unchanged. One selected run only; no retry, extra probe or family run.

Prior saved qualification identifies a later Outer-wall difference at
original13978 on both streams, object `cube_with_hole.obj id:1 copy 0`,
Z10. The **same** printed seam starts at `(158.853,89.79)`; Native next
extrudes to `(158.853,100.21)` clockwise, Ares to `(169.273,89.79)`
counterclockwise. E.32018, WIDTH.42 and F6445 match at that first segment.
These are diagnostic observations, not committed test-coordinate goldens.
Earlier matching motion includes the accepted gap/spiral corrections.

The complete strict comparison retains M73, XYZ/E order, travel/feed,
TYPE/WIDTH, wipe/retract, labels, config/footer, whitespace and trailing
bytes. Only canonical generator lines normalize. M73 remains separately
owned; the earlier full M73 failure is not masked to award GREEN.
An external M73-command-only diagnostic may qualify the actual first
non-M73 wall difference, retaining original-line maps, all removed records
and complete unfiltered contexts. It never determines test acceptance.

Actual private seed topology and Native reach booleans were not observed
in the prior read-only localization. They must not be invented from the
printed role. A bounded Ares-only carrier trace is authorized if needed
before the fix, but no trace is planned in this unchanged-production RED.
Any such trace is scratch evidence, not committed instrumentation.

## Fresh qualification

One selected, nonignored ordinary public run on the accepted base finished
with the expected strict FAIL. Two fresh Native complete exports are stable
with generator-only normalization; validated API and CLI exports completed
and are raw-byte identical. The first full difference is byte2475/line93:
Native `;WIPE_START`, API/CLI `M73 P10 R18`. This remains a test failure.

The separately retained external M73-command-only diagnostic first differs
at original13978 in both streams, diagnostic13870/byte363740/column6,
with the wall endpoints described above. Both preceding printed starts
remain `(158.853,89.79)`. No Fword or other record class was omitted.
Original-line maps, all removed M73 commands and full contexts are retained
in the external evidence bundle. Printed coordinates are not test goldens.

There was no carrier trace, private-operand observation, input variant,
extra probe, family run, retry or production change. A fresh dedicated
target, frozen ELF and exact source tree were verified before execution.
Fmt and strict workspace/all-targets Clippy pass. Only this architecture
result note changes after execution; no postcommit rerun is claimed.

## Literal port and paired perimeter qualification

The port adds `onion/thin_holes.rs` and computes the source room from
existing scaled prelude widths/spacings. The classifier reads unchanged
`normal_first_offset`, never reduced `remaining`; smaller-width holes
use that same normal reach, as upstream does. First-shell flags follow
normal-hole then smaller-hole order into `PerimeterGeneratorLoop` and
`TraversalSeed`; contours and deeper holes retain false. The exact seed
bit reaches orientation across child descent. Old topology proxy, frame
flag propagation and accumulated frame reversal are deleted, not retained
as fallback. Arachne's existing helper call operands are unchanged; only
the shared boolean parameter's name now describes the source property.

A frozen 117-check existing perimeter/Classic/seam/thin-hole family ran
once before and once after in the same worktree with a dedicated target.
Both phases are **48 PASS / 69 FAIL / zero ignored**, with zero flips and
zero prior-PASS to FAIL. Each phase retains 170 Native command-status
receipts. Guard/premise results remain visible; these PASS counts are
not blanket Native-parity credit. The dedicated public target has two
fresh stable Native exports and one validated API/CLI export per phase,
with raw API/CLI identity.

The former original13978 CW/CCW difference no longer leads the external
M73-command-only diagnostic. Its first remaining non-M73 difference is
now original15055 in both streams, diagnostic14945/byte403792/column19:
Native `; printer_agent = crealityprint`, API/CLI `; printer_agent = `.
The ordered non-M73 prefix includes matching wall edges, seam-gap closing,
wipe and subsequent motion before this config footer. No additional
record class was omitted; full original-line maps, removed commands and
complete contexts remain in the external evidence bundle.

The full strict test still **FAILS** at original93/byte2475: Native
`;WIPE_START`, API/CLI `M73 P10 R18`. M73 remains in the comparator and
separately owned; the footer remains in the existing config queue. No
estimator/config/formatter patch, optional carrier trace, GDB, input
variant, retry, extra probe, family expansion or E5 repeat occurred.
Narrow-hole/sandwich behavior remains deferred, not claimed ported.

Fmt and strict workspace/all-targets Clippy pass. All production postimages
match the executed AFTER tree, with distinct frozen before/after ELFs and
identical test inventory. Tests, fixtures, ROOT, runners and comparator
are unchanged. Only this architecture result note changes postexecution;
no postcommit rerun or complete GREEN/FULL/integration claim is made.

## Exit criteria and remaining gates

ARD/public RED `118424f36f906283177a9a3b13ba63b799b0b70f` was approved
before production edits; the declared source boundary and paired family
are qualified above. Preserve all source/executable/manifests, original
failures and any invalid attempts. Complete GREEN still requires the full
M73-inclusive output; any estimator/config prerequisite must be named and
approved explicitly. Report new boundaries instead of broadening scope or
waiving records. E5 was separately qualified after the accepted gap port,
with its former feed discrepancy gone; it was not rerun here, and no cooling
slice belongs to this thin-hole rewrite.
