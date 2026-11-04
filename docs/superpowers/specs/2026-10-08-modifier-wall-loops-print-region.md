# Modifier-volume wall_loops PrintRegion slice

Status: proposed. Historical epsilon admission failures are scope leads, not the
independent runtime RED for this slice.

## Upstream owner and Rust destination

This is a bounded libslic3r rewrite slice, not an Ares pipeline feature:

- PrintObject.cpp:3664–3710, region_config_from_model_volume: modifiers start
  from the parent's PrintRegionConfig, then apply volume/material overrides;
  no layer_range_config is applied again to the modifier. Model-part parent
  config already includes object, volume/material and final height-range overlay.
- PrintApply.cpp:958–1058, generate_print_object_regions: source-volume-major,
  range-minor traversal; reverse intersecting-parent traversal; config-distinct
  object-wide region registration; equal-config fallback to a model-part parent.
- PrintObjectSlice.cpp:338–398, complex slices_to_regions: route modifier slices
  using each active range's parent/region mapping, retaining the merged epsilon
  cursor. Fast comparisons remain exact.
- Layer.cpp:139–211, is_perimeter_compatible/make_perimeters: wall_loops must
  distinguish incompatible perimeter groups; geometry cannot be silently merged
  using a parent's loop count.
- PerimeterGenerator.cpp:1144/1225 and2093/2131: classic and Arachne consumers
  use the selected region's wall_loops plus existing extra_perimeters.

Rust ownership stays inside ares-core:
`project/effective_config/candidates.rs`, `options/region_options/merge.rs`,
`project/region_graph.rs`, `project_slice/volume_regions.rs` and existing complex
region slicing/perimeter consumers. These already carry the upstream concepts;
verify and reuse them rather than introducing a second region registry or flow
pipeline. The blanket modifier wall_loops rejection in candidates.rs is a
**temporary compatibility boundary**, not upstream semantics. Remove only that
key after independent RED; retain all other deferred-option boundaries.
If runtime output reveals a missing source-owned consumer, port only its cited
wall-loop behavior; a broader geometry/ordering prerequisite requires stopping
and submitting a separate spec, not enlarging this slice silently.

## Included behavior

- Legal parameter-modifier volume wall_loops values: distinct positive counts,
  equality with the parent, and zero where Native admits the complete input.
- Both existing classic and Arachne consumers. Neither is claimed merely because
  the generic configuration merge accepts the key.
- Parent inheritance and modifier override precedence across explicit-height
  ranges: the parent range overlay is inherited, then the modifier's own value
  wins. An equal value may share a region outside the range but become a distinct
  region inside it. No object-global mutation or per-layer cloned registry.
- Ordered overlapping/nested modifiers of this one option, including no-overlap
  controls, using the existing bbox/parent-chain/source-volume ordering.
- Physical-layer schedule/template count, complete G-code motion, M73, timing,
  flow/configuration and ordering are all part of the whole-output oracle.

Deferred: top/bottom shell overrides, feature-filament overrides/multifilament,
MM/fuzzy painting, material metadata expansion, spiral+region ranges, adaptive
or painted profiles and active ZAA. No additional region-option family, API,
crate, fallback, golden Native stream or general capability-policy rewrite.
LayerRanges construction, profile generation and epsilon/fast lookup semantics
must not change as an incidental fix.

## Source-first TDD and runtime admission

Before implementation, failure inventory: modifier config ignored; wrong overlay
order; modifier range reapplied after its volume; parent config mutated; equal
configs assigned duplicate IDs; nested modifiers choose wrong parents; distinct
wall counts merged as compatible perimeters; outside-range leakage; wrong actual
slice-Z routing; classic/Arachne divergence; physical layer/template count doubled;
zero-count empty paths mishandled; hidden geometry, progress or configuration
changes; Native failure accidentally classified as parity PASS.

Plan:

1. Derive legal source inputs from the admitted cube/modifier archive. Preserve
   original/input identities, exact settings, source/binary/resources/seed,
   argv/status and full streams outside the repository. Start with no-modifier/no-overlap and
   equal-parent controls, each backend separately. Do not assume an epsilon-era
   legal Native stream remains a new slice's admission result.
2. Run an independent Native-exit0 complete-output witness against unchanged
   merged Core, with a half-cube modifier wall4 versus parent wall2. Ares success
   plus whole-byte equality is the assertion: existing rejection is a runtime
   RED, never an expected UnsupportedProjectFeature test. Preserve failed Native
   admissions as failures/inconclusive, without exceptions or retry-to-PASS.
3. Commit the failing public byte-API E2E/input before production changes. Extend
   one independently observed case at a time: explicit height0.2 range wall3,
   modifier wall4 overriding it; equal wall2 becoming range-dependent; nested
   wall4→wall5; legal zero; outside overlap. Require complete Native output for
   both classic and Arachne before claiming backend coverage. If Native rejects,
   crashes/asserts or times out, disclose it and do not invent portable behavior.
4. Remove the wall_loops-only temporary gate and port any independently exposed
   source-owned missing behavior within the destination above. Keep original
   fixtures/assertions and obtain full-output GREEN.
5. Run unchanged epsilon/Stage1/Stage2/zero-height controls, a fresh ordered same-
   main required-Native seeded baseline/candidate pair, input-bound full common
   stream audit, required-Native workspace nextest, clippy/fmt and core+WASM check.
   Native test admission
   uses shared fail-closed native_environment. Only the entire generated-by line
   may be normalized; never motion/M73/Z/region/configuration/template lines.
6. Conclusion-only ARD. Historical evidence must retain its actual commit binding.

Exit criteria: source override/parent/range precedence, config-distinct identity
and perimeter compatibility proved through unchanged whole outputs in both
backends; all existing common streams unchanged; no Native artifact committed;
no claim beyond this wall_loops source boundary. No new runtime work exists yet.
