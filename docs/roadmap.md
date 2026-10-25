# Ares roadmap

## Future milestone gate
Every future milestone must be a source-cited Rust rewrite slice of `OrcaSlicer/src/libslic3r` or `OrcaSlicer/src/libvgcode`. Specs and plans must name the upstream files/classes/functions being ported, define the Rust crate/module boundary, list included and deferred upstream behavior, and avoid adding Ares-owned pipeline abstractions except as temporary compatibility shells around named upstream concepts. Milestones that add slicing, G-code, configuration, or viewer-data behavior must start from the owning upstream boundary rather than from a new Ares pipeline design. See `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

This applies to every later milestone: the default design target is an upstream `libslic3r` or `libvgcode` rewrite slice, not an Ares-authored pipeline. Any milestone that cannot point to the upstream source boundary and the Rust destination boundary must be rejected or rewritten before work begins.

## Active program: OrcaSlicer v2.4.2 3MF project G-code parity

### By-object spiral per-object transition_out: approved SPEC → side-branch tests-only RED → bounded GREEN

Owner upstream8500fcdcc `SpiralVase.cpp:120–121,154–165,205–216` (the
last layer of EVERY object batch emits a flat taper pass cloned from
the ramp lines, E × finishing factor; `transition_out = last_layer &&
relative_e`) with `GCode.cpp:3805–3806` (single-object pipeline:
`last_layer` is per-object-batch under by-object printing;
`:3698–3705` degenerates to plate-final under by-layer spiral via the
`Print.cpp:1334–1341` admission). Destination: `ares-core
project_slice/gcode_emit/layers/object_entry.rs` — `final_layer`
drops the `is_last_entry` conjunct, becoming per-object
(`layer_index+1 == object_layer_counts[object_index]`); the spec's
consumer table proves `is_last_entry`/`last_entry` plumbing
(`schedule.rs:285–287,329`, `layers.rs:134`, `tool_entry.rs:40,73,287`,
`drive.rs:315`, `object_entry.rs:36,125`) has no other consumer and is
removed. RED on the side branch `spiral-byobject-transition` from the
W1 tip `a1a72362a` (not merged into the frozen window tip; append is a
COORD decision after the RR): witness = the committed
`two-object-by-object-apart.3mf` (A1 by-object spiral; first
difference line 5 — `; total layer number: 83` vs `82`; Native prints
`overhang.obj`'s top-layer flat taper pass, Ares drops it plus the
end-of-object sequence and travel Z; filament totals follow). Complete
unweakened comparator, official + seeded runtime Native, `case_support`
registration. Deferred: M73 P-progress placement (4 sites), final-layer
ramp feed `F3713` vs `F1857` (overhang slow-down classification).
[SPEC](architecture/spiral-byobject-transition-spec.md),
[ARD](architecture/ard-spiral-byobject-transition.md).

### Spiral vase object-start Z-only line: approved SPEC → tests-only RED → bounded GREEN

Owner upstream8500fcdcc `SpiralVase.cpp:131–141` (every Z-only G1 of a
spiral-enabled layer is kept and rewritten to the ramp base z with the
3-decimal `GCodeLine::set` splice, `GCodeReader.cpp:323–357`; base z =
first-Z − Σ dist_Z, `SpiralVase.cpp:86–121`) plus its producer:
`GCode.cpp:5708` (`change_layer` arms `m_need_change_layer_lift_z` every
layer) and `GCode.cpp:6367–6385` (`_extrude`'s guard fires on the flag
alone — upstream's comment names the equal-first-point case — and emits a
forced `travel_to_z(m_nominal_z, comment, true)` bypassing `will_move_z`,
`GCodeWriter.cpp:811–929`; zero-length `travel_to` only flushes the M624
labels). Destination: remove the `already_emitted` dedup in
`ares-core gcode_spiral_vase/project.rs::append_layer_z` (literal
rewrite-all port); `start_travel.rs:148–157` only if RED bytes prove the
writer-side branch also wrong, aligned to `GCode.cpp:5708/6367–6385`
semantics. RED on the W1 chain at `2a20c7dff`: witness
`single-object-by-layer` (A1 spiral; comparator first difference byte
81880 — 47 Native-only bare `G1 Z<z>` lines after M624 markers) plus the
`z_hop=0` variant (byte 81702), complete unweakened comparator, official
+ seeded runtime Native; native preflight passes on RED. GREEN gate: all
`spiral_*` parity cases stay green in both modes — regressions are
root-caused, the dedup is never restored. Deferred: by-object-apart layer
number 83-vs-82 class, smooth-XY extras.
[SPEC](architecture/spiral-vase-object-start-z-spec.md),
[ARD](architecture/ard-spiral-vase-object-start-z.md).

### Spiral vase multi-copy by-layer admission: approved SPEC → tests-only RED → bounded GREEN

Owner upstream8500fcdcc `Print.cpp:1334–1341` (`Print::validate`: with
`spiral_mode`, `total_copies_count = Σ instances > 1` and a non-by-object
sequence rejects with the `spiral vase mode` diagnostic; CLI
`CLI_VALIDATE_ERROR` −51 → 205 with the validate `result.json` fields).
Destination: only `ares-core print/validate.rs::validate_print` gains the
instance-count check on the existing `SliceError::Validate` path; typed reads,
no new exit plumbing; file stays ≤400 LOC. RED = three cases on the W1 chain
at `a62041e78`: witness `two-object-by-layer` (A1, `spiral_mode=1`, by layer;
Native −51/205 + diagnostic, Ares currently exit 0) plus acceptance guards
`single-object-by-layer` and `two-object-by-object-apart` (both accepted by
Native and Ares; byte parity deferred — BBL spiral emission divergences are
recorded findings). Official+seeded, runtime Native, no Native artifacts.
Deferred: `Print.cpp:1342–1353` regions check, spiral emission parity.
[SPEC](architecture/spiral-vase-admission-spec.md),
[ARD](architecture/ard-spiral-vase-admission.md).

### Extruded weight/volume template totals: approved SPEC → formal RED → bounded GREEN

Owner upstream8500fcdcc `GCode.cpp:1675–1690` (per-extruder `w = v ×
filament_density × 0.001`, volume via typed filament_diameter cross-section),
`:1648–1652` placeholder registration, `:4638–4641` before-layer template
render, `PrintConfig.cpp:11188` definition. Destination: `ares-core
project_slice/gcode_emit/motion/state.rs::EmitState::extrusion_totals()`
replaces hard-coded diameter 1.75 / density 1.24 with typed per-extruder
values; consumers `layer_gcode.rs:113-119/218-224` unchanged semantics. RED =
three `native_stream_case::check` cases on the W1 branch chain at `d340f9c84`:
MK4 stock control + targets `filament_density=2.0` (first diff byte 11090,
line 459, M74 W ratio 2.0/1.24) and `filament_diameter=1.85` (byte 11103,
ratio (1.75/1.85)²); official+seeded, runtime Native, no silent pass. M73
transpositions in the density witness are downstream STOP-check effects.
Deferred: vector weight/volume placeholders, interpolate_table engine,
statistics footer, M201 consumer, queued files. CPU18 serial/threads1; no
FULL/rebase/merge/push.
[SPEC](architecture/extruded-weight-totals-spec.md),
[ARD](architecture/ard-extruded-weight-totals.md).

### Unknown-key CONFIG_BLOCK echo: approved SPEC → formal RED → bounded GREEN

Owner upstream8500fcdcc `Config.cpp::set_deserialize_nothrow:576–590` (cleared
keys recorded, never stored), `Config.cpp::set_deserialize_raw:608–629` (no
def/alias → `UnknownOptionException`), `GCode.cpp::append_full_config:5591–5646`
(only defined non-banned non-nil keys reach the CONFIG_BLOCK). Destination is
only the `ares-core options/config_export/writer.rs` raw-echo loop: gate raw
project-settings keys on the existing registry definition table incl.
`OptionMetadata::aliases` (literal alias→canonical port), no hand-written key
lists; `config_export/value.rs` stays with the queued W6 window;
`EXPLICIT_RULES` (handle_legacy port) stays byte-for-byte. RED = two
`native_stream_case::check` cases: P1S ByLayer target with the single unknown
key `first_layer_size_compensation=0.2` (comparator FAIL today: Ares emits
`; first_layer_size_compensation = 0.2`, byte 12675) plus the keyless control;
Native runtime-generated, never committed, no silent pass. GREEN flips the
target to PASS with controls and the complete comparator unchanged.
CPU18 serial/threads1; no FULL/rebase/merge/push.
[SPEC](architecture/unknown-key-config-block-spec.md),
[ARD](architecture/ard-unknown-key-config-block.md).

### Sequential template total: approved SPEC → formal RED → bounded GREEN

Owner upstream8500fcdcc `GCode.cpp::_do_export:2484–2528` mode-dependent
`m_layer_count` → `:2890` placeholder → `:4677–4684` layer template consumer.
Rust boundary: existing private `gcode_emit/orchestration.rs::emit`
`EmitState.total_layer_count` initializer; replace the wrong scalar handoff
with existing `tail_layer_count`, not a new Ares counter/pipeline. Include only
that handoff; defer file-start/role cache, P1S ByObject compound Y/M73 residual,
broader skirt control, geometry, writer, processor, templates and other workers.
Above frozen117f4ffe3 afterW5: docs commit then tests-only Native-first official+
seeded A1 ByObject target (81 vs50) and stock A1/originalP1S ByLayer controls.
Exit RED: one genuine count-only FAIL/two complete PASS each mode, API/CLI
success/raw equality, immutable SHA inputs, unchanged complete comparator and
fmt/strict Clippy. One combined docs+RED REVIEW-REQUEST/HOLD; later GREEN only
the existing field initializer with identical tests. CPU18 serial/threads1;
no FULL/integration/rebase/merge/push or broader runtime qualification.
[SPEC](architecture/sequential-template-layer-count-spec.md),
[ARD](architecture/ard-sequential-template-layer-count.md).

### Sequential origin-hop writer frame: approved SPEC → formal RED → bounded GREEN

Approved source boundary at upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`: `GCode.cpp::travel_to:7480–7506`, `point_to_gcode:8124–8128`, and `set_origin:5659–5669`, reached by sequential `_do_export:3298–3322`. Rust destination is the existing `gcode_emit/layers/transition.rs::emit` hop output and writer XY cache, reusing `state.offset`. Include only this frame conversion; defer generic writer/processor, M73 implementation, broader skirt, and W3/W4/W6 owners. Formal RED target is stock-offset P1S ByObject with unchanged complete comparator including M73; controls are same-scene zero-offset and A1 ByObject. GREEN must pass complete parity; any remaining M73 difference is STOP. Docs+RED use one REVIEW-REQUEST and append coordinator inbox. CPU18 serial/threads1; no FULL/integration/rebase/merge/push.
[SPEC](architecture/sequential-origin-writer-frame-spec.md),
[ARD](architecture/ard-sequential-origin-writer-frame.md).

### Auto-lift query frame: SPEC review → genuine RED review → translation-only GREEN

Upstream `GCode.cpp::needs_retraction:7604` translates the clipped overhang
query by `m_origin - m_writer.get_xy_offset()` into current-plate coordinates.
Destination: existing ares-core `gcode_emit/motion/auto_lift.rs::AutoLift::crosses_overhang`;
correct its nozzle-offset compatibility shell, not an Ares travel pipeline.
Include only that translation and its spiral/progress consequences; defer cache
geometry, detection, nonzero-plate plumbing, writer, processor and W3-owned work.
Use the original P1S two-object ByLayer Native-first public witness. Exit: real
tests-only RED, then unchanged complete-output comparator PASS/raw API==CLI,
Native G17/G3 Z4 spiral and removal of the extra early P28R13 through translation
alone. No separate B timing/M73 implementation. Fresh stack above frozen
sequential label-flush; CPU18 serial/threads1. Each phase requires
REVIEW-REQUEST/HOLD; no FULL/integration.
[SPEC](architecture/auto-lift-query-frame-spec.md),
[ARD](architecture/ard-auto-lift-query-frame.md).

### Sequential origin-travel labels: SPEC → RED → bounded GREEN

Upstream8500fcdcc `GCode.cpp::travel_to`7466–7467, reached by sequential
`_do_export`3321–3327, owns this literal pending-label consumption. Destination:
ares-core `gcode_emit/layers/transition.rs::emit`, reuse existing motion label
writer between travel setup and origin hop. Existing transition is a temporary
upstream compatibility shell, not an Ares pipeline. Include only the missing
consumption/citation; defer generic travel, formatting, geometry, queue/IDs,
priming and tower. Exit: tests-only RED then official+seeded Native-first K1
ByObject2object target and by-layer2object control, complete-output parity and
rawAPI==CLI, unchanged control streams. Base frozen4eeda42be afterW5; no FULL.
[SPEC](architecture/sequential-origin-label-flush-spec.md),
[ARD](architecture/ard-sequential-origin-label-flush.md). RR then HOLD.

### Support-base extension priority: SPEC → RED → bounded GREEN

Upstream8500fcdcc `FillBase.cpp:1834–1948`, specifically the literal conditional
parse at1915, owns this slice. Destination: existing ares-core
`fill/base_support/extend.rs`; replace its incompatible longest-side decision
with valid-previous priority, not a new Ares support pipeline. Include only the
candidate block and affected comments; defer geometry, marking, trimming, costs,
linking and writer behavior. Exit: official+seeded Native-first K1 tree(auto)
overhang, stock and normal(auto) controls, full-output parity and raw API==CLI
after tests-only RED. Stack frozenccc79c314; no FULL; REVIEW-REQUEST then HOLD.
[SPEC](architecture/support-base-extension-priority-spec.md),
[ARD](architecture/ard-support-base-extension-priority.md).

### Spiral hole projected split: SPEC → RED → bounded GREEN

