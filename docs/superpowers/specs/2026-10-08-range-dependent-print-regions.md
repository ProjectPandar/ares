# Stage2: source-owned region-override object ranges

Status: proposed. Builds on merged Stage1 explicit height-only ranges.
**This spec calls region ranges Stage2; it does not cover the previously
proposed painted-profile Stage2.**
Architecture: [range-region ARD](../../architecture/2026-10-08-range-dependent-print-region-boundary.md).
This is a libslic3r Rust rewrite, not an Ares-designed segmentation feature.

## Source-to-Rust ownership

| Upstream owner | Included behavior | Rust destination |
|---|---|---|
| PrintApply.cpp:342-406 LayerRanges::assign/config | sorted first-range-wins overlaps, EPSILON gaps, default tails, range/config association | existing project/effective_config/layers.rs |
| PrintObject.cpp:3664-3710 region_config_from_model_volume | defaults→object→volume→material→range overlay, modifier-parent inheritance, feature-filament mask/clamp | existing options/region_options + effective_config range candidates |
| PrintApply.cpp:958-1058 generate_print_object_regions / get_create_region | per-range volume participation and modifier parent chains; object-wide config-equality region reuse and insertion IDs | project/region_graph.rs + project_slice/volume_regions.rs |
| PrintApply.cpp update_volume_bboxes and region bounds helpers | range-specific transformed bounds, compensation, overlap participation | existing project/region_graph/bounds.rs + project_slice/volume_bounds.rs |
| PrintObjectSlice.cpp:153-241 slicing ranges / :244-267 range lookup | range-bound volume slicing, actual float slice_z and boundary traversal | existing raw_intersections/region_slices orchestration |
| PrintObjectSlice.cpp:269+ slices_to_regions | one-region fast path, active volume overlap detection, complex subtraction/modifier routing per range | region_slices.rs and region_slices/complex.rs |

## Existing scaffold to replace (not another pipeline)

Archive parser already stores sorted ranges with optional height and region
options. Effective config builds multiple ResolvedLayerCandidates for real
region overrides; Stage1 excludes pure height-only ranges because they produce
identical PrintRegion configuration. `project::region_graph::build_region_graph`
already creates object-wide equal-config IDs and range-specific records with
source volume index/parent/region ID; its object_region_graph helper already
computes per-range bounds.

But `project_slice/raw_intersections.rs::validate_layer_ranges` still rejects
region overrides; `volume_regions.rs::build_volume_region_graph` requires exactly
one candidate, passes only full-object boxes, pops one range and discards range
identity. `region_slices.rs` and PendingRegionSlices carry one record list for
all layers, and complex.rs consumes that same list. These are the exact Rust
partial-shell limitations to replace. Keep geometry/mesh slicing, object-wide
region table, PostRegionPrintObject and downstream flows unless the source port
requires the associated range lookup. Do not introduce a parallel region registry
or resolve options from scratch while emitting G-code.

## Included implementation slice

1. Preserve source LayerRanges partition/precedence and existing Stage1
   height-only mapping. Supply source range index/config to candidate resolution;
   range overrides apply after volume/material for model parts; modifiers inherit
   their parent then own options, not the range reapplied a second time.
2. Build one object-wide config-distinct region table while retaining every
   range's participating volume records. IDs follow source first registration
   order, volume-major then range order; config equality—not interval identity—
   controls reuse. Current Rust linear equality dedup is adequate for this slice;
   do not add speculative hash abstractions or per-layer option clones.
3. Port range-specific bbox participation. Do not use one full-object bbox to
   manufacture modifier overlap in an interval where the volume is absent.
   Existing transformed_mesh_range_bboxes is the partial port to reuse/audit.
   Retain stable source-volume/occurrence identity when connecting raw slices,
   bounds and range graphs. No extra geometry intersections for config alone.
4. Route each **actual slice_z** to its source range mapping. Use upstream float
   z/EPSILON and exact-boundary rules from the appropriate fast/complex path;
   do not substitute print_z, midpoint rounding or nominal layer index. Variable
   layer heights from Stage1 remain independent of region assignment.
5. Carry selected range mapping into both the fast slice shuffle and deferred
   complex-layer path, preserving negative volumes, modifier parent chains and
   same-region merging. Allocate each region's layer array once for object-wide
   layer count; inactive interval layers are empty, not absent or renumbered.
6. Remove raw range rejection for explicit-height + region-override
   collections once Native RED→GREEN proves the entire path. Preserve mixed/
   missing height UB boundary, unrelated centering/shared-mesh gates, painted
   profile and activeZAA rejection. Do not enable unsupported option families
   merely because region options were parsed.
