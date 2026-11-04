# Proposed slice: parameter-modifier top_shell_layers

Per-object skirt and counterbore are separate slices; this proposal touches
neither. Modifier filament selectors remain outside this slice.

## Remaining difference

The UnsupportedProjectFeature inventory (2026-10-08, row05) lists modifier
shell counts separately from ordinary process shell settings. Current
`project/effective_config/candidates.rs::first_unsupported_modifier_key`
still rejects any ParameterModifier with top_shell_layers, including an equal
or geometrically inactive override. The already merged wall_loops slice
removed only its own gate and explicitly deferred top/bottom shell counts.

Native's region construction applies modifier volume config to the parent
PrintRegionConfig, and its shell consumers read region-specific top counts.
Thus this is a concrete source-supported admission candidate. Runtime Native
acceptance is not yet measured for the new matrix: prove admission first,
and stop if Native rejects or the public output is ambiguous.
Do not count registry presence or old global-option sweeps as a modifier PASS.

## Upstream ownership

Pinned `OrcaSlicer/src/libslic3r` source:

- `PrintConfig.cpp:6730` registers top_shell_layers; its typed region ownership
  does not make every global-option test exercise a modifier volume.
- `PrintObject.cpp::region_config_from_model_volume:3664-3710`: copy default
  or parent region config, apply the current volume config, and apply a range
  config only for model parts, not again after modifier overrides.
- `PrintApply.cpp::generate_print_object_regions:1032-1088`: modifiers visit
  intersecting model-part/modifier parents in reverse source-region order,
  derive child configs and retain region identity even through equal configs.
- `PrintObject.cpp::discover_vertical_shells:2008`, particularly 2223-2251:
  each region's top_shell_layers and top_shell_thickness determine the upper
  projection window into the current layer.
- `PrintObject.cpp::discover_horizontal_shells:3955`, 3973-3995:
  evstAll delegates to vertical shells; otherwise source stTop uses this
  region's top_shell_layers, and zero disables that propagation.

Existing sliced modifier geometry is owned by `PrintObjectSlice.cpp` and its
source-cited Rust `region_slices/complex.rs` rewrite; reuse that boundary,
not a new geometric masking pipeline.

## Rust destination and scaffold handling

Private ares-core destinations: `project/effective_config/candidates.rs`,
existing `options` region override resolution, `project/region_graph.rs` /
`project_slice/{volume_regions,region_slices/complex}.rs`, and only if the
public RED exposes a gap, existing
`prepare_infill/{vertical_shells,horizontal_shell_propagation}` consumers.

`region_graph::add_modifier` already resolves RegionBase::Modifier(parent)
with current volume overrides. Retain its identity and ordering, and the
existing local shell window's RegionOptions input. The top-only rejection is
a temporary compatibility shell: remove it after independent public RED and
prove the source-owned consumers operate on the modifier child region.
Do not invent a parallel region registry, whole-object shell override or
region-zero owner. If enabling this reveals a different missing owner, stop
for a supplemental SPEC rather than widening the production patch silently.

## Included and deferred

Include a single-tool, fixed-layer-height, by-layer object with one model part
and local ParameterModifier, positive sparse infill, zero top thickness to
isolate layer count, and top counts 0/1/4 versus a distinct parent count.
Include Classic and Arachne controls, equal-parent, no override and disjoint
modifier controls, and one overlapping modifier pair to prove source priority.
Include both vertical-shell evstAll and an admitted horizontal-shell mode.

Defer bottom_shell_layers, all filament selectors, painted/multimaterial
expansion, ranges, spiral, ZAA, support/raft and other modifier option families.
Keep their current behavior and guard ordering; no new guard or fallback.
No public import/selection/placement/API contract changes are proposed.

## RED, GREEN and evidence

Author input-only deterministic 3MF generator/provenance,
starting from the retained modifier-wall-loop control and replacing only the
modifier's wall override with top_shell_layers. Keep the parent, region options
and geometry separately recorded. Use a tall prism with a modifier reaching
the top over half its XY footprint, enough layers to distinguish count 1/4.

Run Native official and seeded in
fresh cache/artifact roots. Require exit0 plus effective config/geometry
witnesses; compare active Native top-count outputs against equal/no-override
controls to prove the local feature changes output. Then run the same inputs
through public core slicing and CLI: expect the current typed top guard RED,
with equal/no-override controls distinguished explicitly. Record all exit
channels, input/G-code SHA, API/CLI equality and the raw first mismatch. Do not
infer a local effect from a global config footer alone.

Commit fixtures and public Native-backed RED separately and report it
before GREEN. Keep strict ordered complete-output comparison in both modes;
no normalization beyond the existing generator-line comparator, retries,
assertion weakening, ignored tests or committed Native outputs. Any existing
negative top-guard assertion needs its own precise retirement.

GREEN must preserve unaffected raw control streams, each Rust file <=400 LOC,
and pass directed matrix plus strict clippy/fmt/wasm32.
