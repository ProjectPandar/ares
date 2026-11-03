# SPEC RR: complete gated role-flow block

Status: **REVISED SPEC-REVIEW-REQUEST; supersedes rejected narrow spec
fffecfa38. No public RED or production implementation yet.** Base: merged
main `445648072103587a9d60d85f1c4fb4425056ae18` (stack-27, FULL27
1636/1636 both modes). Branch: `worker1/wall-flow-discovery-spec`.
Original pre-rebase spec is preserved at `worker1/wall-flow-spec-pre-stack27`.
Coordinator owns integration; rebase only on announced advances and refresh
exact-hash evidence before RR. Interlocking remains shelved/W6; junction
deviation remains queued. Neither belongs to this slice.

## Owning upstream boundary: one block, not two independent features

Pinned source: `8500fcdccaa10b5099ac20d252af3a7c560046f1`. All upstream
paths below are under `OrcaSlicer/src/libslic3r/`.

The port unit is **the entire `GCode.cpp:6485-6507` block in
`GCode::_extrude`**: eight mutually exclusive role branches plus the subsequent
first-layer rule, all inside the **same** `set_other_flow_ratios` gate.
`PrintConfig.hpp::PrintObjectConfig` owns gate/support ratios;
`PrintRegionConfig` owns the remaining ratios. Existing typed parsing is
not the missing port. Definitions/defaults are in `PrintConfig.cpp:1387-1482`:
gate=false, each multiplier=1.

Normative control flow, translated from those exact lines:

```text
if set_other_flow_ratios {
    if role == erExternalPerimeter         { effective *= outer_wall_flow_ratio }
    else if role == erPerimeter            { effective *= inner_wall_flow_ratio }
    else if role == erOverhangPerimeter    { effective *= overhang_flow_ratio }
    else if role == erInternalInfill       { effective *= sparse_infill_flow_ratio }
    else if role == erSolidInfill          { effective *= internal_solid_infill_flow_ratio }
    else if role == erGapFill              { effective *= gap_fill_flow_ratio }
    else if role == erSupportMaterial      { effective *= support_flow_ratio }
    else if role == erSupportMaterialInterface { effective *= support_interface_flow_ratio }
    if on_first_layer && role != erBrim && role != erSkirt {
        effective *= first_layer_flow_ratio
    }
}
```

Roles not matched in the eight-way chain get no role multiplier, **but still
get the first-layer multiplier** when eligible. Do not broaden support to
`erSupportTransition` (the upstream comment asks a question, not permission),
or treat overhang as outer/inner. Preserve brim/skirt exceptions. The first
layer predicate is `GCode.hpp:670`: non-null current layer, id=0, bottom Z
within EPSILON, not merely the first emitted object layer.

`GCode.cpp:6469-6471` first forms geometric flow * print flow, then filament
flow. Existing earlier role/scarf adjustments at `:6473-6483` precede this
block. The block's role multiply precedes its first-layer multiply. E conversion
at `:6512-6513`, speed/volumetric consumers at `:6554+`, cooling annotations,
and writer state must see the same effective flow. A final E-word-only patch,
reordered arithmetic, or double application is not a faithful port.

## Existing Ares first-layer audit and exact Rust destination

The audit did **not** find a global ungated first-layer application in the
public project-slicing path. Current `project_slice/group_fills/params/
projection.rs:235-253::role_flow_ratio` puts sparse, internal-solid and
first-layer multipliers inside `context.object.set_other_flow_ratios.0`.
First-layer there is checked using planned id=0. `fill_entities/flow.rs:15`
and `concentric.rs:135` pre-bake that product into fill flow. This is a
partial, geometry-side shell around the upstream emission concept, not the
complete block. `motion/materialized.rs:50` and `motion/support.rs:116,210`
forward nominal path/support flow without the missing role/first-layer rules.
Thus the observed first-layer gap is **missing wall/support/etc. coverage and
split ownership**, not evidence to assert an ungated bug that was not found.

Legacy `extrusions/options/accessors.rs:154-170` has no gate argument, but
`options/flow_ratios.rs:208-229` installs non-unit role/first-layer options
only with gate=true. It is not this project_slice consumer. Do not switch
to that pipeline or invent a fallback to reuse it.

