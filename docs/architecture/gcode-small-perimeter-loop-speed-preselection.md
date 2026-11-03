# TiQ2: rewrite the `GCode::extrude_loop` small-perimeter speed boundary

Status: diagnosis/spec for coordinator review only. No approved RED/GREEN or
production implementation. Local diagnostic probes have been removed, not
committed. Implementation waits for worker2 integration and refreshed main.

## Owning upstream boundary and architectural decisions

Upstream revision: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `OrcaSlicer/src/libslic3r/GCode.cpp:5808–5813`,
  `GCode::extrude_loop`: small-perimeter speed is a **loop caller's**
  preselection, only when incoming speed is the default sentinel `-1` and
  complete `loop.length()` is at most `SMALL_PERIMETER_LENGTH(threshold)`.
  Percentage speed uses `outer_wall_speed`; zero uses half outer-wall speed.
- `GCode.cpp:5896–5900`: pass the selected small-perimeter speed to eligible
  nonbridge perimeter paths **only when the resolved selection is positive**;
  adjacent path roles and nonpositive selections retain their incoming speed.
- `GCode.cpp:6038–6065,6087–6099,6144`: `extrude_entity` distinguishes
  `ExtrusionPath`, `ExtrusionMultiPath`, and `ExtrusionLoop`. Multipath calls
  `_extrude` directly with incoming speed, **without** loop preselection.
- `GCode.cpp:6515ff,6657–6714`: default role speed, volumetric limits and
  dynamic overhang estimation are later stages, not substitutes for entity
  classification. Reference speed and original speed remain distinct.
- `GCode/ExtrusionProcessor.hpp:315–474`,
  `ExtrusionQualityEstimator::estimate_extrusion_quality`: calculate overlap
  and curl-dependent segment speeds, then clamp to `original_speed`.

Non-negotiable decisions:

1. Bare paths and multipaths never acquire small-loop speed merely because
   their individual paths are short. Complete loop length, not fragment length
   or post-clip length, decides loop eligibility.
2. Preserve explicit incoming speed, bridge-role exclusions, zero/percentage
   resolution and subsequent first-layer/ramp/cap ordering from upstream.
3. `PathProperties::inside_loop` is not an entity discriminator. It is also
   true for multipaths because curl collection has already occurred. Do not
   fix this by gating on that boolean or changing its curl semantics.
4. This is a Rust rewrite of the cited upstream caller boundary. The existing
   Ares per-path heuristic is a temporary mismatching shell to replace, not
   an independently designed speed pipeline to extend.

## Fresh diagnostic evidence

Evidence root: `${TMP_ROOT}/ares-tiq2-line1265/`.
Local branch `pi/tiq2-line1265-diagnosis` started at worker2's fetched head
`693cc5e93461f31f49a196f76207cc4dc50b9f26`; worker2's worktree was read-only.

Immutable input copied from
`/tmp/tiq2-phase-support-aaa-seeded/case-5YcqzB/input.3mf`:
`0435346ed81fff32acdae7a3a3f2c9e880f4f1efe37d99d3427833836b901942`.

One fresh Ares execution and one fresh seeded Native execution, CPU0/seed0,
explicit executable/resources identities, no cache, no retry; 16-thread
limits. One shared 300-second whole-case deadline with owned-process cleanup.
Both exit0; elapsed2.297595594s; zero TIMEOUT. This is diagnosis, not RED/GREEN.

Bindings/captures:
- `fresh/identity.json`, `fresh/result.json`, per-command JSON and raw channels;
- `fresh/audit.json`, `fresh/resources-identity.json`;
- `diagnostic-probes.diff`, `diagnostic-probes-final.diff`;
- `fresh/ares-stderr.bin`, `cooling-pre-0.gcode`, `cooling-post-0.gcode`;
- `fresh/core.json.gcode`, `fresh/native/plate_1.gcode`;
- `run_fresh.py`, `audit_fresh.cjs`, build/Clippy/fmt/resource-audit evidence.

The historical line1265 symptom is located by WIDTH0.38292 and its first move
`X169.811 Y169.854`, not a hard-coded line number (fresh Ares speed line1264).
Fresh observations:

