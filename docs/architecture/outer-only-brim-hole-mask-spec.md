# SPEC-REQUEST: Brim.cpp OuterOnly contour and hole exclusion

## Status and roadmap selection

Worker3; base main `5d1a77062`, after the approved combined-brim merge.
Claude, the sole reviewer, approved SPEC a68680255. The original submission
was documents only. Separate fixtures/tests now have admitted Native RED;
[evidence](outer-only-brim-red-evidence.md) is reported before implementation.

Selected from roadmap **Current acceptance priorities: full acceptance (open)**:
active brim selectors need source-owned, nonempty complete-output witnesses.
The combined-brim SPEC explicitly deferred OuterOnly's hole semantics. This
slice replaces that bounded project-runtime scaffold; it does not close the
seven-value brim domain or global printer/Option acceptance.

Production claims are limited to private brim modules and existing project
BrimPlan dispatch. Coordinator records currently place worker1 in template/
anchor work, worker2 in timelapse, worker6 in Stars geometry, worker5 in
profiles, worker4 in CI/oracles and web in viewer data. No file owned by those
slices is proposed here. Brim overlap with merged worker3 code is intentional
and is reported in this SPEC request before any implementation.

## Upstream ownership and Rust destination

Source: repository read-only OrcaSlicer v2.4.2 seed-replay source link.
Paths below are relative to `OrcaSlicer/src/libslic3r`.
`Brim.cpp` SHA256:
`807c3a449dfc07a887959f3f7badb86eab9dd75e017ff136e03d04f96979dbe3`.
`ClipperUtils.hpp` SHA256:
`09d01802a5d3e4d4a450a25c22bdc2e7db95e804184c90da890376721f7bca04`.
Rust destinations are relative to `crates/ares-core/src`.

| Upstream function and lines | Rust destination and treatment |
|---|---|
| `Brim.cpp::outer_inner_brim_area:448-458` paired width and selector predicates | Existing `project_slice/gcode_emit/brim.rs::BrimPlan::generate`; preserve explicit OuterOnly admission and initial flow/scale |
| `Brim.cpp:512-545` contour-only Round outer area, including line527 re-quantization | New private `brims/outer.rs::area`, registered in `brims.rs`; replace the current whole-ExPolygon outer construction |
| `Brim.cpp:529-530,560-562` reversed holes and no-inner mask | Reuse `brims/inner.rs::reversed_holes`; form `holes - offset(holes, -no_brim_offset)` with default Miter3 |
| `Brim.cpp:564-570` no-outer predicate and objectIsland | OuterOnly has no no-outer mask; Round-offset actual source ExPolygons by gap for objectIsland and append it to exclusions |
| `Brim.cpp::make_brim:903-904`; `ClipperUtils.hpp:19-27,344-350` | `no_brim_offset` is scaled flow spacing, not object gap or brim width; hole-mask inset uses default jtMiter/limit3, without Round resolution |
| `Brim.cpp:661-679,686-711` subtraction and contact retention | Reuse `gcode_emit/brim/bed_area.rs::clip` and `brims/inner.rs::retain_contact`; extend the latter's dispatch to explicit OuterOnly |
| `Brim.cpp::tryExPolygonOffset / makeBrimInfillImpl:813-875` | Reuse `brims/ears/paths.rs::from_area` and existing emitter; no new G-code ordering or independent stream |

The new private area result carries source area, exclusions and objectIslands,
as required by the existing clipping/contact consumers. It is not a new
public API or generalized brim-strategy abstraction. Existing geometry
primitives and their global defaults remain untouched.
[Proposed runtime ARD](ard-outer-only-brim-hole-mask-boundary.md).

## Existing scaffold and replacement

At base5d1, OuterOnly falls through BrimPlan's final branch: offset the complete
source ExPolygons by gap, offset that result by width, then subtract. Its
exclusion is only the first offset; contact retention is applied only to
InnerOnly/OuterAndInner. This is a static source-boundary mismatch, not a
claimed Native RED or proof that every hole input currently diverges.

Upstream constructs each outer ring from the contour alone, adds an independent
no-inner hole-edge mask, then adds objectIsland and applies final contact
filtering. Replace the OuterOnly construction and dispatch with that boundary.
Do not concatenate InnerOnly output or reuse its no-outer mask. BrimPlan stays
a temporary compatibility shell around `outer_inner_brim_area`, `brim_flow`
and `makeBrimInfillImpl`; no Ares-owned adhesion pipeline or legacy fallback
is introduced.

InnerOnly, OuterAndInner and BrimEars producers keep their current behavior.
The public rectangular `generate_brims` adapter remains deferred: no routing
through it, no assertion retirement or widening of this proposal to that API.
Do not refactor neighboring code merely to consolidate error helpers.

## Included behavior

- One object, instance, model-part volume/group and filament/nozzle; ordinary
  first-layer source slices, explicit OuterOnly. Support, prime tower, skirt,
  painting and EFC outline are disabled. Holes and disjoint, non-nested
  footprint islands within the single volume are included.
- Preserve line450 paired width and line527 outer re-quantization. Explicit
  OuterOnly has no automatic adhesion addition or bounding-box width clamp.