Port the complete block into the **existing**
`project_slice/gcode_emit/motion/` lifecycle, preserving these boundaries:

1. `options.rs::MotionOptions`, `options/build/fields/assemble.rs`, and
   `options/build/region.rs::apply` resolve the owning object's gate/support
   ratios and current region's remaining ratios. Reuse resolved-object
   defaults and the existing region refresh, matching config apply before
   entities (`GCode.cpp:6136,6163`). No first-region snapshot or duplicate
   independent config path.
2. Existing `features.rs::PathProperties`, materialized/fill/support/skirt/
   brim handoffs and `path.rs` feed the owning motion consumer. Select the
   exact role from existing role vocabulary, apply **all eight branches**
   and the gated first-layer rule at one effective-flow boundary shared by
   E, volumetric speed and cooling metadata. Reuse the existing
   `EmitState::on_first_layer` lifecycle; do not introduce another producer.
3. `extrusion.rs::for_length`, `path/constant.rs`, `path/variable.rs` and
   existing writer paths must receive that same flow. Keep nominal geometry,
   widths, point order, fitting, seam clipping and support generation intact.
   Do not modify estimators to hide mismatched flow-derived bytes.
4. Retire the **gated-block contribution only** from the earlier fill
   pre-baking when centralizing it, or otherwise prove there is precisely one
   application with upstream arithmetic order. Preserve pre-existing top/
   bottom/internal-bridge flow behavior outside this block. Do not stack new
   emission multipliers on the old sparse/solid/first-layer pre-baked values.
   Any grouping/parameter changes needed to preserve ownership must be
   source-cited and reviewed before implementation; no silent coalescing drift.

These Ares structs are temporary compatibility shells around `_extrude` and
writer concepts, not a new Ares flow engine. No new algorithms, parameters,
clamps, guards, flags, parser relaxation or fallback. Existing files near the
limit (`path.rs` 386, projection.rs 383 LOC) must stay below 400; mechanically
split first if necessary. **Do not touch ironing.rs (399 LOC)** without its
separately verified required split.

## Completed whole-block Native discovery

Discovery root: `${TMP_ROOT}/ares-w1-gated-flow-discovery-v2-fffecfa38/`.
Exact-final-spec rerun root: `${TMP_ROOT}/ares-w1-gated-flow-spec-reviewed-scope/`.
Only the latter's completed `spec-review-audit.json` at the exact clean new
commit is authoritative for renewed RR; earlier hashes are discovery, not
new-hash gates. Both retain scripts, manifest, raw outputs, launch receipts,
actual Native ELF/all-thread affinity, repeat and activity audits.

**Every one of the eight roles has an achievable simple Native-active
witness**, including overhang. No unwitnessed role is proposed for porting.
Three closed, connected, oppositely edge-paired genus-0 meshes, positive signed
volume, are built from the existing input-only 3MF shell: 20x20x4 mm box;
1.3x20x4 mm thin prism for gap fill; 30x10x4 mm T-shaped overhang with support.
`geometry-manifest.json` records vertices, triangles, edge/Euler/volume/bounds.
Each pair differs only in project settings; all non-config ZIP members match.
Classic, 0.2 mm layers, two walls, no scarf/arc/spiral/ironing/tower; explicit
20% rectilinear sparse infill. Print/first-layer flow=1 except named first
legs; retained filament flow=0.98. Support uses the existing normal-auto
configuration. First-layer cases have **positive skirt and brim** to test
both exceptions, not just their absence.

| Case / ratio | Unit control | Native-positive target-role moves, all changed |
|---|---|---:|
| outer 0.75 | box-unit | 80 Outer wall |
| inner 0.65 | box-unit | 80 Inner wall |
| sparse 0.70 | box-unit | 396 Sparse infill |
| internal solid 0.60 | box-unit | 258 Internal solid infill |
| gap fill 0.70 | gap-unit | 20 Gap infill |
| support 0.70 | support-unit | 507 Support |
| support interface 0.65 | support-unit | 312 Support interface |
| overhang 0.70 | support-unit | 12 Overhang wall |
| first-layer 0.55, gate on | first-unit | 33 first-layer moves: 4 outer + 4 inner + 25 sparse |
| first-layer 0.55, gate off | first-unit | 0 changes; complete pre-config body identical |
| all ratios non-unit, gate off | box-unit | 0 changes; complete pre-config body identical |