| Boundary | Observation |
|---|---|
| Rust entity dispatch | `ExtrusionMultiPath`, exactly one member, not a loop |
| Member/complete multipath length | 0.189724169994mm / same (one member) |
| Complete loop length | Not applicable; no loop owns this path |
| Curl state flag | `inside_loop=true`, despite multipath entity |
| Local first point / origin | (4.874803,4.816747) / (165,165) |
| Layer / width | index2 / 0.38292038 |
| Config | outer120; inner200; small30%; threshold5 |
| Ares configured/capped/original | 36 / 36 / 36mm/s |
| Ares overhang result | `processed=None`: no retained variable-speed branch |
| Ares cooling input/output | F2160 / F2160; `slowed=false` |
| Cooling time / target | 6.521622s / 3s |
| Native final target feedrates | F2700 then F4560 (45 then76mm/s) |

Root cause: `motion/features.rs:165–170` applies a loop-only preselection to
this multipath's individual `source_length`. `motion/path.rs` supplies that
length. The incorrect original36 then caps overhang speeds and prevents the
variable-speed emission branch. Upstream multipath dispatch has no such
preselection. Neither resonance (disabled) nor cooling explains the36.

Evidence limits: Native RTTI/original-speed/pre-cooling internal state was
not instrumented. Native final76 and upstream estimator clamp plus
CoolingBuffer's slowdown-only behavior imply original speed at least76;
this bound is source-derived, **not** a captured Native original/pre-cooling
value. Ares pre/post cooling36 is directly captured. Earlier speculation
about a150mm/s percentage base was disproved: inner speed is200 and upstream
percentage explicitly uses outer120.

## Why the upstream entity is also a multipath

The frozen Native footer selects `wall_generator=arachne` (not classic) and
`seam_gap=10%`. Native's owning generation boundary is
`PerimeterGenerator.cpp::traverse_extrusions`, not GCode TYPE text:

- `PerimeterGenerator.cpp:370ff` consumes ordered Arachne `ExtrusionLine`s;
  `:2353–2363` carries each line and its contour classification into that
  ordered list, and `:2471` calls `traverse_extrusions`.
- `:528–547` branches on `extrusion->is_closed`: closed lines become
  `ExtrusionLoop` with a closed endpoint assertion and loop orientation.
- `:548–566` is the open-line branch. It constructs `ExtrusionMultiPath`,
  starts with the first path, groups connected subsequent paths, and appends
  a multipath even when there is **only one member**. A short length does not
  convert an open line into a loop.
- The matching rewrite is `perimeters/arachne/traverse.rs:149–226` and
  `append_open_multi_path`: `is_closed` selects `Loop`; false selects the
  multipath append. The fresh entity probe identifies this exact latter
  branch's one-member result, at the target width/start point/layer.
- Native `GCode.cpp:6087–6099` dispatches this generated multipath through
  `extrude_multi_path`; `:6038–6065` passes its original incoming speed
  unchanged to `_extrude`. `:6516ff` resolves default external-perimeter
  speed there. Neither function performs small-perimeter selection.

Thus the upstream representation of this Arachne open-line slice is also
`ExtrusionMultiPath`, not a tiny closed `ExtrusionLoop`. This is a
**source-boundary argument**, corroborated by the Rust entity probe, matching
Native output geometry and Native final45/76; it is not a claim that Native
RTTI or its Arachne `is_closed` flag was directly inspected at runtime.
If later independent evidence shows the upstream line topology differs,
retain that as a separate geometry/materialization discrepancy; do not infer
entity identity solely from `;TYPE:Outer wall` or rounded G-code endpoints.

## E and complete-byte comparison scope

Native first two E values .00228+.00226 equal Ares .00227+.00227 in exact
1e-5 units. The final short segment differs .00128 versus .00131.
`GCode.cpp:7111–7149` uses quantized positions for variable-speed segment
lengths; its constant branch uses raw scaled segment lengths
(`GCode.cpp:6997ff`). Rust already mirrors those branches in
`motion/path/variable.rs` and `constant.rs`. Incorrect suppression of the
variable branch therefore changes the length basis despite identical printed
XYZ. Do not patch E precision, geometry, flow ratios or formatting to hide it.
Verify the E consequence through the restored branch after approval.

Both retraction/wipe totals are exactly4mm in1e-5 units:
Native `390015+2237+3874+3874=400000`;
Ares `378920+4722+8179+8179=400000`.
Wipe contribution ratio .09985/.2108 is approximately36/76, consistent with
last-feedrate redistribution; not evidence of a separate retract-total bug.

The complete-byte fresh result remains **DIVERGENT**. Excluding only the
generated-by identity line, the first difference is earlier, byte11101,
normalized line439: Native travel X169.4 versus Ares X169.344. Preserve this
raw evidence and leave the travel boundary with worker2. No reference retry,
status promotion, comparator relaxation or automatic nondeterminism exception.
M73 and the entire footer remain authoritative.