7. Narrow/replace old range rejection tests before code; legal explicit-height
   region-range cases become positive Native E2E. Preserve pure-None/mixed
   rejection controls. Existing single-candidate public slicing controls continue
   unchanged; no compatibility wrapper with fabricated empty ranges.

## Included/deferred behavior and limits

Included: valid explicit-height range collections, scalar already-supported
PrintRegion option overlays, overlaps/gaps/default tails, config equality reuse,
range-bound selection and existing model/negative/modifier routing where Native
controls establish legal behavior. Primary witness uses wall_loops and constant
height so layer geometry changes are attributable to region routing; subsequent
combined height+region witness proves Stage1 interaction.

Deferred: painted profiles, MM/fuzzy painting region expansion, activeZAA,
SlicingAdaptive automatic profile generation, incremental front-end config/cache
invalidation and UI undo/redo. Existing supported modifier behavior must not
regress. Full upstream extruder/filament-changing range behavior is not silently
claimed from a wall_loops witness: it needs an independently applied multi-
filament Native case and tool-ordering/flow parity before broad acceptance.
If an already-parsed option crosses an unported downstream capability, keep its
existing capability boundary or explicitly request a follow-up source slice;
never mask it by dropping the override. Source-cite any newly discovered blocker
and return for review instead of growing an unrelated pipeline.

Missing layer_height is a real upstream UB input for current range-generated
profiles, not a default rule. All successful Native witnesses below provide it
on every range. Do not launch known-UB pure-None/mixed controls as numerical
oracles, and do not change their existing Rust rejection in this stage.

## TDD plan and independent RED (not yet run)

Public seam: bytes→ares_core::slice_project_plate_validated→full G-code.
Native CLI slice of the identical archive is independent oracle; config export,
region/layer diagnostics are supplemental artifacts, never substitutes for bytes.
Failure modes to cover before the relevant production change:
wrong object-order association; wrong overlap/gap precedence; range overlay
before volume/material; duplicate configs receiving new IDs; wrong first/slice_z
boundary; ghost modifier bbox; wrong complex-layer range; region flow/tool choice
not following selected configuration; loss of empty layers/stable identities.

1. Derive from a known Native-admitted
   tall single-model FDM archive; preserve every existing member except added
   range XML. Constant nominal/first height, explicit legal layer_height on range,
   interior wall_loops override clearly different from default. Use enough layers
   below/inside/above to verify switching both directions. Prefer >few-mm height
   rather than the short Stage1 seam fixture so wall-loop changes are observable.
2. Existing seeded2.4.2 Native must exit0 and emit full stream; prove loaded range
   association and changed wall-loop paths inside but not outside. Bind input,
   actual argv, executable/resources/source/seed, command/status and complete
   Native bytes. Current Ares must fail its existing range capability boundary
   on the **same bytes**. Commit failing full-output E2E before production edits.
3. Minimal source GREEN for primary, then new independent Native witnesses and
   prewritten failing tests per vertical slice: adjacent boundary/gap/overlap;
   same option value/config reuse; object versus volume/material versus range
   precedence; mixed variable height+wall settings; two objects with different
   ranges; negative/modifier whose overlap exists only inside one range.
   Multi-filament routing, if included, requires a separate explicit Native
   witness before changing that path; no assumption based on scalar parity.
4. Every successful fixture compares whole Native/Ares bytes with only the whole
   generated-by line normalized. Retain all inputs/statuses/raw streams/hash
   records. Native rejection, crash or timeout is disclosed non-PASS, not
   fixture filtering, repaired input, retry-to-PASS or Rust crash emulation.
5. Replace obsolete unsupported assertions before code. Prefer integration/E2E
   over new internal unit tests; any unavoidable isolated test first documents
   failure modes and derives expectations from independent upstream proof.

## Acceptance gates

Stage1 four ranged full-output fixtures, mixed control and seam-height control
remain GREEN; no-range baseline outputs unchanged. New legal region-range cases
match complete bytes across boundaries and modifiers. Fresh same-round exact
main base/candidate full seeded plus targeted cases, workspace nextest,
clippy -D warnings, cargo fmt, browserWASM compile, and same-round official
comparison. Base absent new tests
is disclosed, not counted coverage. New Ares-only failures block completion.
Report complete common raw-byte preservation and each legitimate changed/new
stream, source owners, first differences and residual non-PASS outcomes.

Use independent targets/caches. Debug only existing Native with
`nix shell nixpkgs#gdb -c gdb ...`, never rebuild OrcaSlicer for diagnosis.

## Current evidence

Read-only source investigation; no Native probes, fixture/test or production
Rust changes. PrintApply.cpp and PrintObjectSlice.cpp match the admitted2.4.2
seeded source by SHA256 (ARD). Stage1 is merged; Stage2 is proposed.
