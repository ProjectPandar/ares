# ARD: upstream object layer-height profiles, not an Ares adaptive pipeline

Status: Stage1 approved; Stage2 requires separate approval.

## Non-negotiable decisions

1. Rewrite `libslic3r/Slicing.cpp::layer_height_profile_from_ranges`,
   `generate_object_layers`, and `PrintObject::update_layer_height_profile`
   into the existing `ares-core` object-layer planning interface. This is not
   a new Ares-owned layering algorithm. Preserve upstream arithmetic, profile
   selection, overlap precedence, first-layer rules, interpolation and Z basis.
2. Stage1 is object `layer_config_ranges.layer_height`; stage2 is imported
   painted `layer_height_profile`. Source-driven adaptive profile *generation*
   (`SlicingAdaptive`) is adjacent/deferred, not required to consume stored
   profiles. No automatic mesh-derived profile replaces an authored profile.
3. Existing `project_slice/profile.rs::fixed_layer_height_profile` is a partial
   compatibility shell for the empty-range upstream case. Replace its production
   role with the upstream range-profile owner; do not maintain two competing
   algorithms or add a legacy fallback. Reuse `layers.rs` pair interpolation,
   precise-Z alignment and `PlannedLayer` only where their source semantics agree.
4. Stage1 ports only explicit-height intervals. Mixed Some/None intervals retain
   their existing unsupported range boundary: upstream dereferences a missing
   `layer_height` option in Slicing.cpp:249 (`getFloat`), which is UB, not a
   default-height rule. Do not imitate UB or fabricate defaults; pure None keeps
   its pre-existing behavior. Coordinator confirmed this bounded scope.
5. Retain all source ranges, including height-only entries, default gaps and
   empty tails: PrintApply.cpp::LayerRanges::assign (:342-395) and
   generate_print_object_regions (:958-1027) preserve their global cardinality.
   Equal-config get_create_region deduplicates PrintRegion, not those ranges.
   The approved [spiral range slice](spiral-single-range-spec.md) corrects the
   former height-only candidate filter through this one canonical assignment.
   PrintObjectSlice.cpp:194-222 selects its single-range spiral threshold or
   release multi-range base mode using that global count. Decision4's mixed
   missing-height rejection is unchanged.
6. Remove only the capability branch proved supported by independent Native RED
   and subsequent strict GREEN. Painted and ZAA branches stay while deferred.
   Update earlier rejection tests before production changes, never after.
6. ZAA exists in admitted OrcaSlicer2.4.2 libslic3r but is **not adaptive layer
   generation**. Its owners are `PrintObject::contour_z`, `ContourZ.cpp`, mesh
   ray queries and spatial extrusion paths. Retain its active-feature gate until
   a separately approved source slice; do not treat existing box-inactive parity
   as proof for active slope following.
7. Keep byte-in/byte-out core platform-neutral (browserWASM/Windows/macOS/Linux),
   no direct file I/O, UI painting state, OpenGL, terminal or Native invocation.
   Existing archive adapter supplies object data; production core never executes
   Native. No extra crate or configurable strategy is necessary.

## Source identity

`OrcaSlicer/src/libslic3r/` checked files are byte-identical to admitted source
`/nix/store/hxvv0g1qlw9lwimj2km3ghxphrl9p4qf-orca-slicer-2.4.2-seed-replay-source/`:

| File | SHA256 |
|---|---|
| Slicing.cpp |eeb81fdb913fc28fda60d3b37818d14a556efaa38775837448553c97e9672e15|
| SlicingAdaptive.cpp |2e7564a717aa8e94faca7973fb7cb44ae2753c4f64555d404b834f547b250aa5|
| PrintObject.cpp |7efa9c467c6f32a46008a167d525458d582859f76706a5be6412a84d7c6ab589|
| ContourZ.cpp |59e7495cc05715f252b12305701d4632796be65647e7b0b301646a1ff8c387e4|

Read-only investigation only; no Native RED has yet run and no production Rust
has changed. See the [port spec](../superpowers/specs/2026-10-08-object-layer-height-profiles.md)
for exact source/destination ownership and approval/TDD milestones.
