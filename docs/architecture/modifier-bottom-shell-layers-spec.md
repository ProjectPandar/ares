# Proposed slice: parameter-modifier bottom_shell_layers

Status: docs only, SPEC review requested. Worker3.
Dependency: code-approved top slice `5fa60caaeeadc73d6c64972aa121b549907a1931`,
awaiting stacked exclusive full. Its branch/worktree remain frozen unchanged.
This document lives in separate worktree /tmp/ares-worker3-modifier-bottom-spec,
branch pi/worker3-modifier-bottom-shell-spec, based on that candidate. After
stack merge, carry only this new proposal onto the broadcast main before any
implementation. No builds, tests, Native or fixtures are run/changed here.

## Remaining source difference

`project/effective_config/candidates.rs::first_unsupported_modifier_key` still
rejects ParameterModifier bottom_shell_layers. The top rewrite did not change
this branch or any filament selector. This is independent of whether the
modifier intersects a model part or equals its parent setting.

Native `region_config_from_model_volume` applies volume config to the parent
region and shell consumers read region-specific bottom counts, rather than
rejecting the option. Runtime admission/effect for this new matrix remains to
be proved after SPEC approval and a directed window. The top matrix's Native
success is adjacent evidence, not bottom admission or parity proof.

## Owning upstream boundaries

Pinned OrcaSlicer/src/libslic3r:

- PrintConfig.cpp:1156-1175: bottom_shell_layers registry, default3/min0, and
  bottom_shell_thickness. This slice varies count, fixes thickness0.
- PrintObject.cpp::region_config_from_model_volume:3664-3710: modifier inherits
  already resolved parent region; its own volume config applies last, without
  reapplying a layer-range override after modifier config.
- PrintApply.cpp::generate_print_object_regions:1032-1088: reverse-order
  intersecting model-part/modifier parents and child region registration.
- PrintObject.cpp::discover_vertical_shells:2008, bottom branch2252-2280:
  for each current region/layer, project lower bottom surfaces upward; count
  window uses i > idx_layer-n_bottom_layers and bottom_z differences, then
  the one-layer anchor-area special case. Preserve this bottom-specific
  direction/evaluation, not a textual top-loop reversal.
- PrintObject.cpp::discover_horizontal_shells:3955,3973-3995: evstAll delegates
  to vertical shells; stBottom and stBottomBridge use region bottom count;
  count0 disables that source-kind propagation.

This is a libslic3r region-config/shell rewrite slice, not a new Ares pipeline.

## Rust boundary and existing scaffold

Limit production to removing the bottom key from the private compatibility
rejection in effective_config/candidates.rs, provided the existing source
consumers pass the independent RED matrix. Preserve all six filament guards.

Reuse region_graph::add_modifier -> RegionBase::Modifier(parent) -> child
region_id and sliced modifier geometry in volume_regions/region_slices/complex.
PerimeterInputRecord.current.region_index determines region_options; vertical
projection stage.rs uses the child options and its matching cache range;
vertical_shell_projection/gather.rs:127-150 reads bottom count/thickness and
lower-layer bottom_z. horizontal_shell_propagation/window.rs:36-64 dispatches
Bottom/BottomBridge with the upward source propagation window.

These existing transports are temporary compatibility shells around named
upstream regions, not an invitation to duplicate a registry, apply a global
bottom override or select region zero. If the matrix reveals a missing owner,
stop and request an amendment instead of adding an unreviewed consumer patch.
No public load/selection/placement/API contract changes.

## Included and deferred

Include fixed-height, by-layer, one model part/tool, prism with a bottom-reaching
right-half ParameterModifier, positive sparse infill, parent bottom2,
bottom thickness0, modifier0/1/4/equal2, absent override and disjoint controls.
Include an overlapping later modifier1 to check priority, Classic/Arachne,
vertical evstAll and admitted horizontal-shell mode. Ordinary build-plate bottom
surfaces are the initial tracer; no support/raft policy needs changing.

Defer other modifier keys/filament selectors, range overrides, sequential,
spiral/ZAA, material/painting, raft/support and new bridge-producing geometry.
The existing BottomBridge consumer is preserved, but not claimed newly covered
by a plain prism. Keep its behavior unchanged; new bridge-specific behavior
would require its own fixtures and scope review.

## Native preflight, public RED and GREEN

After SPEC approval and an open directed window only, author input-only
3MF generator/provenance in a new bottom directory. Reuse the existing admitted
modifier-wall prism geometry, replacing its modifier wall override with bottom
count, independently record settings and input changes. Do not mutate top
fixtures or relabel top receipts as bottom evidence.

CPU12 outer taskset, jobs1/threads2, fresh cache/artifact root per run,
START/END inbox. In official and seeded, first require Native exit0/result0
and a local lower-layer solid/sparse geometry change between bottom1 and4.
Check modifier half vs parent half, rather than a global footer or elapsed time.
If zero or another case is rejected by Native, report and revise its admission
classification; do not add an Ares rejection to imitate it or retry to PASS.

Then commit fixtures/public RED separately: existing core and CLI slicing,
strict ordered complete-output comparison, API/CLI raw equality, input/G-code
SHA and first mismatch receipts. All override cases currently encounter the
bottom guard; no-override controls should retain strict output. Prove Native
admits the active local effect before GREEN. Only generator-line comparison
uses the existing permitted comparator; no other normalization/weakening.

After RED report, remove only bottom guard and prove the source consumers
through full-output GREEN. Preserve new bottom controls' raw bytes and rerun
unchanged top matrix controls. Keep each Rust file <=400 LOC. The old fixed-order
unit array bottom tuple (7->6) needs a separate precise retirement request after
RED; retain all remaining tuples/assertions/helper/imports and use an independent
test commit. Run that retained core test plus directed both modes, strict/fmt/
wasm32 before REVIEW. Workspace/full require allocation; no push.