Pinned upstream8500fcdcc `GCode.cpp::extrude_loop:5754–5775` keeps the hole
exclusion only on orientation, not configured-spiral projected splitting.
Destination: existing ares-core `gcode_emit/motion/perimeter.rs` dispatch and
unchanged nearest-projection helper; the extra role gate is an incompatible
caller shell, not an Ares pipeline feature. Include hole loops/bottom layers;
defer geometry, SpiralVase arithmetic, seam ranking, travel/wipe and processor.
Exit: Native-first official/seeded MK4 spiral-hole, stock-hole and existing CW
control full-output equality plus raw API==CLI after tests-only RED. Stack on
frozen83e2e4f09; no FULL; review/hold for single W1 window after W5.
[SPEC](architecture/spiral-hole-projected-split-spec.md),
[ARD](architecture/ard-spiral-hole-projected-split.md).

### Grid per-surface collections: SPEC → tests-only RED

Priority: preserve upstream `Layer::make_fills` (`Fill.cpp:1326-1357`) →
`Fill::fill_surface_extrusion` (`FillBase.cpp:161-182`) per-expolygon collection
identity on pinned8500fcdcc. Rust destination: existing private
`fill_entities/grid.rs:46-65/120`; replace the grouped-polylines-to-one-collection
compatibility shell, not Grid geometry or GCode island assignment. Live traces
prove identical ordered surfaces/full vertices but Native collections[1,1]
versus Rust[2]. Keep flat geometry consumers and path semantics unchanged.
Exit for this assignment: docs SPEC then shared `native_stream_case` tests-only
K1 bridge strict RED + MK4 bridge passing control, frozen input SHA, Native
double replay before API/actual CLI, raw API==CLI, complete generator-only
ordered comparison, official/seeded CPU13 serial threads1, then REVIEW/HOLD.
GREEN/FULL require separate approval. Defer grouping, other infill patterns,
multiline algorithm changes, support/raft, travel, seam/rear, brim, scan,
cooling and processor/M73 behavior. No golden output or comparator weakening.
[ARD / SPEC](architecture/ard-grid-surface-collections.md).

### GUI painted fuzzy skin: discovery rejection, SPEC review pending (#126)

Frozen beam92ce9b02c is unchanged; this separate docs slice owns upstream8500fcdcc
`PrintObjectSlice.cpp1033–1141/1227–1241 apply_fuzzy_skin_segmentation`,
`MultiMaterialSegmentation.cpp2211–2226 →1955–2194 segmentation_by_painting`,
`PrintApply.cpp1081–1110` real fuzzy region IDs and classic `FuzzySkin.cpp561–684`
masked consumer. Destination: ares-core project XML/mesh carrier, existing real
region graph, private region_slices/load_stages and classic fuzzy materialization;
staged marker registries/global-fuzzy helper are incomplete compatibility shells.
Eight Native first (both modes/replays) on exact stock cube single-face Ripple
pair: full generator-only replay stable,1/4raw pairs identical and3timestamp-only;
then two CLI: control matches both Native, painted rejects canonical XML attribute.
No public RED/API qualification claimed; observer initial-tool stop preserved.
Priority/exit: independently approve SPEC → success-required Native/API/CLI RED
→ RED RR and separate GREEN instruction → complete real consumed chain. Keep
MMU→fuzzy→single interlocking→compensation, stock EFC; no admission-only patch,
new registry/backend/API or comparator weakening. Combined annotations/beam,
Arachne-painted/nonzero XY/broader runtime remain deferred. CPU18/jobs1; files≤400.
No production/tests/FULL/rebase/merge/push here; beam integration window separate.
[SPEC](architecture/painted-fuzzy-skin-spec.md),
[ARD](architecture/ard-painted-fuzzy-skin.md).

### W5 MM flush_into_infill object-label envelope: SPEC+RED / HOLD

Discovery on main `78d642702` (worker W5; evidence dir
`/home/indexyz/ares-coord/evidence/w6-mm-toolchange-discovery/`, name
historical): stock P1S AMS two-filament witness, single knob
`flush_into_infill` 0→1. Stable dual-mode residual, zero motion bytes:
84 extra `; stop printing object, unique label id`/M625 pairs plus 2
stale layer-end stop ids (Native 142/142 envelope balance vs Ares 142/205).
Upstream pin8500fcdcc boundary: `GCode.cpp:5482-5493` — an unflushed
`set_object_start_str` is dropped and the M625 end label is NOT armed;
Ares `object.rs::queue_stop` BBL branch arms unconditionally and
`motion/state/object_labels.rs::queue_object_stop_label` lacks the
`queue_exclude_end` drop guard. Fix lands in those two files only;
`ToolOrdering.cpp:1593,1672-1690,1785` wipe selection is context (motion
already byte-identical) and deferred. Tests-only RED
`test(mm_flush_into_infill_labels)`: target FAIL / base control PASS
per mode, Native required, inputs committed, no Native output committed.
Exit after approval: same selector green in official+seeded, fresh caches,
CPU11/threads1; any other residual including M73 → STOP.
[ARD/SPEC](architecture/ard-mm-flush-into-infill-labels.md).

### W5 P1S ByLayer brim: area-boundary SPEC revision / HOLD

Frozen rib3 `a96cf4d03` → docs `f4c714372` → approved RED `61b15c6a8`.
Read-only entry traces locate the eight missing notch vertices before
materialization (Native contour60 vs Ares52, identical hole/second area).
Corrected upstream pin8500fcdcc `Brim.cpp::outer_inner_brim_area`:
569–576 collects translated gap-expanded islands into print-wide no-brim
geometry; 662–679 subtracts that complete collection from each owner's brim.
Rust destination: `project_slice/gcode_emit/brim.rs` collection/finalization,
`brim/owner.rs` translation and `brim/bed_area.rs::clip_plate` consumer.
`brims/outer.rs` remains the local producer; paths.rs is not the repair target.
Existing per-owner orchestration is a temporary upstream compatibility shell,
not an Ares pipeline design. Priority: approve revised SPEC before GREEN.
Unchanged-code baseline `test(brim) | test(p1s_bylayer_brim_)`: 150 selected,
149 PASS + target FAIL per official/seeded mode, CPU11/threads1/fresh caches.
Exit after approval: same complete-stream target/control/neighbors dual run;
any residual including M73 → STOP, no mask or scope expansion. Defer new area
heuristics, shared geometry, registration/emission and timing changes.
Docs-only revision → REVIEW-REQUEST/HOLD; no GREEN/FULL/integration yet.
[ARD/SPEC](architecture/ard-p1s-bylayer-brim-contour.md).


### Painted MMU → beam call order: discovery complete, SPEC RR pending

Legacy #70–#73 carrier/segmentation goals are superseded by the merged complete
MMU and side-cut chains. Next bounded upstream slice: pinned8500fcdcc
`PrintObjectSlice.cpp:1207–1243` MMU/fuzzy/interlocking order, preserving
`MultiMaterialSegmentation.cpp:2169–2207` beam suppression of side cutting.
Rust boundary: existing private load_stages post-painted/pre-compensation handoff
and region_slices interlocking consumer; current early placement and painted+beam
rejection are temporary compatibility shells, not an Ares pipeline design.
Priority/exit: independently approved SPEC → public success-required RED with
beam-off/active-unpainted controls → RED RR before any GREEN; final strict dual
FULL only in a separately assigned window. Discovery has16 Native/4CLI calls,
not public acceptance. Fuzzy painting/combined annotations remain deferred.
CPU18/jobs1/test-threads1; no new registry/backend/API or comparator relaxation.
[SPEC](architecture/painted-beam-call-order-spec.md),
[ARD](architecture/ard-painted-beam-call-order.md).

### Chamber writer-used extruder reduction: docs-first SPEC → test-only RED

Coordinator accepted discovery on frozen exhaust53890b780. New branch
`worker1/chamber-used-extruder` stacks docs then tests only on that commit.
Upstream8500fcdcc `GCode.cpp:3008–3013,3037,3130–3133,3498–3499` and
`GCodeWriter.cpp:192–212` own activation OR, independent maximum, placeholder,
startup/shutdown bytes. Destination: existing private machine/temperature.rs;
its object scan is an incomplete upstream shell. Reuse existing writer
`ExtruderRegistry::ids()` handoff, never a new registry or object-scan fallback.
Exit: four immutable inputs/SHA, two genuine full-stream REDs and two controls
per official/seeded mode, global Native-first preflight, strict Native/API/CLI
and raw API==CLI evidence on CPU18; send RED RR and stop for GREEN authorization.
get_at deferred with discovery evidence retained, no slice. Exhaust and chamber
rebase together only in the authorized integration window. No production edits,
FULL, merge/push or broader runtime qualification here.
[SPEC](architecture/chamber-used-extruder-spec.md),
[ARD](architecture/ard-chamber-used-extruder.md).

### Exhaust-fan writer-used extruder reduction: SPEC → public RED (#102/#103)

COORD accepted readonly discovery #101 on main6f6ef10f5. New branch commits
docs-first SPEC/ARD, then four frozen input3MF+SHA public NativeCase tests only.
Owner upstream8500fcdcc `GCode.cpp3181–3196/3501–3516` reduces over
`m_writer.extruders()`; Rust destination is existing private
`gcode_emit/machine/exhaust_fan.rs` at current start/end call locations.
The current all-config-vector scan is temporary incomplete upstream scaffold,
not an Ares fan pipeline. Preserve writer ID provenance, gates, zero speed,
get_at/max/PWM arithmetic, flavor, ordering and configuration serialization.
Do not create a shadow registry or change Print/ToolOrdering behavior.

Priority/exit: docs commit → T0-only/high-unusedT1 target plus equal,
unused-disabled and machine-capability-off controls → global Native-first,
official/seeded directed CPU18 public success-required complete ordered Native /
API / actual CLI and raw API==CLI → RED RR. Expected target FAIL/three controls
PASS per mode; no Native constants, movement goldens or expected-error inversion.
No GREEN before separate approval; defer nonzero/multiple/support/custom-tool
runtime witnesses, adjacent fans/temperature/registry work, other workers'
skirt/z-hop/print_order/M73/seam-occlusion scopes, FULL, merge/push and platform
runtime qualification. Prior painted stage text below remains historical.
[SPEC](architecture/exhaust-used-extruder-spec.md),
[ARD](architecture/ard-exhaust-used-extruder.md).

### BBL Type1 wipe tower: milestone approved, T1–T2 SPEC review

Milestone docs approved/committed as `8a22fa596`. W5's
[T1–T2 consumed-chain SPEC](architecture/bbl-type1-t1-t2-consumed-chain-spec.md)
now inventories the accepted X1C baseline: rib wall, single-nozzle two-slot
same-category planning, and separate Type1 activation versus Type2 priming-order
flags. Config-derived reachability is distinguished from observed output and
untraced dynamic branches. SPEC review is required before any public RED.

Priority: restore stock X1C AMS tower output before more literal-placeholder
edge cases. Accepted two-object Basic/Matte discovery has Native success in
both modes but no Ares tower; stock G29 bounds are a downstream symptom.
Owner W5; frozen placeholder integration stack55a81063b stays unchanged.
No code/RED/FULL/rebase authority is implied by this proposal.

Source pin8500fcdcc: `Print.hpp:1086` BBL effective Type1 →
`Print.cpp:3290-3301/3390-3571` activation/planning →
`GCode/WipeTower.cpp::generate_new`, block planning, tool_change_new and
finish_layer_new → `GCode.cpp::WipeTowerIntegration::append_tcr` →
`Print.cpp:3020-3052/3154-3164` generated hull → `GCode.cpp:2926-2939` bounds.
Four dependent slices: T1 identity/activation, T2 Type1 planning/material blocks,
T3 consumed writer/toolchange/finish integration, T4 geometry/live consumers.
Rust boundary is private `ares-core::project_slice::gcode_emit::wipe_tower`
Type1 modules plus existing ordering/runtime/footprint consumers. Reuse
[ARD-0105](architecture/ard-0105-smooth-type2-timelapse-port.md) shared seams,
not Type2 algorithms; existing shells are temporary upstream adapters.

Each slice requires a GUI-reachable stock X1C Native-success/public whole-stream
RED: baseline; prime-volume variation; smooth-timelapse/unequal heights;
moved/rotated tower respectively. Only baseline is currently measured.
SPEC→frozen RED→separate GREEN approval; slices are checkpoints, not independently
shippable activation. Exit requires complete consumed-chain Native/API/CLI parity
in official+seeded, raw API==CLI, Type2 regressions, fmt/strict Clippy/WASM and
coordinator-scheduled exact-tip FULL. No isolated G29 fix, guards, omissions or
Type2 fallback. Preview-only rendering/general multi-transform preparation defer;
all activated live Type1 consumers need qualification, not assumed X1C coverage.
[Milestone ARD](architecture/ard-bbl-type1-wipe-tower-port.md).

### Painted side-layer cutting: approved SPEC, public RED only (#99)

New branch from frozen1045b1775; original painted branch/worktree remain unchanged.
COORD authorizes docs-only SPEC/ARD/roadmap, then Native-first public RED only;
no GREEN, production change, FULL, merge or push. Source boundary at8500fcdcc:
`MultiMaterialSegmentation.cpp1144–1168 cut_segmented_layers`,2169–2181 gate/order,
2197–2207 wrapper; actual consumer`PrintObjectSlice874–886/1206–1224`.
Rust destination is private`ares-core/project_slice/region_slices/painted/cut.rs`,
consumed before existing caps/merge/actual apply. Existing omission/guard is only
an upstream compatibility shell. Preserve casts, object-layer parity and source
width0/depth>width behavior, not tooltip ignore rules or unused width-depth math.
Priority precedes painted+beam's separate generator; beam/fuzzy painting defer.
RED exit: three discovery-exact target3MFs(width1/depth0,1/1.5,0/0.5), one unchanged
existing painted-split control, full ordered NativeCase/API/CLI assertions and
raw API==CLI; dual-mode CPU18 expected3target FAIL/1control PASS with fresh global
Native-first barrier, static/target gates, retained artifacts and independent
RED-RR. Later GREEN/integration/FULL need separate authority; #90 still waits its
explicit W1 integration turn. [SPEC](architecture/painted-side-cut-spec.md),
[ARD](architecture/ard-painted-side-cut.md).

