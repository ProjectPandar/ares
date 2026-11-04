# Proposal: per-object skirt groups, first bounded rewrite slice

## Non-negotiable scope and sequencing

The first slice covers single-filament by-layer printing, no
brim/support/raft/tower/draft, `min_skirt_length=0`, including disjoint,
overlap, transitive and copies cases. The existing `skirt.rs:64-73` guard
may only be narrowed to deferred conditions (brim/support/raft/tower/draft,
minimum-skirt extension, sequential or multiple filaments). Add **no new
UnsupportedProjectFeature**. Never use combined-mode fallback.

The object-centering C1 change touches both `skirt.rs` and
`skirt/occupied.rs`; production implementation, including its preparatory
split, builds on that change and its ownership/geometry contract. Split
affected production files before semantic edits if needed to keep every
changed file <=400 LOC. Native acceptance **and real per-object skirt output** must be established
before claiming RED; no admission-only or combined-skirt witness suffices.

## Producer boundary

This proposal consumes existing resolved
PrintObject identity and already-transformed hulls. It does not alter group
resolution, raw intersections, object centering, instance transforms or
exclusion labels. Any RED exposing that producer's defect belongs to the
centering slice, not repaired inside the skirt slice.

## Upstream source boundary

Vendored `OrcaSlicer/src/libslic3r`:

- `PrintConfig.cpp:5721+`, `skirt_type`: combined/per-object vocabulary.
- `Print.cpp:2646`, `Print::_make_skirt`: occupied hulls and loop construction.
- `Print.cpp:2784-2864`: `SkirtGroupItem`, object membership, union-find and
  grouped occupied points. PrintInstances of one PrintObject contribute to
  that object's single occupied item. Combined forces all items together.
- `Print.cpp:2865-2900`: iterate expanded group envelopes until collision
  merging stabilizes; emitting groups expand by
  `skirt_distance + skirt_loops * spacing`.
- `Print.cpp:2973-2996`: construct each group skirt from its occupied hull,
  reverse loops, retain membership in `m_skirt_brim_groups`; flattened
  `m_skirt` is a preview/extents compatibility mirror, not emission ownership.
- `GCode.cpp:4456-4484`: `find_skirt_brim_group_idx` and
  `GCode::generate_object_skirt_group`; select owner group and use
  `m_skirt_group_done[group_idx]` to avoid repeated group emission.
- `GCode.cpp:5334`: object-local emission entry in the layer processing path.

These upstream structures/functions, not a new Ares pipeline, own the slice.

## Current Ares gap and exact destination

`crates/ares-core/src/project_slice/gcode_emit/skirt.rs:64-73` rejects
positive-loop per-object skirts whenever there is more than one resolved
object. Single-object per-object skirts already enter the combined-shaped
implementation. `SkirtPlan` retains one flat loop list, one seam angle and
one layer schedule, losing upstream group emission ownership.

`skirt/groups.rs` currently implements zero-loop occupied-hull merging only;
it has no retained object membership or obstacle-only item distinction.
This is a temporary compatibility shell around upstream SkirtBrimGroup,
not a sufficient object-skirt plan. Do not merely delete the admission guard
and send multi-object skirts through a global combined hull.

Rust destination:

- `gcode_emit/skirt.rs` and its `skirt/` children: source-mapped group records,
  member identity, per-group hull and reversed loop construction.
- `gcode_emit/layers/skirt_emit.rs`: adapt emission scheduling so selecting
  the first member of a group emits it once at the correct layer/object
  boundary, matching `generate_object_skirt_group` ownership.
- A new public parity test module under `ares-cli/tests/orca_parity/`, with
  minimal registration. Split edited modules before >400 LOC;
  do not move unrelated motion/centering code.

## Included — smallest useful source slice

First slice: single-filament, by-layer printing, no brim, no support, no
raft, no wipe tower, draft shield disabled, positive skirt loops,
`min_skirt_length=0`, uniform object skirt start angle.

Include real multiple resolved PrintObjects; disjoint groups, overlapping
skirt envelopes, transitive merging and copies belonging to one PrintObject.
Retain stable upstream object/group ordering, object membership, plate-frame
hulls, spacing, reversed loops and once-per-group/layer emission. Preserve
combined mode and existing single-object per-object output bytes.

This scope is not complete per-object skirt support. Before widening the
current guard, audit existing capability admission and preserve explicit
rejection for combinations whose upstream behavior remains deferred; do not
silently emit combined-mode output or add a legacy fallback. If a narrow
source-cited admission boundary cannot be enforced without a broad new
option policy, stop and request a scope decision before implementation.

## Deferred upstream behavior

- `Print.cpp:2805-2817,2822-2825`: brim/support-brim occupied contributions
  and obstacle-only wipe tower participation. No speculative obstacle API.
- `Print.cpp:2902-2970`: group brim ownership and combined-brim connectivity.
- Sequential/by-object emission and shared-skirt collision validation,
  `Print.cpp:3779+`; multiextruder ownership/tool ordering.
- Different per-object skirt start angles, minimum-filament extra loops,
  draft-shield lifetime, raft/support height interactions.
- Object-group centering, source identity and instance transform rewrite;
  consume its contract only. No counterbore, interface-shell, anchor or web
  changes are included.

## RED plan

1. Export repeatable 3MFs with two simple printable prisms, single filament,
   positive skirt loops, by-layer order, all deferred features disabled.
   Far-apart case must keep separate groups; near case must merge; three
   objects arranged in a collision chain exercise iterative transitivity.
   Add repeated-instance and nonprintable-earlier-source-index variants.
2. First run official and seeded Native against each identical input. Require
   producer success and actual skirt extrusion; record exact input hashes,
   G-code, group/topology witness and producer/wrapper metadata. Do not count
   schema acceptance or a no-skirt file as path reachability.
3. Public Ares API and CLI must slice the same bytes. Commit tests **before**
   production edits: baseline should fail with the existing
   `UnsupportedProjectFeature("skirt_type per-object")`, with Native success
   independently retained. A producer-coordinate failure is classified as
   a dependency blocker, never normalized away.
4. Retain byte-identical Native strict full-output comparisons for GREEN,
   plus API/CLI byte equality and artifact witnesses. Existing combined and
   single-object cases are unchanged controls. Add zero-loop control to
   protect existing occupied-hull/extent behavior. No assertions relaxed,
   ignored tests, UnsupportedProjectFeature workaround or post-hoc unit tests.

Success: source-owned groups, unchanged controls, all admitted scoped inputs
strictly matching both reference modes. New failures in deferred combinations
must remain explicit limitations, not inflated parity claims. Gate with
strict/fmt/WASM and workspace checks.

## Remaining dependency checks / stop conditions

The scope is the initial by-layer/no-obstacle grouping. Retain the existing
guard for all listed deferred conditions; do not add new Unsupported errors.
Consume the object-centering C1 contract only. If exact full-output skirt
ordering still depends on centering C2, report that dependency rather than
touching resolution/centering files.