- For each ExPolygon, Round-offset its contour by gap, Round-offset that
  boundary by outer modified width, then append the difference. Pass
  SCALED_RESOLUTION only to the Round operations as upstream does.
- Reverse hole winding. Because has_inner_brim is false, add the hole-edge
  exclusion `diff_ex(holes, offset_ex(holes, -spacing))` using default Miter3.
  A collapsed inset leaves the complete hole in this mask. There is no inner
  ring generation and no no-outer contour expansion mask.
- Round-offset the original ExPolygons, including holes, by gap for
  objectIsland; append these to exclusions. Apply existing printable-area
  subtraction, then retain fragments whose 2*spacing Round expansion contacts
  objectIslands or another pre-filter fragment of this object. The other-object
  set is empty in this scope. Gap alone is not a detached-fragment witness:
  objectIsland itself already includes that gap.
- Pass retained area through the existing shared path producer and emitter.
  Preserve upstream append order and whole output, including travel/E/M73/
  footer. Width0 and sub-pair collapse legitimately produce no brim.

## Deferred behavior and scope exclusions

AutoBrim heuristics, ear/painted behavior, EFC-adjusted outlines, support/tower
brims, nested islands in another island's hole, multiple objects/instances/
volumes/groups/tools and cross-object contact composition remain deferred.
So do public rectangular adapter repair/test retirement and further geometry
kernel work. No template/timelapse, APA/anchor, profiles, Stars/multiline,
CLI policy, oracle/CI, viewer, option registry or platform-specific core code.
If admitted RED first fails in one of those owners, report it rather than
repairing it under this SPEC.

## Failure inventory before tests or implementation

1. Input mesh has no actual hole, changes grouping, or is rejected by Native/API/CLI.
2. Whole-ExPolygon offsets emit extra hole-side brim paths or use wrong winding.
3. Outer width omits the second quantization or acquires automatic modifications.
4. Hole mask uses gap/width, positive inset, Round, or the wrong miter limit.
5. Reusing InnerOnly's no-outer mask erases legitimate exterior brim.
6. ObjectIsland uses contour-only geometry instead of original ExPolygons.
7. Bed-clipped detached fragments survive, or valid contact fragments disappear.
8. Multi-island append order or shared-path emission changes unrelated bytes.
9. Existing combined/inner/ear controls regress, or inactive widths emit brim.

## Test-first plan and review exits

1. After approval, author deterministic input-only 3MFs and Native-backed CLI
   integration tests under `tests/parity/outer-only-brim` and
   `crates/ares-cli/tests/orca_parity/outer_only_brim.rs`, registered through
   existing `brim_bed_region.rs`. No top-level parity/case-helper restructuring.
   Proposed finite inputs: rectangular hole, concave hole, two holes, narrow
   neck, positive gap, collapsed hole inset, two non-nested footprint islands,
   no-hole control, width0 and sub-pair width. Keep real single-volume meshes,
   signed-volume/edge-closure provenance and exact input/member hashes.
2. Active inputs must show positive first-layer exterior Brim XY/E and no
   hole-side brim; inactive widths must show no brim. Use authored contours
   and holes, not bounding boxes. Add a proposed printable-region clipping
   input to distinguish retained exterior paths from a detached fragment.
   Native/API/CLI admission and actual clipping/contact witnesses must be
   established before counting that case; an inadmissible or inactive case
   is reported and reshaped before RED, never credited as contact coverage.
3. Save effective Native settings, commands, input/output SHAs, full Native,
   validated byte API and real CLI G-code, regional motion witnesses and first
   byte differences. Witnesses supplement unchanged generator-only ordered
   full-byte comparison; Native output is never checked in as fixture data.
4. Run seeded/official with original control-set-v2, jobs2/threads2, load<30,
   fresh cache/root every run. Existing InnerOnly/OuterAndInner/N/O/E and bed
   controls retain their assertions/comparators; compare their raw Ares bytes
   against pre-implementation evidence, with hole behavior changes identified
   explicitly rather than claiming all existing OuterOnly bytes must freeze.
5. Commit fixtures/tests plus admitted Native RED separately and report RED
   to inbox before writing implementation. If existing no-hole controls change,
   audit their exact source cause and report; do not weaken an assertion or
   invent a compatibility branch to restore a previous scaffold result.
6. Implement only the approved source boundary. Changed Rust/new docs <=400
   lines, no probes/ignore/suppression/regions[0], no unit tests written after
   implementation. Complete finite GREEN, directed both modes, strict/fmt/WASM.
7. Workspace/full require explicit coordinator allocation. Use standard
   workspace environment and canonical full environment with each full leg's
   fresh cache/root/target, threads2 and load<30. Rebase/carry, Native anomaly
   adjudication, code review and exact-hash merge approval remain Claude's
   decisions. SPEC approval does not authorize push or merge.

This is a source-cited proposal only; no Native outcome or implementation
acceptance is claimed at submission.

After approval and separate RED9c589662e report, the bounded port reaches
finite GREEN47/47 in both modes. All138 existing/inactive raw Ares streams
remain unchanged; see [GREEN evidence](outer-only-brim-green-evidence.md).
Committed-head directed, code review, workspace/full and merge approval remain
separate required gates.