### Painted MMU complete consumed chain: SPEC request (#90)

Base main3e862584d; upstream8500fcdcc. COORD accepted the seeded-only discovery
of the real PrintObjectSlice1224→apply877→MMS2207→segmentation2127/2153→
build_graph1555 caller chain, not phase/public GREEN. Default VD repair is now
integrated; painted projection/graph/actual apply still are not. No RED yet on
this new SPEC hash; old f894/0ce6 runs remain historical.

Priority/order: independently approve this docs-only SPEC → public NativeCase
whole-stream RED via native_stream_case::check → independent RED approval/separate
GREEN instruction → ONE complete consumed-chain production integration unit.
Retain source-ordered phase discovery/design RRs as document checkpoints, not
additional private-phase prerequisites for public RED. Projection491–588/
2029–2120 → painted_lines/color614–956 → MMU graph59–479/1527–1822 →
top/bottom1181–1472 → merge1825–1870 → real PrintObjectSlice apply874–1030/
1207–1224 must reach the post-conical/pre-compensation handoff142–151 for simple
and complex objects. Rust destination is private region_slices/painted/* with
actual shared registry and separate MMU carrier/ordinary versus strict exporters;
PostRegion/RegionGraph remain upstream compatibility shells, not a new pipeline.
MUST port Point.hpp506–659 radius multimap, never overwriting PointHashMap.

Exit: frozen leaves/split plus unpainted controls, Native-first with positive
model extrusion for both painted materials, complete ordered public API/actual
CLI parity/raw API==CLI in official+seeded. Current paint_color rejection is legal
success-required RED, not expected-error PASS. No private-entry phase tests;
read-only phase GDB is GREEN diagnosis only. Branch stage commits each cargo
check; no allow/expect/cfg parking, fake consumer or dormant public entry.
All final fmt/strict workspace-all-target Clippy/wasm32/dual-mode canonical FULL
bind the exact complete integration tip and assigned window. W2 owns FULL now;
this request authorizes only docs, not RED/GREEN/FULL/push/merge. Cut/width-depth,
painted+beam/fuzzy stay deferred; preserve support/interlocking and fail-closed
compositions. [SPEC](architecture/painted-projection-graph-spec.md),
[ARD](architecture/ard-painted-projection-graph.md).

### Default Voronoi detection / rotation repair: prerequisite integrated

The approved five-commit default-repair stack is integrated in main3e862584d.
Its historical painted complete-unit SPECf894/public RED0ce6 remain unmerged;
#90 has resumed discovery/SPEC under the complete-chain conditions above, not
implementation. The actual frozen painted-split Native MMU calls use
REPAIR_SUCCESSFUL/effective local storage; this is distinct from the earlier
incident-anchor prerequisite's no-repair rectangle.

Source owner: `Geometry/Voronoi.cpp::construct_voronoi28-73`, copy90-158,
detect166-240, ordered rotation repair248-362 and source-cited cell-range/
filtered-planarity/point helpers. Rust target is private
`ares-core/src/geometry/voronoi/*`, OUTSIDE vendor, with the REAL existing
`geometry/medial_axis/diagram.rs::build` default-constructor consumer
(`Geometry/MedialAxis.cpp:458-460`). No raw-Boost substitute or dormant facade.
Adjacent closing retry and painted implementation defer. Arachne's actual
`SkeletalTrapezoidation.cpp:378` also uses default repair; that consumer/migration
is explicitly deferred, not qualified by raw-Boost parity. GREEN needs targeted
MedialAxis/consumer regressions plus final coordinator-scheduled FULL to prove
expected constructor behavior changes do not regress consumers.

Completed prerequisite order/exit: approve docs SPEC/ARD -> fresh real reachable repaired Native-first
constructor RED -> wrapper plus real consumer -> independently qualified exact
GREEN, complete ordered effective arrays/states and consumer regressions,
fmt/strict/WASM. No tolerance, repair suppression, FFI or backend fallback.
Durable measured evidence is hash-verified under `~/ares-evidence/worker1/`;
old hashes are not new-base credit. No FULL/push/merge now; coordinator schedules
broad validation. [SPEC](architecture/voronoi-default-repair-prerequisite-spec.md),
[ARD](architecture/voronoi-default-repair-prerequisite-ard.md).

### Conditional scarf eligibility: SPEC request (2026-10-17)

Approved proposala30db87 selects libslic3r `GCode.cpp::GCode::extrude_loop5777-5785`
→ `ExtrusionEntity.cpp::ExtrusionLoop::is_smooth369-424` into existing Rust
`gcode_emit/motion/scarf.rs::build`/private helper and config/callsite context.
Adjacent positive-overhang guard5787-5791 is IN SCOPE with source-owned
`SeamPlacer.cpp1083-1092` raw unsupported-distance producer and1550-1563 chosen
candidate handoff through existing layer plans/ordered loops/nearest context.
No zero-only support, invented threshold/fallback, old penalty/embedding rewrite
or new Ares pipeline. Existing scarf construction remains unchanged; painting,
new random/nearest placement, geometry/travel/M73 and processor behavior defer.

Exit: approved exact SPEC, fixed-budget Native low/high nonzero-dependency
qualification, frozen five-case Native-ACTIVE/INACTIVE public RED with both
API/CLI complete strict comparisons in both modes, then separately authorized
bounded GREEN all wholePASS plus targeted regressions/fmt/strict clippy/WASM.
Qualification or unrelated geometry prerequisite failure means STOP, not wider
port/fixture search. RED only integrates with GREEN; full/workspace remains
coordinator-only. [SPEC](architecture/scarf-conditional-spec.md),
[ARD](architecture/ard-scarf-conditional.md).

### spRear/back candidate comparison and alignment: SPEC request (2026-10-17)

W4's coordinator-approved proposal ports `GCode/SeamPlacer.cpp` spRear
comparison Y priority777-779, similarity839-841, pick917-928, rear fit
length1364-1366 and alignment routing1486-1492 (pin8500fcdcc). Rust boundary is
existing `project_slice/seam_placement` runtime/preparation/alignment; replace
its Back bypass, not the unrelated basic-perimeter max-Y helper. No max-Y
shortcut or new Ares pipeline; paint/scarf/new random/nearest and clocks defer.
Base **approved physical-plane GREEN4dbba4fc09b4b84033797ab83f2941e5d8484b95**,
keeping shared files on the same stack25 ancestry. Discovery on4351: admitted
6-vertex/8-triangle four-layer prism, aligned control Native/CLI whole-PASS;
only back positive Native0/CLI0 whole-FAIL byte451/line18, first motion raw64
Native110,113.446 versus Ares110.43,112.509. Not an admission/defensive guard.
Docs only: SPEC approval→frozen Native-first public API+real CLI control/positive
RED in official+seeded→RED REVIEW→separate bounded GREEN REVIEW. Exit: whole
PASS/raw API==CLI both modes, Native first-motion parity, existing seam-family
regressions including audited Native gates, fmt/strict scoped clippy/WASM;
residual whole failure means STOP. No code/test/fixture permission before gates;
CPU15 targeted only, coordinator owns full/workspace/integration; RED merges
only with GREEN. [SPEC](architecture/seam-rear-spec.md),
[ARD](architecture/ard-seam-rear.md).

### Per-object skirt guard inventory: approved ByObject slice, integration pending (2026-10-17)

W1 Slice 1 GREEN `c1780265dd00dfd9322ffc6bba734d335c5311fe` ports
`GCode.cpp:3321-3322` / `GCodeWriter.hpp:266-267` / `GCodeWriter.cpp:678`
transition descriptions. Slice 2 GREEN
`ee270b46a3ebf5d66f8f6838b06b6557b1c0ce00` admits separated ByObject
per-object skirt groups, reusing `Print.cpp::_make_skirt:2780-2977` and
`GCode.cpp:4465-4483,5334` per-group emission. Both are owner-approved and
queued together, **not yet merged**. Slice 2 official/seeded targeted28/28 each,
fmt/strict/WASM pass; shared-group source-branch reachability remains unproven.
[SPEC](architecture/per-object-skirt-by-object-spec.md),
[Slice 2 boundary](architecture/per-object-skirt-by-object-slice2-spec.md).

Remaining guards in W1's approved
`crates/ares-core/src/project_slice/gcode_emit/skirt/generate.rs::SkirtPlan::generate`
are recorded below; W2/W5 concurrently own related edits, so their integration
status is not inferred from this branch. Coordinator resolves stack conflicts.

- Early multi-object/perobject guard: enabled prime tower (W5); more than one
  filament-diameter entry / multi-filament (W5); any object enabling support or
  raft layers (W2); installed raft schedule (W2); nonempty tower corners (W5).
- Brim guard remains: an installed brim plan with combine_brims enabled, a
  non-OuterOnly brim type, or a print-object group with other than one instance.
  Existing singleton OuterOnly/no-combine admission is not broadened here.
- New bounded post-group guard remains: ByObject + perobject with multiple
  object members in a computed generation group returns the existing
  UnsupportedProjectFeature. `Print.cpp:2551-2556` throws SlicingError for
  has_shared_per_object_skirt; W1 does not invent that diagnostic/error variant
  without a reachable witness. The distance35 probe rejected earlier at
  sequential clearance, so it does not certify this shared-skirt branch.

Only the blanket early sequence disjunct was removed. Native ByObject clearance,
height/order constraints (`Print.cpp:613,1286-1303`) remain unchanged. No blanket
reset of skirt histories between objects, no new grouping/sequence pipeline,
and no fallback. Future guard removals require their own source-owned SPEC,
Native-first public RED and independent GREEN approval; no full/workspace run
is implied by this docs-only inventory.

### Native-undefined support-ironing patterns (2026-10-17)

`support_ironing_pattern` values Lightning, AdaptiveCubic, and SupportCubic
remain an explicit Native-undefined boundary in 2.4.2, rather than deferred
coverage. `SupportCommon.cpp:1879-1885` constructs the ironing filler without
initializing `FillLightning::Filler::generator` (`FillLightning.hpp:25`; the
generator is assigned only by `Fill.cpp:1260,1456`), while the Lightning path
dereferences it at `FillLightning.cpp:17`. The adaptive paths likewise depend
on `adapt_fill_octree`, assigned only at `Fill.cpp:1246,1451`. Ares therefore
keeps the existing guard and does not claim a RED or port for these three
patterns. Ordinary ironing remains the two-value enum from
`PrintConfig.cpp:6565-6575`.

### Physical seam candidate plane: resource-control SPEC request (2026-10-16)

W4 localized unchanged d951 resource-control motion995/line1198 to third-layer
region1 inner hole-wall seam selection. All24 oriented pre-seam loops and67
candidate XY/angles agree; Native slice_z0.62 versus Rust0.32 flips candidate8/10.
The upstream owner is `GCode/SeamPlacer.cpp::gather_seam_candidates:1014-1040`,
consuming `PrintObjectSlice.cpp::compute_slice_z/new_layers:24-75`'s physical
`Layer::slice_z`. Rust destination is only the runtime region-height scan and
`seam_placement.rs::apply_objects` caller, replaced by the retained
`PostClassicTraversalPrintObject::physical_layers` view. No new plane/pipeline.
Minimal-delta modifier density50→20 control whole-PASSes official Native/API/CLI;
positive whole strict FAIL is retained. M73 is a total-time-denominator symptom,
not a postprocessor fix; old v2 NO-RED remains closed. SPEC approval→frozen public
RED review→separate GREEN permission. Exit: both whole streams/raw API==CLI in
both modes, scoped regressions/fmt/clippy/WASM, no scoring/geometry/M73 changes;
residual owner means STOP. main41f2 now; rebase only after stack24 MAIN-ADVANCE.
FULL24 FREEZE: CPU15 targeted only; coordinator owns full/workspace/integration.
[SPEC](architecture/seam-physical-plane-resource-spec.md),
[ARD](architecture/ard-seam-physical-plane-resource.md).

### 3MF vertex coordinate defaults: amended shared importer SPEC (2026-10-15)

Next W4 docs-only candidate: both
`_BBS_3MF_Importer::_handle_start_vertex:3772-3782` and
`ObjectImporter::_handle_object_start_vertex:5485-5495` use zero for missing
x/y/z before their existing unit factors. Rust destination is the single
`project/model_xml.rs::Vertex` decoder through `project/load/graph.rs`; no
role-specific DTO partition. Triangle `v1/v2/v3` zero defaults at main `:3814`
and resource `:5527` are explicitly deferred. Preserve explicit coordinates,
unit scaling, mesh finite/index checks, XML security, and both roles. SPEC review
and input approval precede any Native/API/CLI/test/fixture work. Public RED must
assert target Native equality and fail until GREEN; controls assert equality.
[SPEC](architecture/3mf-resource-vertex-defaults-spec.md),
[ARD](architecture/ard-3mf-resource-vertex-defaults.md).

### Modifier feature filament IDs: six Native-qualified public REDs (2026-10-16)

Worker4's next rewrite slice is `PrintObject.cpp:3592-3705` explicit feature-ID
mask and modifier base-extruder precedence, `PrintApply.cpp:1032-1055` parent
region construction (painting override order at810-830/1060-1075 stays separate),
and `PrintRegion.cpp:13-15/81-86` sparse filament routing/used-tool collection.
Rust destinations: the six-key modifier guard in effective_config/candidates.rs,
existing region_graph::add_modifier and options/region_options/merge resolver;
no new pipeline. Native discovery on CPU15 accepted both frozen input members
in both modes: positive only adds sparse_infill_filament_id=2 to an existing
modifier with absent extruder, using identical complete two-filament settings.
Positive/control diverge; positive780 actual T1 sparse segments inside modifier
bounds vs controlzero. Observer TYPE/FEATURE dialect fix was offline only;
four primary Native slices, no retry/input substitution. SPEC a2ffbb77b approved
with per-role guard retirement: only a role with its own reviewed public RED
may lose its guard. Before extension, fresh public control whole strict PASS /
raw API==CLI was established in both modes. Five additional single-key positives
now qualify in Native: local T1 counts405 internal-solid/43 top/63 bottom/200
outer/196 inner in each mode, versus controlzero; sparse780 retained. All six
enter independent target-success/full-equality REDs, inputs+SHA only committed,
Native generated fresh at runtime. No unqualified role in the frozen matrix;
future qualification failure retains its guard. GREEN needs separate approval;
no other guards, M73 or downstream repairs. Nonzero modifier-extruder/material/
painting/overlap runtime matrices stay deferred. Exit: both modes
whole strict Native/API/realCLI equality, raw API==CLI, unchanged control and
approved targeted regression/fmt/clippy/WASM. Coordinator owns full/merge.
[SPEC](architecture/modifier-feature-filament-spec.md),
[ARD](architecture/ard-modifier-feature-filament.md).

### Shared triangle vertex-index defaults: proposed source slice (2026-10-16)

Worker4 ports the previously deferred v1/v2/v3 omission contract:
`Format/bbs_3mf.cpp` main `_handle_start_triangle:3813-3819` and resource
`ObjectImporter::_handle_object_start_triangle:5526-5532` both read absent
indices as zero using `bbs_get_attribute_value_int:508-514`. Destination is
shared `project/model_xml.rs::Triangle` reached by both `project/load/graph.rs`
roles; current fields are required u32. No separate role DTO or mesh repair port.
Active Native `_generate_volumes_new` bounds-checks then constructs TriangleMesh,
not an automatic STL repair; first proposed witness removes already-zero v1
from `(0,1,2)` in unchanged controladb38, avoiding new degeneration. No fixture
or runtime yet. Priority: SPEC/input approval, bounded CPU15 Native discovery,
separately approved public target-success/equality RED with strict-PASS control,
then authorized minimal shared defaults GREEN. Exit: both modes whole strict
Native/API/realCLI equality/raw API==CLI, unchanged control and targeted 3MF/load
regressions plus fmt/clippy/WASM. Nonzero-index omissions/degenerate repair,
malformed numeric conversion, extra role/index fixtures and M73 are deferred.
[SPEC](architecture/3mf-triangle-index-defaults-spec.md),
[ARD](architecture/ard-3mf-triangle-index-defaults.md).

### Resource build/item non-consumption: merged, refreshed at main41f2 (2026-10-16)

`Format/bbs_3mf.cpp::ObjectImporter` resource start/end handlers5612-5687 do
not dispatch build/item, unlike main `_handle_start_item:3893-3909`. This port
is already present in main41f2: `project/load/graph.rs` decodes resource payload
as `ModelDocument<Option<IgnoredAny>>`; `project/model_xml.rs` converts it to
empty internal build items. Required main Build and instance/security behavior
remain unchanged. Merged GREEN86b7f3e1c implements original approved GREEN06d31;
existing frozen public witness/control now assert full strict Native/API/CLI
and raw API==CLI equality. The repeated assignment is docs-only refresh and
owner adjudication, not permission to fabricate another RED or undo parity.
Refreshed SPEC records exact current pair, included/deferred scope, failure
inventory, historical pre-GREEN design and planned oracle seam for separately
approved verification. No new Native/test/production work; M73 stays deferred.
[SPEC](architecture/3mf-resource-build-item-ignore-spec.md),
[ARD](architecture/ard-3mf-resource-build-item-ignore.md).

### Referenced 3MF resource-part build omission: approved GREEN, stack-17 (2026-10-15)

Worker4 ports `Format/bbs_3mf.cpp::ObjectImporter` resource build-omission semantics
(active workers1780-1817, dispatch5612-5687, end_model5323-5327) into private
`project/model_xml.rs`/`project/load/graph.rs` role-aware decoding under
`load_project_with_preparation`. Main required Build remains unchanged. Approved
public RED `a88578afa946db02e4e6d68420730b8dae71902f` always executes fresh
Native→public; old e1/ad442 rejected commits are not its ancestor chain. Exact
controladb38ee8/positive0eab5d4d differ only by resource empty-build omission.
Owner approved GREEN `87ac91eb5243d70138681e12a5c2b20602cc92bc` for stack-17;
RED may merge only with GREEN. No GREEN batch rerun after API/socket failures.
The independent [M73 discovery](architecture/discovery-m73-resource-control-order.md)
is archived/deferred, no work. Present nonempty resource build/item behavior is
separate proposed work above, not implicitly included in this omission GREEN.
[SPEC](architecture/3mf-resource-part-build-spec.md),
[ARD](architecture/ard-3mf-resource-part-build.md).

### Arachne fuzzy-skin junctions: proposed SPEC (2026-10-13)

Worker3 next slice: `PerimeterGenerator.cpp::process_arachne:2095,2125` and
`traverse_extrusions:378–435`; `Feature/FuzzySkin/FuzzySkin.cpp:223–291,345–558,685–814`
own junction noise/modes, compatible-region effects and source-ordered splitting.
Rust destination is private `project_slice/perimeters/arachne/fuzzy_skin` plus
existing context/config/traverse seams; replace two fuzzy guards after public RED.
Doc only; proposed stochastic oracle boundary and Native closure/width preflight
require SPEC review. Alternate6a7d842bc is code-approved but not merge-authorized;
full/workspace HOLD. [SPEC](architecture/arachne-fuzzy-skin-spec.md),
[proposed ARD](architecture/ard-arachne-fuzzy-skin.md).

### Arachne alternate extra wall: bounded GREEN, review pending (2026-10-13)

Worker3 ports `PerimeterGenerator.cpp::process_arachne:2131–2144,2163–2175`
per-surface alternate count, integer sparse-density predicate and ordering before
single-wall overrides into `project_slice/perimeters/arachne/{config,walls}.rs`.
Replace only its alternate guard; existing generator/regeneration consumers remain.
Fuzzy skin and overhang each await separate SPEC/RED/REVIEW.
SPEC c3d238a8a approved; Native official/seeded each2PASS/8 guard-rejection FAIL,
40/40 Native admissions, 24 public off/inert control comparisons PASS.
[RED evidence](architecture/arachne-alternate-extra-wall-red-evidence.md);
approved obsolete negative unit retirement76b8fbe43 is separate from GREEN.
Finite official/seeded10/10 paired tests PASS each; all48 raw off/inert control
streams unchanged. [GREEN evidence](architecture/arachne-alternate-extra-wall-green-evidence.md).
Broad oracle-dependent gates await RESTORED; workspace/full await allocation.
[SPEC and public test-first gates](architecture/arachne-alternate-extra-wall-spec.md),
[proposed ARD](architecture/ard-arachne-alternate-extra-wall.md).

### OuterOnly contour and hole exclusion: bounded GREEN, review pending (2026-10-12)

Next worker3 candidate: `Brim.cpp::outer_inner_brim_area:448-570,661-711`
explicit OuterOnly contour-only Round area, default-Miter3 no-inner hole mask
and contact retention; `make_brim:903-904` owns spacing as no_brim_offset.
Replace the bounded whole-ExPolygon project scaffold with private `brims/outer.rs`
and existing BrimPlan dispatch, reusing shared clipping/contact/path emission.
One object/instance/volume/group/tool; auto/EFC/support/painting, nested islands,
cross-object policy and the public rectangular adapter remain deferred.
SPEC a68680255 approved; admitted fixtures/tests [RED9c589662e](architecture/outer-only-brim-red-evidence.md)
committed/reported before production (both modes39PASS/8 new geometry FAIL).
Finite GREEN47/47 both modes; all138 existing/inactive raw Ares streams unchanged.
[GREEN evidence](architecture/outer-only-brim-green-evidence.md) records clipping
Native226 retained and prior52 detached paths removed. Directed/engineering/code
review remain separate; workspace/full wait explicit allocation.
[SPEC and test-first gates](architecture/outer-only-brim-hole-mask-spec.md),
[proposed ARD](architecture/ard-outer-only-brim-hole-mask-boundary.md).

### G4 timelapse-template current-filament context: bounded GREEN, gates pending (2026-10-12)

Port `GCode::process_layer` direct/tracked timelapse invocation context from `GCode.cpp:4672-4674,5167`: implicit vectors use the current logical writer filament.
Rust boundary is `gcode_emit/timelapse.rs`: shared render context plus approved `GCode.cpp:5455-5460` once-only traditional insertion check/set using the existing physical-layer flag. Preserve validation/write-back and position/last-Z bookkeeping.
Six finite Native/API/CLI controls must witness direct/traditional phases and real M104/tool consumption. BBL/tower/safe-position and other template owners remain deferred.
SPEC20444a910 and PLAN extension1f79cf081 approved; admitted RED2ceaa08bf tests/assertions retained. Finite six cases PASS both modes with full Native/API/CLI equality, direct/traditional100 blocks over50 layers. Whole option_coverage and engineering gates pending; heavy gates require allocation. No overlap with other workers' production claims.
[SPEC and test-first gates](architecture/timelapse-filament-context-spec.md), [proposed ARD](architecture/ard-timelapse-filament-context.md).

### G4 layer-template current-filament context: merged bounded GREEN (2026-10-12)

Port `GCode::process_layer` before/after layer-template invocation context from `GCode.cpp:4639-4641,4682-4684`: unindexed legacy vectors use the current logical writer filament.
Rust boundary is existing `gcode_emit/layer_gcode.rs`; reuse the parser and writer registry, preserve hook timing/write-back and explicit indexes.
Six finite Native/API/CLI controls distinguish nonzero/current/upcoming tools through actual M104 and wall-tool consumption. Other template owners and printer/option completeness remain deferred.
Merged `afca7400a` after approved RED/repair: six finite controls PASS both modes. Accepted workspace actualc7@d7518361/8361 and candidate full actual55f6@361988/988 in both modes; these are carried candidate results, not a new full baseline.
Final afca directed seeded214/214, official213PASS/1 coordinator-accepted Native adaptivePA variant (fresh3PASS, five identical Ares SHA samples), strict/fmt/wasm PASS. Raw failure evidence remains recorded.
[SPEC and test-first gates](architecture/layer-template-filament-context-spec.md), [proposed architecture decision](architecture/ard-layer-template-filament-context.md).

### G4 first-layer sequence: bounded GREEN (2026-10-12)

Witness upstream `ToolOrdering::apply_first_layer_order` with two actual outer-wall tools, not only wall/infill sharing.
Five finite one-based configured-vector controls cover size gating, reversal and first-occurrence duplicate handling.
Merged `36ed482eb`: machine-start implicit vector rendering follows the resolved initial filament (`GCode.cpp:3118`); first-layer ordering itself is retained.
Full evidence is the actual `4650357d2`-based candidate: seeded974/974 PASS, official973 PASS/1 coordinator-adjudicated known Native variant, fresh diagnostic3/3 PASS; not a new clean full baseline.
Other-layer count is already merged; SAFC Native nondeterminism is owned by worker6.
[Sources, Rust destinations, failure inventory and test-first exits](architecture/option-g4-first-layer-sequence-coverage.md).

### OuterAndInner combined brim: merged bounded source slice (2026-10-12)

Next worker3 source slice: `Brim.cpp::outer_inner_brim_area` combined selector,
contour Round outer area + original-width Miter3 hole area, shared masks/contact
and existing `tryExPolygonOffset`/`makeBrimInfillImpl` paths. One object/instance/
volume/group/tool; EFC, supports, painting, cross-object composition and the
rectangular public adapter remain deferred. SPEC53b22ddc9 approved; ten input-only
fixtures/Native-admitted REDb35dc2bd5 committed/reported before implementation.
Finite GREEN25/25 both modes; all nine InnerOnly API/CLI outputs byte-identical
across both modes (36 streams). Merged `5d1a77062` after exact-hash approval.
Workspace8718380/8380 accepted; strict/fmt/WASM green. Directed seeded232/232,
official231 PASS/1 Native M73 displacement accepted after fresh3 and unchanged
Ares SHA versus main/history. Full executed871@oldbaseafca: seeded1007/1007 PASS,
official1005 PASS/2 Native113.388 variants, each fresh3 PASS with identical Ares
SHA; full evidence accepted and carried through catalog-only rebase. Raw FAILs
remain retained; old-base full is not registered as a new clean baseline.
[SPEC](architecture/outer-inner-brim-rewrite-spec.md),
[runtime ARD](architecture/ard-outer-inner-brim-runtime-boundary.md).

### Inner-only hole brim: merged bounded source slice (2026-10-12)

Next worker3 slice of the open active-dependency/complete-output priority:
`Brim.cpp::outer_inner_brim_area` InnerOnly hole offsets/masks/contact rules
and existing `tryExPolygonOffset`/`makeBrimInfillImpl` → private core inner
area producer and real project BrimPlan dispatch. One object/group/tool;
public rectangular adapter, OuterAndInner, painting and EFC remain deferred.
Exit: admitted Native-backed RED before code, both-mode finite hole witnesses
and whole option_coverage, unchanged full comparisons, Tier1/engineering gates,
then reviewed canonical candidate full with0FAIL. SPEC696cd308a approved;
test-first RED063a161bc reported before implementation. Finite Native/API/CLI
GREEN15/15 in both modes; merged `10058be78` after Claude's approval.
Workspace8356/8356 and directed222/222 both modes passed; strict/fmt/WASM green.
Full executed38@oldbaseaeb: seeded993/993 PASS; official991 PASS/2 known Native
variants accepted only after each fresh3 PASS and identical Ares SHA. Raw FAIL
evidence is retained; rebase-carry accepted, old-base full not registered anew.
[SPEC](architecture/inner-only-brim-rewrite-spec.md),
[runtime ARD](architecture/ard-inner-only-brim-runtime-boundary.md),
[RED evidence](architecture/inner-only-brim-red-evidence.md).

### Adaptive volumetric fit: coupled-family SPEC request (2026-10-11)

Port witnesses for `GCode::calc_max_volumetric_speed` and `_extrude` current-filament fitted/user cap intersection.
Eight finite enable/model controls must prove actual positive-XYE role/F effects, not empty probes or footer presence.
Existing typed speed algebra is a compatibility scaffold until project-emission reachability is proven.
No Native or production changes before SPEC approval; arbitrary strings and multi-filament contexts remain deferred.
[Source ownership, Rust destinations, failure checklist and test-first gates](architecture/option-adaptive-volumetric-fit-coverage.md).

### SAFC coupled model: source-slice SPEC request (2026-10-11)

Select `small_area_infill_flow_compensation` + `_model`; current model probe is empty-only, not active-curve proof.
Admit upstream SmallAreaInfillFlowCompensator/PCHIP and `_needSAFC` in a finite single-region linear-extrusion context.
Seven paired enable/model/pattern controls require actual solid-role E effects and unchanged whole comparison.
APA, load-settings and owned matrix DIVERGENT work stay excluded; no Native or production edit before approval.
[Source ownership, lawful curves, Rust destination and test-first exit gates](architecture/option-safc-coupled-model-coverage.md).

### G4 other-layer sequence count: bounded GREEN (2026-10-11)

Admit upstream sequence count0/1/2 plus empty/reverse-priority consumers; other G4 domains stay deferred.
Port type-owned numeric/bool vector nil footer semantics, without String/Point suppression or key exceptions.
Fresh workspace8295/8295, strict/fmt/WASM and seeded full923→924 PASS; official base923 PASS.
Official candidate923 PASS/1 FAIL: owner exempts only the observed Native internal-solid path-order outlier.
Native nondeterminism remains a separate OPEN item; no comparator relaxation or automatic full rerun.
[Source scope, gate receipts and exact one-time adjudication](architecture/g4-sequence-review.md).

### G4 Manual-map slice (2026-10-10)

`ToolOrdering` physical-map consumers and `GCode` timelapse/label/M624-M625 lifecycle are rewritten in core.
Only four coherent two-nozzle Manual `filament_map` cases are promoted; other five keys/by-object masks are deferred.
Fresh workspace8265/8265 and strict Clippy/fmt/WASM passed; seeded full893→894 PASS.
Original official failures remain visible; candidate-only Native path-order ND was explicitly owner-adjudicated.
No comparator change or reruns.
[Source boundaries, four-leg receipts, raw inventory and adjudication](architecture/g4-manual-map-review.md).

### Printable-height coverage: bounded GREEN, pending integration review (2026-10-10)

Source-owned BuildVolume zero-height scene sentinel and Print::validate generated-layer ceiling are ported; G2 sequential fixtures are reused.
Seeded/official fresh cold witnesses passed; other 649 plans unchanged. Fresh full BASE626/CAND5572: seeded898→903 and official898→903 PASS, both failure subsets pass.
The concentric Native capture multiplicity disclosure is retained and owner-accepted; no comparator change or rerun.
[Source boundaries, bounded context and exclusions](architecture/option-printable-height-bounded-coverage.md).

### G2 clearance/height coverage: harness GREEN, pending integration review (2026-10-09)

Four bounded coFloat plans: seeded/official cold/hit 12 strict sweep matches; 14 sequential and 8 loaded-rejection witnesses; no production edits.
Actual fcd50205a/1151db4bc full pairs: seeded 882→889 PASS, official 881 PASS/1 FAIL→889 PASS; 705/667 common streams unchanged.
[Source boundary, exclusions and actual execution receipts](architecture/option-g2-clearance-height-coverage.md).

### Stock signal audit: all twelve cases docs only (2026-10-09)

Native nondeterministic crashes rule out runtime-test admission; no tests or production changes remain.
[Audit, retained failures and verification](architecture/2026-10-09-stock-signal-runtime-admission.md).

### TiQ2 complete-output parity (whole-file GREEN, review pending)

The committed `tests/parity/validation/tiq2-type2-enabled.3mf` now produces
complete G-code equal to the runtime seeded OrcaSlicer oracle, with only the
generated-by banner normalized. The final footer rewrite follows
`GCode.cpp:3481–3491` and `PlaceholderParser.cpp:827/915`: SEMM selects the
active filament, non-SEMM visits every configured end template, and implicit
vector indexing uses the invocation filament index while `current_extruder`
remains writer-active. Rust destination: existing `gcode_emit/finish.rs`.
[Source boundary](architecture/tiq2-footer-filament-template-dispatch.md).
Runtime controls cover nonzero SEMM active slot, unused configured slot and
assignment write-back between templates. Fresh default workspace passes
8239/8239; seeded and official paired controls each pass863/863 base and
868/868 candidate, including the strict original whole-file comparison.
Clippy, formatting and core/adapter WASM checks pass. Final merge approval
remains pending; this result does not establish parity for unrelated projects.

### X2D-FM #55 dual-nozzle flush-matrix preparation: closed (2026-10-09)

Landed on main (RED `ab6d7d2e1` → GREEN `e0134bcd7` → review fixes
`f14c02af6`/`0c6e3ccf8` → dependency rule `310f5ae9a` → generated-id
`1c3a64a59`/`cbee83539`). Exit-condition evidence is itemized per
condition — matrices, whole-output equality (seeded 594/594,
byte-stability 500/500), nontrigger preservation, gates and approval
hashes, with stated gaps (workspace/WASM/browser recorded on the
e0134-era candidate, covered by later main gates) — in the
closure record of
[the spec](architecture/x2d-dual-nozzle-flush-matrix-preparation.md).
Scalar UB inputs follow the closed O8 contract.

### O8-X2D-SCALAR: closed as a documented unsupported-input contract (2026-10-09)

Closed by coordinator ruling: the original `case-LqoYic` divergence (Native
success via the out-of-bounds per-nozzle selector read versus Ares's exact
upstream matrix-size error, `GCode.cpp:5596-5614`) is a documented
unsupported-input contract — `UB_ADJACENT`, `parity_pass=false`, not a
parity PASS. The admitted RED and all artifacts are retained; pinned
closure controls live in `runner/x2d_flush_matrix.rs` (the
runtime-constructed original shape asserting the exact CLI
error/exit-1/no-output and a requires-native guard); the legal-size
eight-cell scalar shape is recorded as UB_ADJACENT runtime evidence
with both observed values, not an Ares-side contract.
Full record: [closure spec](architecture/o8-x2d-scalar-flush-context.md).
No UB emulation, numeric patch or relaxed size guard.

### Profile-based slicing entry (2026-10-05, interface approved)

Port PresetBundle loading/inheritance/full_fff_config and Preset compatibility into
`ares-core::profiles`, with byte/JSON WASM adapters and STL filename semantics.
[ARD](architecture/ard-profile-slice-entry.md),
[spec](superpowers/specs/2026-10-05-profile-slice-entry.md),
plan.
Native `--load-settings` complete-output RED precedes production changes. Exit:
full-output native E2E, preserved embedded-3MF behavior, Rust compatibility/import,
web filename callers updated, one real BBL filter timing recorded, four platform/
lint/test gates and full official/seeded controls. No caching or TS config logic.
This precedes multi-transform membership/shared centering; it is not a completed
stock inventory or an Ares-owned pipeline milestone.

### Selective WIP ports: completed boundaries

- Adaptive PA emitter/filter — DONE `56e4da53e`; `AdaptivePAProcessor` / interpolators and `GCode.cpp` marker consumers, within [ARD-0104](architecture/ard-0104-adaptive-pressure-advance-port.md)'s bounded port.
- BBL timelapse forced-Z handoff — DONE `b1a5a84d4`; `GCodeWriter::travel_to_xyz` force-Z state, within [ARD-0105](architecture/ard-0105-smooth-type2-timelapse-port.md).
- Type2 tower planning/emission, PA restore/reset and live geometry consumers — DONE `c53be4fe9`, `ff2a912e0`; `Print::_make_wipe_tower`, `WipeTower2` and `WipeTowerIntegration`, within [ARD-0105](architecture/ard-0105-smooth-type2-timelapse-port.md).
- Initialized bridge candidate traversal and mesh affine arithmetic — DONE `50fe97a6e`; [ARD-0075](architecture/ard-0075-initialized-bridge-candidate-direction.md) / [ARD-0076](architecture/ard-0076-slicing-mesh-affine-rounding.md).
- Type2 `enable_prime_tower=1` verification — GREEN, docs-only review pending;
  [measured gate and exact output](architecture/type2-tower-activation.md).
  Resolves bounded `21c18b79d`: apply-time normalization disables the effective
  tower flag despite two emitted tools / 68 changes; no fix/new test, not active
  tower geometry coverage.

### Selective WIP ports: remaining boundaries (open)

- Adaptive PA deferred upstream behavior — OPEN `2863ef3ea`, worker1;
  [ARD-0104](architecture/ard-0104-adaptive-pressure-advance-port.md) /
  [APA-R2 SPEC-REQUEST](architecture/adaptive-pa-deferred-r2-spec.md): residual
  multipath/sloped suppression, activation/calibration, source-specific resets
  and lookahead; calibration admission and Type1 remain explicit deferred gates.
  Landed bounded ports do not close full Adaptive PA or printer/Option coverage.

### Current acceptance priorities: completed subitems

- Initial ToolOrdering/region-role dual-tool milestone — DONE `76916d01`; bounded initial-tool integration, not global printer/Option acceptance.
- Stock saved-3MF/full-output harness preserving original preset settings — DONE `eb65bca35`; [source boundary and contract](superpowers/specs/2026-10-04-stock-preset-parity.md).
- Full stock execution and outcome classification — DONE `362e4847a`; [latest complete stock audit](architecture/2026-10-09-stock-full-current-audit.md) records 929 PASS and zero DIVERGENT while retaining raw non-PASS outcomes.
- Full Option execution and outcome classification — DONE `d3ecda964`; [latest complete Option audit](architecture/2026-10-09-option-full-current-audit.md) records zero byte divergences without declaring incomplete/failed domains covered.
- Obsolete implementation-source pinning removal — DONE `2439f9d10`; retain schema inventory and public behavior coverage.

### Current acceptance priorities: full acceptance (open)

Range min/max/interior and active dependency coverage still require their complete
source-owned witnesses. The fresh nonempty all-PASS full-inventory exit, unchanged
complete comparators, Tier-1 verification and independent six-axis review remain
OPEN. Completed executions/classifications and individual domain closures are not
that global exit; raw Native errors, crashes and incomplete domains are not PASS.

### Completed rewrite slices

- Prusa terminal wipe segments — DONE `aa52ef2b7`; `GCode.cpp::Wipe::wipe` / `Polyline::append` → `motion/travel/wipe_path.rs`, [ARD](architecture/ard-wipe-zero-length-segments.md).
- re3D common travel boundary — DONE `31996f489`; `AvoidCrossingPerimeters.cpp` end-first common-boundary selection → `router/closest_lines.rs`, [ARD](architecture/ard-re3d-common-travel-boundary.md).
- Classic extra-overhang perimeters — DONE `931691eb4`; `PerimeterGenerator::apply_extra_perimeters` → `perimeters/classic/extra_perimeters` and infill-boundary subtraction, [ARD](architecture/ard-extra-overhang-perimeters.md).
- Qidi Grid collection ordering — DONE `e865eb2a4`; `Fill::no_sort` / `Layer::make_fills` sorting and non-reversal → `fill_entities/grid.rs`, [ARD](architecture/ard-grid-collection-ordering.md).
- Sparse-anchoring centered reference — DONE `7cfa0b33b`; `Layer::generate_sparse_infill_polylines_for_anchoring` / `PrintObject::bounding_box` → `bridge_over_infill/sparse_anchoring`, [ARD](architecture/ard-sparse-anchoring-reference.md).
- Vendor stock fill-reference repair — DONE `94a0a6295`; `PrintObject::bounding_box` / `Fill::_infill_direction` → `fill_entities/generate.rs`, [ARD](architecture/ard-print-object-fill-reference.md).
- Thin-tip support fill connection — DONE `7ac555551`; `FillBase::mark_boundary_segments_overlapping_infill` default collision interval → `fill/base_support/mark.rs`, [ARD](architecture/ard-thin-tip-support-fill.md).
- Filament ironing speed emission — DONE `f10c3b172`; `GCode::_extrude` region/process speed selection → `motion/options/build/fields/assemble.rs`, [bounded analysis](architecture/option-incomplete-coverage.md).
- Support-interface fan requests — DONE `16ac907b9`; `GCode::_extrude` / `CoolingBuffer` START/END → `motion/fan.rs` / `cooling/role_fans.rs`, [spec](architecture/support-interface-fan.md).
- Ironing fan requests — DONE `bf7f17d45`; `GCode::_extrude` / `CoolingBuffer` priority/restoration → `motion/fan.rs` / `cooling/role_fans.rs`, [spec](architecture/ironing-fan.md).

These are completed bounded source slices; adjacent behavior deferred by their
ARD/spec remains separate, without a full printer/Option parity claim.

## Completed milestones

Each milestone is recorded as one line (number, name, DONE); the
full exit criteria, specs and run logs remain in git history.

- M1: Basic slicer framework skeleton — DONE
- M2: STL model import — DONE
- M3: Layer planning and first typed options — DONE
- M4: XY slice segments — DONE
- M5: Contour stitching — DONE
- M6: Hardware option typing — DONE
- M7: Profile fragment inheritance — DONE
- M8: Full profile composition and option groups — DONE
- M9: Slicing pipeline diagnostics — DONE
- M10: Perimeter path generation — DONE
- M11: Infill path generation — DONE
- M12: Ordered print path artifacts — DONE
- M13: Toolpath move emission — DONE
- M14: Extrusion value emission — DONE
- M15: Speed / feedrate emission — DONE
- M16: Skirt path emission — DONE
- M17: Brim path emission — DONE
- M18: Bridge option and role scaffold — DONE
- M19: libslic3r/libvgcode architecture alignment — DONE
- M20: libslic3r crate boundary foundation — DONE
- M21: libslic3r print domain foundation — DONE
- M22: PrintConfig option registry foundation — DONE
- M23: libvgcode rendering-neutral G-code data model port — DONE
- M24: libslic3r GCodeWriter boundary — DONE
- M25: WASM/browser API over rewritten core boundaries — DONE
- M26: PrintConfig common params option registry — DONE
- M27: PrintConfig physical printer option registry — DONE
- M28: PrintConfig FFF travel avoidance option registry — DONE
- M29: PrintConfig bed temperature other-layers option registry — DONE
- M30: PrintConfig bed temperature initial-layer option registry — DONE
- M31: PrintConfig bed type and filament sequence option registry — DONE
- M32: PrintConfig shell and gap-fill option registry — DONE
- M33: PrintConfig overhang fan option registry — DONE
- M34: PrintConfig bridge angle and density option registry — DONE
- M35: PrintConfig solid infill flow ratio option registry — DONE
- M36: PrintConfig other-flow gate option registry — DONE
- M37: PrintConfig wall flow ratio option registry — DONE
- M38: PrintConfig overhang and sparse infill flow ratio option registry — DONE
- M39: PrintConfig internal solid and gap-fill flow ratio option registry — DONE
- M40: PrintConfig support flow ratio option registry — DONE
- M41: PrintConfig one-wall quality option registry — DONE
- M42: PrintConfig overhang reversal option registry — DONE
- M43: PrintConfig overhang speed option registry — DONE
- M44: PrintConfig brim flow and combine option registry — DONE
- M45: PrintConfig brim ear option registry — DONE
- M46: PrintConfig compatible profile option registry — DONE
- M47: PrintConfig print sequence and order option registry — DONE
- M48: PrintConfig cooling and default profile option registry — DONE
- M49: PrintConfig internal bridge option registry — DONE
- M50: PrintConfig G-code and shell pattern option registry — DONE
- M51: PrintConfig wall ordering and small perimeter option registry — DONE
- M52: PrintConfig extruder clearance option registry — DONE
- M53: PrintConfig bed mesh option registry — DONE
- M54: PrintConfig extruder visual and offset option registry — DONE
- M55: PrintConfig filament and print flow ratio option registry — DONE
- M56: PrintConfig pressure advance option registry — DONE
- M57: PrintConfig adaptive pressure advance option registry — DONE
- M58: PrintConfig cooling slowdown option registry — DONE
- M59: PrintConfig fan cooling and filament color note option registry — DONE
- M60: PrintConfig filament mapping and hardware flag option registry — DONE
- M61: PrintConfig filament flush and toolchange timing option registry — DONE
- M62: PrintConfig bed temperature and flush dataset option registry — DONE
- M63: PrintConfig pellet flow and shrinkage option registry — DONE
- M64: PrintConfig filament load/unload speed option registry — DONE
- M65: PrintConfig filament cooling and stamping option registry — DONE
- M66: PrintConfig filament tower purge and interface option registry — DONE
- M67: PrintConfig filament tower temperature and final cooling option registry — DONE
- M68: PrintConfig filament ramming option registry — DONE
- M70: PrintConfig filament material and support option registry — DONE
- M71: PrintConfig filament identity and statistics option registry — DONE
- M72: PrintConfig infill direction and extra solid option registry — DONE
- M73: PrintConfig gyroid optimization and sparse infill pattern option registry — DONE
- M74: PrintConfig lateral lattice and infill anchor option registry — DONE
- M75: PrintConfig acceleration option registry — DONE
- M76: PrintConfig wall, infill, and travel jerk option registry — DONE
- M77: PrintConfig default jerk registry with pre-middle shard split — DONE
- M78: PrintConfig initial-layer line, speed, and slow-down registry — DONE
- M79: PrintConfig first-layer temperature and fan-speed registry — DONE
- M80: PrintConfig filament ironing override option registry — DONE
- M81: PrintConfig fuzzy-skin option registry — DONE
- M82: PrintConfig process and G-code utility option registry — DONE
- M83: PrintConfig nozzle material and hardness option registry — DONE
- M84: PrintConfig printer structure and fan speed-up option registry — DONE
- M85: PrintConfig fan PWM, cost, and printer support option registry — DONE
- M86: PrintConfig G-code flavor and object-label option registry — DONE
- M87: PrintConfig infill combination and rotation-template option registry — DONE
- M88: PrintConfig skin, skeleton, and combined-infill option registry — DONE
- M89: PrintConfig wrapping detection and sparse-infill utility option registry — DONE
- M90: PrintConfig inheritance, MMU interlocking, and calibration flag option registry — DONE
- M91: PrintConfig ironing and Z contouring option registry — DONE
- M92: PrintConfig custom G-code, machine limit flag, and small-area flow option registry — DONE
- M93: PrintConfig machine speed and acceleration limit option registry — DONE
- M94: PrintConfig machine jerk, min-rate, and acceleration PRT option registry — DONE
- M95: PrintConfig resonance avoidance and input shaping option registry — DONE
- M96: PrintConfig fan max and extrusion-rate smoothing option registry — DONE
- M97: PrintConfig auxiliary fan, min layer, and nozzle option registry — DONE
- M98: PrintConfig notes, host, nozzle-volume, and MMU parking registry — DONE
- M99: PrintConfig loading move, start/end points, ooze, and filename registry — DONE
- M100: PrintConfig make-overhang and wall option registry — DONE
- M101: PrintConfig wall loop, post-process, and printer identity registry — DONE
- M102: PrintConfig raft, resolution, and retraction trigger registry — DONE
- M103: PrintConfig retraction length, cut, and toolchange registry — DONE
- M104: PrintConfig Z-hop, lift-boundary, and extruder/nozzle type registry — DONE
- M105: PrintConfig extruder variant and ID registry — DONE
- M106: PrintConfig restart, retraction speed, M73, and seam registry — DONE
- M107: PrintConfig scarf seam registry — DONE
- M108: PrintConfig wipe speed and loop registry — DONE
- M109: PrintConfig skirt and draft-shield registry — DONE
- M110: PrintConfig slowdown and solid-infill registry — DONE
- M111: PrintConfig internal solid infill and spiral registry — DONE
- M112: PrintConfig timelapse and preheat registry — DONE
- M113: PrintConfig start G-code and filament-change registry — DONE
- M114: PrintConfig wipe-tower and ramming registry — DONE
- M115: PrintConfig priming, slicing mode, Z offset, and support-enable registry — DONE
- M116: PrintConfig support type and support placement registry — DONE
- M117: PrintConfig support Z-distance and enforced layers registry — DONE
- M118: PrintConfig support filament registry — DONE
- M119: PrintConfig support interface base avoidance and line width registry — DONE
- M120: PrintConfig support interface loop, filament, layers, and spacing registry — DONE
- M121: PrintConfig support bottom interface spacing, interface speed, and patterns registry — DONE
- M122: PrintConfig support pattern spacing, speed, expansion, and style registry — DONE
- M123: PrintConfig support independent layer height and threshold registry — DONE
- M124: PrintConfig tree support branch and tip registry — DONE
- M125: PrintConfig tree support diameter, wall, and infill registry — DONE
- M126: PrintConfig support ironing registry — DONE
- M127: PrintConfig chamber temperature registry — DONE
- M128: PrintConfig nozzle temperature registry — DONE
- M129: PrintConfig head-wrap detect zone and thin-wall registry — DONE
- M130: PrintConfig change G-code registry — DONE
- M131: PrintConfig top-surface and top-shell registry — DONE
- M132: PrintConfig surface-density registry — DONE
- M133: PrintConfig travel-speed registry — DONE
- M134: PrintConfig wipe and prime-tower base registry — DONE
- M135: PrintConfig flush and prime-volume registry — DONE
- M136: PrintConfig wipe-tower placement and width registry — DONE
- M137: PrintConfig wipe-tower angle and brim registry — DONE
- M138: PrintConfig wipe-tower max purge speed registry — DONE
- M139: PrintConfig wipe-tower wall type registry — DONE
- M140: PrintConfig wipe-tower rib and filament registry — DONE
- M141: PrintConfig prime-tower interface registry — DONE
- M142: PrintConfig flush-into registry — DONE
- M143: PrintConfig wipe-tower extra and idle-temperature registry — DONE
- M144: PrintConfig XY compensation and polyhole registry — DONE
- M145: PrintConfig thumbnails registry — DONE
- M146: PrintConfig relative E and wall-generator registry — DONE
- M147: PrintConfig wall-transition registry — DONE
- M148: PrintConfig minimum feature and wall-length registry — DONE
- M149: PrintConfig wall maximum resolution registry — DONE
- M150: PrintConfig bead-width registry — DONE
- M151: PrintConfig filament extruder override registry — DONE
- M152: PrintConfig narrow internal solid infill registry — DONE
- M153: PrintConfig extruder/filament option key lists — DONE
- M154: PrintConfig SLA display and tilt registry — DONE
- M155: PrintConfig SLA relative correction registry — DONE
- M156: PrintConfig SLA axis and absolute correction registry — DONE
- M157: PrintConfig SLA foot and gamma correction registry — DONE
- M158: PrintConfig SLA material identity and cost registry — DONE
- M159: PrintConfig SLA exposure time registry — DONE
- M160: PrintConfig SLA material correction registry — DONE
- M161: PrintConfig SLA profile identifiers registry — DONE
- M162: PrintConfig SLA support head and pillar registry — DONE
- M163: PrintConfig SLA support base and placement registry — DONE
- M164: PrintConfig SLA support points registry — DONE
- M165: PrintConfig SLA pad base registry — DONE
- M166: PrintConfig SLA zero-elevation pad registry — DONE
- M167: PrintConfig SLA hollowing registry — DONE
- M168: PrintConfig SLA material speed registry — DONE
- M169: PrintConfig legacy key aliases — DONE
- M170: PrintConfig legacy simple value migrations — DONE
- M171: PrintConfig legacy different-settings key-list normalization — DONE
- M172: PrintConfig legacy wall sequence migrations — DONE
- M173: PrintConfig legacy extruder variant values — DONE
- M174: PrintConfig legacy recovery shell rotation migrations — DONE
- M175: PrintConfig legacy alias and top-wall migrations — DONE
- M176: PrintConfig legacy ironing and draft-shield migrations — DONE
- M177: PrintConfig legacy pattern migrations — DONE
- M178: PrintConfig legacy filament migrations — DONE
- M179: PrintConfig legacy prime-tower rib and hardware migrations — DONE
- M180: PrintConfig legacy obsolete-key ignore list — DONE
- M181: PrintConfig legacy thumbnail composite normalization — DONE
- M182: PrintConfig legacy wiping-volumes matrix composite — DONE
- M183: GCodeThumbnails definition parser API — DONE
- M184: PrintConfig variant option key sets — DONE
- M185: PrintConfig min object distance API — DONE
- M186: PrintConfig normalize_fdm extruder role propagation — DONE
- M187: PrintConfig normalize_fdm spiral mode normalization — DONE
- M188: PrintConfig normalize_fdm resolution clamp — DONE
- M189: PrintConfig normalize_fdm prime tower normalization — DONE
- M190: PrintConfig normalize_fdm_2 prime tower changed keys — DONE
- M191: PrintConfig handle_legacy_sla correction expansion — DONE
- M192: PrintConfig get_parameter_size API — DONE
- M193: PrintConfig extend_extruder_variant API — DONE
- M194: PrintConfig set_num_extruders vector resizing API — DONE
- M195: PrintConfig set_num_filaments vector resizing API — DONE
- M196: PrintConfig validate basic dimension and count checks — DONE
- M197: PrintConfig validate firmware retraction compatibility — DONE
- M198: PrintConfig validate gcode flavor enum value — DONE
- M199: PrintConfig validate infill pattern enum values — DONE
- M200: PrintConfig validate skirt height and bridge flow ratios — DONE
- M201: PrintConfig validate extruder clearance dimensions — DONE
- M202: PrintConfig validate filament flow ratio — DONE
- M203: PrintConfig validate spiral vase CLI constraints — DONE
- M204: PrintConfig validate extrusion width limit — DONE
- M205: PrintConfig validate line-width numeric ranges — DONE
- M206: PrintConfig validate min-only line-width numeric ranges — DONE
- M207: PrintConfig validate FFF aggregate API — DONE
- M208: DynamicPrintConfig validate printer-technology dispatch — DONE
- M209: DynamicPrintConfig filament type display API — DONE
- M210: DynamicPrintConfig different extruders API — DONE
- M211: DynamicPrintConfig support different extruders API — DONE
- M212: DynamicPrintConfig get_index_for_extruder no-id lookup — DONE
- M213: DynamicPrintConfig get_index_for_extruder complete-id lookup — DONE
- M214: DynamicPrintConfig get_index_for_extruder generated-ID lookup — DONE
- M215: DynamicPrintConfig update_values_from_single_to_multi string/int copy — DONE
- M216: DynamicPrintConfig update_values_from_single_to_multi float limit — DONE
- M217: DynamicPrintConfig update_values_from_single_to_multi FloatOrPercent limit — DONE
- M218: DynamicPrintConfig update_values_from_single_to_multi bool resize — DONE
- M219: DynamicPrintConfig update_values_from_multi_to_multi string/int copy — DONE
- M220: DynamicPrintConfig update_values_from_multi_to_multi float merge — DONE
- M221: DynamicPrintConfig update_values_from_multi_to_multi FloatOrPercent merge — DONE
- M222: DynamicPrintConfig update_values_from_multi_to_multi bool merge — DONE
- M223: DynamicPrintConfig update_values_from_multi_to_multi_2 float nullable merge — DONE
- M224: DynamicPrintConfig update_values_from_multi_to_multi_2 FloatOrPercent nullable merge — DONE
- M225: DynamicPrintConfig update_values_from_multi_to_multi_2 bool nullable merge — DONE
- M226: DynamicPrintConfig update_values_from_multi_to_single_2 float nullable collapse — DONE
- M227: DynamicPrintConfig update_values_from_multi_to_single_2 FloatOrPercent nullable collapse — DONE
- M228: DynamicPrintConfig update_values_from_multi_to_single_2 bool nullable collapse — DONE
- M229: DynamicPrintConfig filament identity query API — DONE
- M230: DynamicPrintConfig update_values_to_printer_extruders string/int copy — DONE
- M231: DynamicPrintConfig update_values_to_printer_extruders float/percent copy — DONE
- M232: DynamicPrintConfig update_values_to_printer_extruders FloatOrPercent copy — DONE
- M233: DynamicPrintConfig update_values_to_printer_extruders bool copy — DONE
- M234: DynamicPrintConfig update_values_to_printer_extruders enum copy — DONE
- M235: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments string/int copy — DONE
- M236: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments float/percent copy — DONE
- M237: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments FloatOrPercent copy — DONE
- M238: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments bool copy — DONE
- M239: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments enum copy — DONE
- M240: DynamicPrintConfig normalize stride-2 float vectors — DONE
- M241: DynamicPrintConfig non-diff base-config variant index setup — DONE
- M242: DynamicPrintConfig non-diff base-config direct inheritance — DONE
- M243: DynamicPrintConfig non-diff different-key no-op classification — DONE
- M244: DynamicPrintConfig non-diff restore count guard — DONE
- M245: DynamicPrintConfig non-diff restore stride and expected size — DONE
- M246: DynamicPrintConfig non-diff stride-2 float type check — DONE
- M247: DynamicPrintConfig non-diff stride-2 size mismatch detection — DONE
- M248: DynamicPrintConfig non-diff stride-2 source and target normalization — DONE
- M249: DynamicPrintConfig non-diff stride-2 set_with_restore mapping — DONE
- M250: DynamicPrintConfig non-diff stride-1 vector size mismatch detection — DONE
- M251: DynamicPrintConfig non-diff stride-1 source vector resize — DONE
- M252: DynamicPrintConfig non-diff stride-1 target temporary resize — DONE
- M253: DynamicPrintConfig non-diff stride-1 set_with_restore mapping — DONE
- M254: DynamicPrintConfig diff child-config variant index setup — DONE
- M255: DynamicPrintConfig diff child-config direct set branch — DONE
- M256: DynamicPrintConfig diff vector set_only_diff branch — DONE
- M257: DynamicPrintConfig diff child-config update assembly — DONE
- M258: compute_filament_override_value long-retraction override defaults — DONE
- M259: ConfigOptionVector apply_override mapping — DONE
- M260: compute_filament_override_value update assembly — DONE
- M261: Print filament override key-loop assembly — DONE
- M262: PrintApply print-config diff scalar branch — DONE
- M263: PrintApply full print-config diff branch — DONE
- M264: PrintApply printable-filament change guard — DONE
- M265: PrintApply printable-area polygon extraction — DONE
- M266: PrintApply scaled printable-area polygons — DONE
- M267: PrintApply extruder diff first-result collection — DONE
- M268: PrintApply all-extruder intersection append — DONE
- M269: PrintApply find_intersections control flow — DONE
- M270: PrintApply intersection-id set comparison — DONE
- M271: PrintApply printable-filament staged assembly — DONE
- M272: PrintApply LayerRanges assign normalization — DONE
- M273: PrintApply instance printable-filament invalidation — DONE
- M274: PrintApply LayerRanges config lookup — DONE
- M275: PrintApply ModelObjectStatus state — DONE
- M276: PrintApply ModelObjectStatusDB operations — DONE
- M277: PrintApply PrintObjectStatus state — DONE
- M278: PrintApply PrintObjectStatusDB operations — DONE
- M279: PrintApply model-volume solid-or-modifier predicate — DONE
- M280: PrintApply bbox transform composition — DONE
- M281: PrintApply transform Z-rotation/mirroring predicate — DONE
- M282: PrintApply transformed indexed-triangle bbox2d — DONE
- M283: PrintApply transformed indexed-triangle bboxes in Z ranges — DONE
- M284: PrintApply keep reusable cached volume ids — DONE
- M285: PrintApply find volume extents lookup — DONE
- M286: PrintApply find modifier volume extents — DONE
- M287: PrintApply print region ref count helpers — DONE
- M288: PrintApply verify-update region initialization — DONE
- M289: PrintApply verify-update volume-region matching — DONE
- M290: PrintApply verify-update parent scan existing overrides — DONE
- M291: PrintApply verify-update current modifier bbox lookup — DONE
- M292: PrintApply verify-update parent bbox intersection gate — DONE
- M293: PrintApply verify-update missing override config gate — DONE
- M294: PrintApply verify-update existing region config change gate — DONE
- M295: PrintApply verify-update existing region ref-count split gate — DONE
- M296: PrintApply verify-update existing region config diff keys — DONE
- M297: PrintApply verify-update existing region invalidate callback — DONE
- M298: PrintApply verify-update existing region config apply-only — DONE
- M299: PrintApply verify-update existing region ref increment — DONE
- M300: PrintApply painted region extruder config — DONE
- M301: PrintApply painted region update gate — DONE
- M302: PrintApply painted region config apply — DONE
- M303: PrintApply painted region ref increment — DONE
- M304: PrintApply fuzzy painted region config — DONE
- M305: PrintApply fuzzy painted region update/apply — DONE
- M306: PrintApply fuzzy painted region ref increment — DONE
- M307: PrintApply region merge verification — DONE
- M308: PrintApply update_volume_bboxes volume order/cache ids — DONE
- M309: PrintApply update_volume_bboxes single-layer extents — DONE
- M310: PrintApply update_volume_bboxes multi-layer old extents — DONE
- M311: PrintApply update_volume_bboxes multi-layer expanded ranges — DONE
- M312: PrintApply update_volume_bboxes multi-layer cached reuse — DONE
- M313: PrintApply update_volume_bboxes multi-layer uncached insertion — DONE
- M314: PrintApply generate_print_object_regions layer-range shell — DONE
- M315: PrintApply generate_print_object_regions update_volume_bboxes call — DONE
- M316: PrintApply generate_print_object_regions region-set helper — DONE
- M317: PrintApply generate_print_object_regions model-part volume region — DONE
- M318: PrintApply generate_print_object_regions modifier parent scan — DONE
- M319: PrintApply generate_print_object_regions modifier changed-config append — DONE
- M320: PrintApply generate_print_object_regions modifier unchanged fallback — DONE
- M321: PrintApply generate_print_object_regions painted region append — DONE
- M322: PrintApply generate_print_object_regions painted region sort — DONE
- M323: PrintApply generate_print_object_regions fuzzy volume-region append — DONE
- M324: PrintApply generate_print_object_regions fuzzy painted-region append — DONE
- M325: PrintApply generate_print_object_regions fuzzy painted-region sort — DONE
- M326: PrintApply apply normalization prelude — DONE
- M328: PrintApply apply support-used flag — DONE
- M329: PrintApply apply scarf joint seam flag — DONE
- M335: PrintApply apply filament_map extraction — DONE
- M337: PrintApply apply filament_map_mode guard — DONE
- M338: PrintApply apply filament_map auto-mode gate — DONE
- M339: PrintApply apply auto filament_map diff prune — DONE
- M340: PrintApply apply manual filament_map setup — DONE
- M341: PrintApply apply manual filament_map same-map prune — DONE
- M342: PrintApply apply print_diff set reassignment — DONE
- M343: PrintApply apply status initial diff update — DONE
- M344: PrintApply print_diff config invalidation — DONE
- M345: PrintApply full_config_diff placeholder entry — DONE
- M351: PrintApply extruder count change handling — DONE
- M852: OrcaSlicer source crate partition checkpoint — DONE

## Completed 22O task slices

- Task 22O.1: Classic perimeter prelude — DONE
- Task 22O.2: Classic dynamic top-one-wall split — DONE
- Task 22O.3: Classic raw-onion iteration — DONE
- Task 22O.4: Classic loop hierarchy — DONE
- Task 22O.5: Classic traversal seeds — DONE
- Task 22O.6: Exact open-path Clipper — DONE
- Task 22O.7: Raw extrusion path materialization — DONE
- Task 22O.8: Chained extrusion loops — DONE
- Task 22O.9: Ordered entity collections — DONE
- Task 22O.10: Perimeter collection append — DONE
- Task 22O.11: Pre-medial Classic gap domain — DONE
- Task 22O.12: ThickPolyline medial-axis prerequisite — DONE
- Task 22O.13: Classic gap medial-axis extraction — DONE
- Task 22O.14: Classic variable-width gap extrusion — DONE
- Task 22O.15: Classic infill-boundary construction — DONE
- Task 22O.16: Layer-region perimeter outputs — DONE
- Task 22O.17: Surface-type detection and clipped fill transfer — DONE
- Task 22O.18: Fill-surface shell preparation — DONE
- Task 22O.19: Single-region vertical-shell cache — DONE
- Task 22O.20: Single-region vertical-shell projection gather — DONE
- Task 22O.21: Single-region vertical-shell internal trimming — DONE
- Task 22O.22: Single-region vertical-shell morphology regularization — DONE
- Task 22O.23: Single-region vertical-shell tiny-island filtering — DONE
- Task 22O.24: Single-region vertical-shell fill-surface assignment — DONE
- Task 22O.25: Horizontal-shell extra-solid promotion — DONE
- Task 22O.26: Horizontal-shell propagation — DONE
- Task 22O.27: Region-expansion direct wave propagation — DONE
- Task 22O.28: ClipperZ wave-seed discovery — DONE
- Task 22O.29: source-taking RegionExpansion propagation — DONE
- Task 22O.30: direct RegionExpansionEx wave output — DONE
- Task 22O.31: source-taking RegionExpansionEx composition — DONE
- Task 22O.32: group expanded polygons by source ExPolygon — DONE
- Task 22O.33: merge region expansions into source ExPolygons — DONE
- Task 22O.34: compose source wave expansion and ExPolygon merge — DONE
- Task 22O.35: expand and merge one external-surface kind — DONE
- Task 22O.36: compose bridge anchors and expansions across ordered zones — DONE
- Task 22O.37: group bridge regions by overlapping expansions — DONE
- Task 22O.38: direct bridge-direction selection — DONE
- Task 22O.39: detect grouped bridge directions — DONE
- Task 22O.40: merge bridge groups — DONE
- Task 22O.41: expand bridges and detect orientations — DONE
- Task 22O.42: active external-surface processing — DONE
- Task 22O.43: gather internal-bridge candidates — DONE
- Task 22O.44: connect infill to its boundary — DONE
- Task 22O.45: generate and connect CrossHatch fill surfaces — DONE
- Task 22O.46: generate sparse infill polylines for anchoring — DONE
- Task 22O.47: gather deep sparse bridge area — DONE
- Task 22O.48: resolve thick solid-infill bridge Flow — DONE
- Task 22O.49: apply the internal bridge angle override — DONE
- Task 22O.50: nearest anchor-line AABB tree — DONE
- Task 22O.51: automatic bridge-angle vote — DONE
- Task 22O.52: indexed line intersections and contour outside — DONE
- Task 22O.53: anchored bridge polygon construction — DONE
- Task 22O.54: bridge candidate-layer clustering — DONE
- Task 22O.55: bridge candidate ordering — DONE
- Task 22O.56: lower-cluster bridge subtraction — DONE
- Task 22O.57: current-layer bridge expansion context — DONE
- Task 22O.58: candidate bridge area filtering — DONE
- Task 22O.59: candidate boundary polylines — DONE
- Task 22O.60: candidate bridge angle composition — DONE
- Task 22O.61: candidate anchored bridge — DONE
- Task 22O.62: candidate collision reconstruction — DONE
- Task 22O.63: bridge postprocessing geometry — DONE
- Task 22O.64: bridge candidate commit history — DONE
- Task 22O.65: bridge rewrite-area collection — DONE
- Task 22O.66: region bridge ensuring-area preparation — DONE
- Task 22O.67: internal infill rebuild — DONE
- Task 22O.68: internal bridge surface conversion — DONE
- Task 22O.69: internal solid recomposition — DONE
- Task 22O.70: region bridge surface commit — DONE
- Task 22O.71: first internal bridge transaction — DONE
- Task 22O.72: infill-combination identity gate — DONE
- Task 22O.73: base fill grouping — DONE
- Task 22O.74: full fill grouping — DONE
- Task 22O.75: full-grouping sparse anchoring — DONE
- Task 22O.76: CrossHatch fill entities — DONE
- Task 22O.77: rectilinear vertical segmentation — DONE
- Task 22O.78: rectilinear contour links — DONE
- Task 22O.79: rectilinear pinch intersections — DONE
- Task 22O.102: Arachne transition middle generation — DONE
- Task 22O.80: monotonic region generation — DONE
- Task 22O.81: monotonic region neighbors — DONE
- Task 22O.82: rectilinear contour context — DONE
- Task 22O.83: rectilinear perimeter primitives — DONE
- Task 22O.84: source rectilinear links — DONE
- Task 22O.85: monotonic region costs — DONE
- Task 22O.86: monotonic path matrix — DONE
- Task 22O.87: monotonic ant chain — DONE
- Task 22O.88: monotonic polyline emission — DONE
- Task 22O.89: monotonic surface filler — DONE
- Task 22O.90: monotonic fill entities — DONE
- Task 22O.91: layer fill entity stage — DONE
- Task 22O.92: thin fill append — DONE
- Task 22O.93: layer-region extrusion ownership — DONE
- Task 22O.94: extrusion island assignment — DONE
- Task 22O.95: island print phase order — DONE
- Task 22O.96: pure infill chaining dependencies — DONE
- Task 22O.97: external seam candidate topology — DONE
- Task 22O.98: Arachne extrusion-line primitives — DONE
- Task 22O.99: Arachne beading strategies — DONE
- Task 22O.100: Arachne skeletal graph — DONE
- Task 22O.101: Arachne skeletal trapezoidation builder — DONE
- Task 22O.103: Project G-code emission lifecycle — DONE
- Task 22O.125: Effective-flow volumetric speed cap — DONE
- Task 22O.126: Layer interval precision — DONE
- Task 22O.127: Forward stored-path wipe — DONE
- Task 22O.128: Global seam visibility — DONE
- Task 22O.129: Aligned seam strings — DONE
- Task 22O.130: Internal seam corner projection — DONE
- Task 22O.131: Lifted next-layer travel — DONE
- Task 22O.132: Emitted-loop wipe path — DONE
- Task 22O.133: Contiguous extrusion feedrate state — DONE
- Task 22O.134: Overhang-role base kinematics — DONE
- Task 22O.135: Dynamic overhang overlap speeds — DONE
- Task 22O.136: Quantized dynamic-segment extrusion — DONE
- Task 22O.137: Rounded dynamic-overhang base speed — DONE
- Task 22O.138: Cooling-buffer feedrate normalization — DONE
- Task 22O.139: Reverse extrusion-path wipe — DONE
- Task 22O.140: Remove simplification checkpoint pinning — DONE
- Task 22O.141: Arc-path source simplification — DONE
- Task 22O.142: Inner-path aligned seam projection — DONE
- Task 22O.143: Seam-gap clip ordering — DONE
- Task 22O.144: Pre-seam arc-range simplification — DONE
- Task 22O.145: Forward stored-path wipe traversal — DONE
- Task 22O.146: Aligned collection seam target — DONE
- Task 22O.147: Lifted-travel modal feedrate — DONE
- Task 22O.148: Retained pre-seam fitted arcs — DONE
- Task 22O.149: Arc-offset number formatting — DONE
- Task 22O.150: Pre-fit aligned seam corner — DONE
- Task 22O.151: Full-pivot aligned-seam precision — DONE
- Task 22O.152: Retained-polyline fitted-arc wipe — DONE
- Task 22O.153: Scaled object-origin arithmetic — DONE
- Task 22O.154: Source-epsilon extrusion filtering — DONE
- Task 22O.155: Near-vertex seam snapping — DONE
- Task 22O.156: Standard monotonic queue distribution — DONE
- Task 22O.157: Exact rational scanline ordering — DONE
- Task 22O.158: Adjusted solid fill flow — DONE
- Task 22O.159: Zero-initialized monotonic link limit — DONE
- Task 22O.160: Retained fill arc-fitting payload — DONE
- Task 22O.161: Standalone extrusion wipe direction — DONE
- Task 22O.162: Generated filament statistics — DONE
- Task 22O.163: Arachne transition midpoint radius — DONE
- Task 22O.164: Arachne central-end transition filter — DONE
- Task 22O.165: Arachne bead-count region dissolution — DONE
- Task 22O.166: Arachne nearby transition discovery — DONE
- Task 22O.167: Arachne transition-middle filtering — DONE
- Task 22O.168: Arachne transition module partition — DONE
- Task 22O.169: Arachne transition-end generation — DONE
- Task 22O.170: Arachne transition-end continuation — DONE
- Task 22O.171: Arachne transition-end branch direction — DONE
- Task 22O.172: Arachne transition application — DONE
- Task 22O.173: Arachne transition test-support partition — DONE
- Task 22O.174: Arachne nonlinear extra ribs — DONE
- Task 22O.175: Arachne node beading storage — DONE
- Task 22O.176: Arachne transitional beading interpolation — DONE
- Task 22O.177: Arachne upward segment ordering — DONE
- Task 22O.178: Arachne upward beading propagation — DONE
- Task 22O.179: Arachne downward beading copy — DONE
- Task 22O.180: Arachne downward beading merge — DONE
- Task 22O.181: Arachne downward propagation dispatch — DONE
- Task 22O.182: Arachne node-beading propagation stage — DONE
- Task 22O.183: Arachne segment junction generation — DONE
- Task 22O.184: Arachne toolpath segment assembly — DONE
- Task 22O.185: Arachne paired junction connection — DONE
- Task 22O.186: Arachne quad peak edge selection — DONE
- Task 22O.187: Arachne even-quad junction connection — DONE
- Task 22O.188: Arachne adjacent junction concatenation — DONE
- Task 22O.189: Arachne odd-quad classification — DONE
- Task 22O.190: Arachne junction domain traversal — DONE
- Task 22O.191: Arachne odd-segment deduplication — DONE
- Task 22O.192: Arachne local-maximum single beads — DONE
- Task 22O.193: Arachne segment-stage orchestration — DONE
- Task 22O.194: Arachne full toolpath orchestration — DONE
- Task 22O.195: raw Arachne wall toolpaths — DONE
- Task 22O.196: Arachne wall-line stitching — DONE
- Task 22O.197: Arachne small odd-line removal — DONE
- Task 22O.198: Arachne inner-contour separation — DONE
- Task 22O.199: Arachne variable-width simplification — DONE
- Task 22O.200: Arachne concentric-internal entities — DONE
- Task 22O.201: concentric loop finalization — DONE
- Task 22O.202: per-ExPolygon no-overlap intersection — DONE
- Task 22O.203: Arachne outline preparation — DONE
- Task 22O.204: skeletal beading recovery — DONE
- Task 22O.205: source circle-fit arithmetic — DONE
- Task 22O.206: monotonic three-opt path pass (withdrawn) — DONE
- Task 22O.207: flat rectilinear offset contours — DONE
- Task 22O.208: rectilinear stick and small-contour cleanup — DONE
- Task 22O.209: medial Voronoi coordinate conversion — DONE
- Task 22O.210: inclusive island bounds prefilter — DONE
- Task 22O.211: preserve classic surface ownership — DONE
- Task 22O.213: sparse infill simplification resolution — DONE
- Task 22O.214: source no-op monotonic three-opt — DONE
- Task 22O.215: live-cursor infill entity chaining — DONE
- Task 22O.216: concentric path orientation — DONE
- Task 22O.217: clipped-arc endpoint quantization — DONE
- Task 22O.218: clipped-arc sweep containment — DONE
- Task 22O.219: fitted-circle center quantization — DONE
- Task 22O.220: rectilinear offset float scaling — DONE
- Task 22O.221: rectilinear rotation half-up rounding — DONE
- Task 22O.222: fitted-circle Point constructor rounding — DONE
- Task 22O.223: internal-region retraction containment — DONE
- Task 22O.224: Rectilinear offset integer conversion — DONE
- Task 22O.225: compensation PolyTree ordering — DONE
- Task 22O.226: grouped offset PolyTree execution — DONE
- Task 22O.227: Arachne dense domain traversal — DONE
- Task 22O.228: elephant-foot batch preservation — DONE
- Task 22O.229: project print-flow ratio — DONE
- Task 22O.230: processor acceleration envelopes — DONE
- Task 22O.231: processor centripetal acceleration — DONE
- Task 22O.232: module and oracle cleanup — DONE
- Task 22O.244: closing offset coordinate determinism — DONE
- Task 22O.245: rectilinear fractional offsets — DONE
- Task 22O.246: semantic golden boundary — DONE
- Task 22O.247: scheduler-independent semantic golden — DONE

## Web preview and editing upstream slices (ARD-0106 follow-up)

The web frontend (ARD-0106, M1 merged) is unblocked by two source-cited
slices. Each slice gets its own REVIEW-REQUEST:

1. M853:
   `GCodeProcessorResult::moves` and `LibVGCodeWrapper::convert` →
   ares-core / ares-vgcode, for toolpath preview.
2. M854:
   `ConfigBase::set_deserialize` user overrides at the project slice
   boundary, for settings editing and re-slice.

M853 lands first because Preview is the larger user-visible gap. M854 is
independent and can follow on the same branch.

### ColorChange / M600 preview continuation

Filament/ColorPrint `9b5db6d7a` is approved and merged. The
[ColorChange rewrite spec](architecture/web-colorchange.md) is approved and
merged at `70323c9a7`: upstream
`GCodeProcessor` custom items, `UsedFilaments::volumes_per_color_change` and
normal custom times → ares-core; `libvgcode::ColorPrint` and GCodeViewer
partial display records → ares-vgcode; thin WASM and lazy display-only TS.
Exit requires native-oracle-backed event/volume/segment parity, full existing
browser coverage plus ColorChange E2E artifacts, unchanged bundle budget,
Rust/build gates and coordinator approval. Native probing resolved the
zero-volume question: preserve raw data and report partial usage unavailable
rather than indexing absent volumes. Automatic M600 injection and
bare-M600 color inference remain outside this slice.

Verification on `43df7c95d`: workspace 7,935 passed / 9 skipped; seeded
reference 561/561 in both trees; full browser 14/14 base and 15/15 candidate;
candidate initial JS gzip 124,953 B. Official retains eight shared environment
failures; the extra base tree-support failure has exact raw-Ares equality
and differing native M73 insertion evidence. See the architecture record for
artifacts, setup corrections and approved diagnostic distinctions. A later
VG1/UsedFilaments merger must rebase and rerun.

### Tool-position Time row — implemented, awaiting review

The approved [marker-time slice](architecture/web-marker-time.md) ports
`libvgcode::ViewerImpl::get_estimated_time_at` (`ViewerImpl.cpp:1461-1465`)
into ares-vgcode and transports its inclusive f32 prefix plus the existing
selected vertex duration to the lazy Time property
(`GCodeViewer.cpp:345-355,452-454`). No core timing, M73, slicing or accounting
changes. The README's slider `alternate_values` time/distance claim was
incorrect: those values are G-code IDs, not a new feature to invent.

Exit: source-kernel/native processor f32-bit and display-text evidence,
RED-before-GREEN, unchanged G-code/M73, reload/filter/seam correctness, full
Rust/build/browser gates and initial JS gzip strictly below 125,000 B.
Stealth UI, acceleration/jerk/PA, actual-speed profile and slider time/distance
remain deferred. Historical VG1 checkpoint `9e3e81837`: full browser base/candidate
15/15 and 16/16 passed, initial gzip 124,949/124,954 B. Workspace: 7,953 passed,
9 skipped. Corrected-protocol seeded: 575/575 on both trees. Official: 566/575
on both trees, with eight shared environment failures and two distinct native
ND cases backed by identical raw Ares bytes. That checkpoint is historical.

Final gates on `1b1751434` (later main commits through `21c18b79d` are
docs-only): fmt, workspace all-target clippy `-D warnings`, workspace nextest
**7,964 passed, 9 skipped**, CLI/example/WASM/library/app builds, typecheck
and lint. Fresh corrected v2 references with empty caches: seeded **585/585**
on both trees; official base **574/585**, candidate **575/585**. Eight shared
reference-environment failures remain. Base-only `painted-blocker` is ND
(identical raw Ares bytes). Two shared native-oracle 300 s SIGKILLs were rerun
solo per ruling (threads=1, unchanged timeout): `change_filament_flush_blocks`
passes on both trees; `narrowed_x2d_whole_file` fails identically, with
identical Ares bytes, only on official 2.4.2's uninitialized object-label ids
(accepted as the `initialized_*` environment class, ARD-0081), and passes
whole-file under seeded on both trees. Full browser: base **14/15**,
candidate **15/16**; the single shared failure is `presetImport.spec.ts:96`
(`bridge_speed contains invalid value`), an O3 regression on main: native
official accepts the BBL array value (reported to the coordinator). Initial gzip
**124,951 / 124,948 B**. Not an all-green official gate; coordinator
review/acceptance remains required.
The tagged stable-Time authority and bounded raw difference are explicitly
approved and documented in the ColorChange Known difference section.

### Preview Acceleration / Jerk / Pressure Advance — spec approved, in progress

The [motion-attributes spec](architecture/web-motion-attributes.md) ports
`GCodeProcessor::store_move_vertex` `pressure_advance`/`acceleration`/`jerk`
recording (`GCodeProcessor.cpp:5630-5706,5771-5885`) and the M900/M572/
`SET_PRESSURE_ADVANCE` commands (`:5031-5060`) into ares-core, the
`LibVGCodeWrapper::convert` pass-through into ares-vgcode, and the three
`GCodeViewer` view-type items, legend ranges and ToolPosition rows into the
lazy web UI. Exit: native-oracle f32-bit vertex parity on processor fixtures,
unchanged G-code/M73, full Rust/reference/browser gates and initial JS gzip
strictly below 125,000 B. Approved with rulings (initialized-native PA oracle,
f32 bit-exact machine state); Klipper `SET_VELOCITY_LIMIT` travel-accel fix in-slice.

### `GCodeProcessor::process_M205` machine limits — integrated

Integrated at `d0e8b64f9bf35750f51f5e6ca5a6d01bbfcda16b`; see the
[source-owned M205 slice](architecture/gcode-processor-m205-machine-limits.md).
The Rust `processor/motion` rewrite ports `GCodeProcessor.cpp:5236-5265`
(X also sets Y; a later Y overrides; S/T/J update existing-parser values),
minimum-rate initialization including the RRF reset (`:2087-2098`), exact
E-delta travel/extrusion selection (`:3987-3989,4347-4349`), and the block
feedrate floor (`:5726-5740`) without changing commanded F or JD formulas.
Six permanent same-input Native/public processed-output cases preserve the
four original complete-byte REDs and two controls; all six are now PASS.

Fresh matching-base `7b4b1ee2b` acceptance: seeded base598/598 and
candidate604/604; official base588/598 and candidate594/604 have exactly
the same ten failure names (both exit100 retained, not a count-based waiver).
Workspace7990/7990 with nine existing skips, strict Clippy/rustfmt, and WASM
check/release pass. All517 common raw Ares G-code streams remain byte-identical;
118 inputs have exact archive SHA equality and399 have identical payloads
for every ZIP entry. All12006 upstream resource JSON files were inspected:
522 authored minimum-rate fields are zero, so the nonzero-rate stock subset
is empty; the67 missing retained stock inputs are not credited as PASS.
Browser E2E is explicitly waived for this slice: no web changes, unchanged
common output bytes, and no nonzero stock minimum rates. Independent remote
confirmation after non-force FF integration matches the approved hash.