The four unit controls plus eleven target/control legs make **15 inputs**.
The eight role targets change all corresponding positive E moves and **no
other positive role moves**; ordered XYZ, roles and layers are unchanged.
First-layer gate-on changes every eligible first-layer move, no later moves;
33 positive skirt and 101 positive brim moves stay unchanged. Both gate-off
legs retain the non-unit settings in config and keep the complete body equal
to their units. Genuine feedrate/M73/statistics changes are retained.

Completed discovery: **60/60 successful Native slices** = 15 inputs x
2 modes x 2 independent processes, fresh serial roots/caches, canonical env,
seeded CPU18, actual Native ELF and all sampled descendant threads CPU18.
Complete repeats/cross-mode outputs match after only generator identity/
timestamp normalization, retaining config/M73/statistics/order. Discovery
paired-body/positive comparisons prove activation; they never substitute
for public same-input whole-stream parity.

The initial 0%-sparse attempt is preserved as **SPARSE-INACTIVE NON-GATE** at
`${TMP_ROOT}/ares-w1-gated-flow-discovery-fffecfa38/`; it was a fixture fault, not
proof of equivalence. Corrected v2 explicitly activates sparse infill. All
archive/member/geometry hashes and per-key retained settings are audited.
No Ares API/CLI or public RED has been run for this revised slice yet.

## Proposed public E2E RED, only after SPEC approval

Freeze the exact 15 v2 input-only archives and complete manifests into the
existing parity layout; never commit Native outputs. For **each** role leg,
first prove the Native activity above, then require public byte-in API and
**real CLI success**, raw API=CLI, and unchanged
`golden::compare_ordered_bytes_generator_only` against Native for the **same
input**. Include first-layer gate on/off, all-ratios gate-off and all four
unit controls. Raw API rejection or CLI failure must fail the success
assertion, never be recast as equivalent rejection.

Record genuine mismatch/error/exit per leg in both official and seeded
modes. Sparse/internal-solid already have partial consumers, so they may
PASS; report that honestly as existing behavior/regression coverage, not a
fabricated RED. First-layer gate-off may also PASS. **Do not implement a
behavioral change for any role lacking an authentic public RED**; if a
Native-active leg already passes, preserve it byte-for-byte through any
separately justified mechanical consolidation. If existing implementation
fully covers the block, STOP NO-RED rather than manufacture a failure.

If an authentic RED is blocked by a fixture/geometry/support preparation
fault outside this block, preserve the fault as NON-GATE, cite the owning
source and obtain a revised approved boundary. No port of an unwitnessed
role, no suppression or silently shortened two-role implementation. Any
future proposed additional role requires its own Native-first RED evidence.

After independent exact-clean-committed RED approval only: implement the
whole block in the owning lifecycle. GREEN requires all 15 same-input full
comparisons in both modes, raw API=CLI, retained control and already-passing
role bytes, unchanged geometry, no duplicate sparse/solid/first-layer factors,
and actual API flow activity consistent with Native. Preserve complete
Native repeats and real process/thread affinity receipts.

CPU18, jobs2/nextest threads2, targeted runs only; serial official/seeded,
fresh caches/artifacts, required Native and all canonical references. Exact
hash fmt, strict core+CLI all-target clippy and WASM checks accompany runtime
RED/GREEN review. No full/workspace/push/merge. No comparator changes, body/
config/M73 filtering, tolerances, sorting, estimator repair, or paired input
substitution. Coordinator owns integration.

## Included / deferred and approval boundary

Included: all eight role branches, first-layer inside the same gate, exact
brim/skirt exceptions, current object/region ownership, arithmetic order,
and existing flow-derived motion consumers. The full block is the port unit;
the earlier two-role scope is explicitly rejected.

Deferred: unrelated earlier flow/scarf/top/bottom/bridge behavior changes,
new geometry/support algorithms, additional multifilament/raft/tower/
Arachne/scarf/arc/spiral composition witnesses, and non-unit print-flow
composition coverage. Deferred coverage is **not** an exemption from the
shared role/gate semantics. No role algorithm should be redesigned to make
this slice pass. Unanticipated ownership changes require an amendment.

Request independent revised SPEC review; wait for owner approval before
public RED, and independent RED approval before production GREEN.
