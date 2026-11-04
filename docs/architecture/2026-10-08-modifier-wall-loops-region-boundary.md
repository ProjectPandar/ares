# ARD: modifier wall-loop region ownership

## Non-negotiable decisions

1. Owner is libslic3r PrintObject.cpp::region_config_from_model_volume
   (3664–3710), PrintApply.cpp::generate_print_object_regions (958–1058) and
   Layer.cpp::is_perimeter_compatible/make_perimeters (139–211). This is a Rust
   rewrite slice of those boundaries, not a new Ares pipeline.
2. A modifier inherits the parent's already range-resolved config, then applies
   its own volume overrides. The modifier must not reapply the layer-range
   override after wall_loops. RegionOptions::resolve and region_graph::add_modifier
   remain the single config/parent/identity owners; no parallel registry.
3. The existing blanket wall_loops modifier rejection in effective_config/
   candidates.rs is only a temporary compatibility shell. Retire that one key
   after independent runtime RED. Do not relax adjacent shell/filament gates.
4. Distinct wall counts are perimeter-incompatible in Layer.cpp:146. Preserve
   source region-major identity, volume/range/parent ordering, physical-layer
   indexing and existing classic/Arachne consumers. If a larger missing source
   concept is exposed, stop for its own spec instead of inventing a local shim.
5. Explicit-height profiles and merged epsilon complex lookup are unchanged;
   fast shuffle remains exact. MM/fuzzy/spiral/adaptive/ZAA, material expansion,
   additional option families and multifilament remain deferred.
6. Both backends require independent legal runtime admission and complete-output
   proof. Zero wall count is included only for Native-admitted valid inputs;
   rejection/crash/assertion/timeout is preserved, never portable parity PASS.
7. Original Orca is the runtime oracle; only generated-by whole-line normalization
   is allowed. Inputs may enter the repository, Native streams/receipts/diagnostics
   stay /tmp. No weakened tests, guard assertions, retries-to-PASS or legacy fallback.

## Source identity and destination

Read source SHA256 (current admitted source tree):

- PrintApply.cpp: a067414959c475d23f782ebac9f5a185b63012304aced66974726d95a95daf20
- PrintObject.cpp: 7efa9c467c6f32a46008a167d525458d582859f76706a5be6412a84d7c6ab589
- PrintObjectSlice.cpp: d1aece12e7189353aa9a5d8ae6b1f0f320087c0d2574d7da64fe372eca076021
- Layer.cpp: 2d429738a2a92e09ff760b595fa15dfcf4b1039c61ced7bfbaef4513ed12b86c
- PerimeterGenerator.cpp: 45e0a6c56df0acf4e66ae82ae7a4aa4792dc558fb263a12b1211438de47b7de6

Destination: existing ares-core effective_config/candidates.rs,
options/region_options/merge.rs, project/region_graph.rs, project_slice/
volume_regions.rs, region_slices/complex.rs and existing perimeter compatibility/
classic/Arachne region consumers. The latter consume wall_loops at upstream
PerimeterGenerator.cpp:1225/2131, not an independently defined count policy.

The earlier epsilon wall-loop modifier attempt is a historical lead: Native
succeeded but Core's control hit its existing gate. It is not the new slice's
runtime RED or a general admission claim. Only source inspection and these two
documents have been added for this next slice; no tests/probes/production edits.