## Proposed minimal Rust destination slice

Destination: existing rendering-neutral core emission modules
`project_slice/gcode_emit/motion/{loop_paths,materialized,features,path}.rs`
and their caller context; no new crate or Ares-owned pipeline.

- Replace unconditional per-path small-perimeter selection in `features`.
- At the true loop caller, compute complete loop length before tail clipping;
  resolve the upstream default-speed/threshold decision once and pass the
  resulting optional requested speed to eligible paths. Ordinary path and
  multipath callers retain default role-speed behavior.
- Keep explicit requested speed distinct from default role speed; do not
  overload the shared curl-collection flag or introduce mutable global state.
- Reuse existing zero/percentage config resolution where it already matches
  upstream; include its source behavior in tests rather than inventing a
  fallback. Preserve later layer/ramp/volumetric/overhang/cooling semantics.
- Keep touched modules at most400LOC; WASM/Windows/macOS/Linux core boundary
  must remain free of probe filesystem/terminal behavior.

### Affected caller inventory

| Existing caller | Required context/behavior |
|---|---|
| `motion/perimeter.rs` bare `ExtrusionEntity::Path` → `materialized::emit` | No small-loop override; default role speed even for short paths |
| `motion/perimeter/multi_path.rs` → `materialized::emit` | No small-loop override for any member; preserve whole-entity curl collection and `inside_loop=true` |
| `motion/loop_paths.rs` ordinary loop → `materialized::emit` | Only owning `ExtrusionLoop` may select small speed using sum of complete unclipped member lengths; all eligible fragments receive the same selection |
| `loop_paths.rs` scarf loop branch → `materialized::emit` | Preserve existing explicit slope-speed ordering; carry new caller context without repurposing slope/curl flags; no independent scarf rewrite |
| `motion/materialized.rs` → `path::emit` → `features::kinematics` | Transport the caller's requested/default speed; remove the local short-path heuristic |
| `motion/support.rs` direct `PathProperties` construction | No new loop override; support role speed stays unchanged |
| `motion/skirt_brim.rs` skirt/brim constructions | Keep their explicit/default role speeds and clipping; no Inner/Outer small-perimeter rule |
| Feature/path/scarf fixtures constructing `PathProperties` | Specify actual caller context, not an inferred loop flag; no post-implementation shallow test additions |

The currently incorrect heuristic reaches both Inner and Outer wall roles,
including short standalone paths, multipath members, and short fragments of
otherwise long true loops. Support/skirt/brim do not currently enter its
role predicate, but any context-field plumbing must leave their bytes
unchanged. Do not alter entity production, Arachne openness, curl collection,
line439 travel, loop ordering, geometry or E calculation to fix this slice.

Included: loop-only applicability, whole-loop length, default versus explicit
speed, eligible perimeter roles, zero/percentage resolution and unchanged
later speed stages. Deferred: worker2 phase/travel work, unrelated geometry,
seam/curl algorithm changes, independent E numerical patches, CLI exit
families, matrix promotion, other unproven overhang discrepancies.

## RED/GREEN and exit criteria after approval

Wait for worker2 integration; fetch/rebase onto latest main (coordinator has
announced circle-volume main3b6e0ebdd). Write RED before implementation.

1. Lock the unchanged TiQ2 input and fresh producer/input/output receipt
   identities. Demonstrate multipath wrongly chooses36 and the target
   complete-output discrepancy, retaining any earlier travel divergence.
2. Source-cited E2E controls: short bare path/multipath; true small loop;
   long multipath loop with a short fragment; explicit speed; bridge-role
   exclusion; threshold endpoints; zero and percentage speed. No new shallow
   unit tests after implementation, fixture tuning or ignored assertions.
3. Implement only the approved caller-boundary rewrite. Verify actual Native
   target45/76 and E/wipe output via full-byte evidence, not speed-only PASS.
   Keep any independent discrepancy open rather than widening the fix.
4. Fresh same-round seeded/official gates on the rebased main with explicit
   identities/caches/CPU/seed, runs1 and deadlines; compare exact failure names
   and unchanged streams. Any travel nondeterminism ruling requires separate
   coordinator approval and exact evidence, not count equality.
5. Workspace nextest, fmt, strict Clippy and Tier1/WASM gates; no lint waiver.
   Exact-hash review before any nonforce FF push and independent confirmation.

No RED/GREEN, implementation commit or push is authorized by this diagnosis
alone. Review this spec first; diagnostic probes remain external/local-only.
