# Painted split Native comparison boundary

## Source owner and Rust destination

Upstream is `345efec64c7b95a05ce5cf96df0dde66a158835c`. Paths below are relative to `src/libslic3r/`.

- `MultiMaterialSegmentation.cpp::segmentation_by_painting`, beginning at2188, owns prepared layer polygons, facet projection, colored contours, side segmentation, top/bottom segmentation and merge. The side producer at2361–2437 partitions colored contours by prepared ExPolygon island, including its holes, runs the single-color or MMU graph path per island, and appends state output in island order. This is the source-owned candidate boundary, not a claim about the cause of the public failure.
- `MultiMaterialSegmentation.cpp::multi_material_segmentation_by_painting` at2471–2482 supplies ordinary MMU facets, state cardinality, effective options and `IncludeTopAndBottomLayers::Yes`.
- `PrintObjectSlice.cpp::apply_mm_segmentation` at1062 consumes the segmentation through real layer/region slices. Its call at1414 is the actual slicing consumer.

The exact private Rust candidates are `crates/ares-core/src/project_slice/region_slices/painted/mod.rs::sides` and `apply_project`, `painted/graph/mod.rs::segment`, and the existing `painted/apply.rs::apply` / `split_layer` handoff. Existing `projection`, `painted_lines`, `contour_color`, `preprocess`, `top_bottom` and `merge` modules provide the adjacent data boundaries. No new module, public API, geometry backend or independent Ares painting pipeline is introduced.

`PostRegionPrintObject`, `RegionSurface`, and the nested layer/state polygon vectors remain temporary representation shells around upstream `PrintObject`, `LayerRegion` and segmentation output. They must not become independent algorithmic definitions. This document permits a public comparison reproduction and source-boundary analysis only. It does not prescribe or authorize a production replacement.

## Public contract

The existing target is `crates/ares-cli/tests/orca_parity/paint_color.rs::paint_color_split`, through `check("painted-split")`. The fixed input manifest and preparer authenticate the original archive. Input identities are not expected Native output values.

The unchanged path is `runner/native_stream_case.rs::check` / `qualify` and `runner/native_api_cli.rs::NativeCase::{slice,compare}`. It requires two fresh Native exports for one input before the API/CLI leg, complete ordered Native equality after only the existing generator-line allowance, successful public API and actual CLI output, and full raw API/CLI equality. CONFIG, headers, time fields, M73, tools, roles, motion ordering and coordinates remain strict. No role-only, motion-only or geometry-only result substitutes for this contract.

`NativeCase::compare` records both API and CLI outcomes before its first comparison assertion. A failing assertion can prevent the later `native_stream_case::check` annotation step. Therefore the original `comparison.json`, `native-replay.json` and complete raw streams are the records to inspect. Do not infer a successful later annotation or raw-equality assertion merely because both raw files exist.

## Included analysis

1. Reproduce the original test on one unchanged Rust tree in both Native modes. Establish Native input/resource identity, successful fresh export and complete Native replay before interpreting any Ares difference.
2. Use the original ordered-byte comparator to identify the earliest failing public assertion and first complete-stream difference. Record its complete surrounding protocol and layer/role/tool context without filtering the acceptance comparison.
3. Map that difference to the actual producer/consumer boundary. For a possible painted side-segmentation difference, inspect the cited source dataflow and existing private Rust counterparts. Distinguish prepared ExPolygon islands and holes, colored-contour ranges, state-bearing graph inputs, side output, cap output, merged segmentation and actual applied region slices.
4. Treat upstream per-island segmentation and Rust's layer-wide `sides` call as a static difference to investigate, not a demonstrated cause. Establish the consumed input identity and responsible upstream operation before attributing the public mismatch. A first byte difference alone does not prove a geometry, graph, formatter or ordering cause.

No private runtime tracing is included. Native rebuilds, instrumented Native sources, inferior assignments/calls, new test hooks and private-entry tests are outside this boundary. Any private observation requires its own exact source/input/output protocol and separate authorization. No claim of internal phase equality can be made from complete G-code alone.

## Data boundaries that must remain intact

- Prepared retained layer indices and slice planes belong to the real PrintObject slice state, not to a replacement Z/count snapshot.
- Native side output is indexed by retained layer and facet state, including default state0. Top/bottom output uses its own state/layer axes before merge. Ordinary facet projection and strict slab facets are distinct consumers.
- An island is an ExPolygon with all its holes. Colored-contour membership follows the upstream EdgeGrid/colorization traversal, including the treatment of empty contours. Do not regroup by bounds, nearest material, sorted coordinates or output order.
- The source single-color branch examines the island's complete colored-contour range. The graph branch consumes that same range, removes multiple edges, prunes degree-one nodes and extracts state-bearing regions. The source's conditional face union and ordered island append are upstream behavior, not permission to normalize a comparison.
- State cardinality, shared region identities, transforms, nozzle and filament settings are source-owned inputs to this slice. A mismatch there must be identified at that owning boundary rather than repaired indirectly in graph segmentation or G-code.

## Deferred behavior

Production or test assertion changes, fixture/manifest/preparer edits, new expected outputs, independent graph or Voronoi redesign, shared region identity/order/membership changes, configuration/nozzle/mapping/flush behavior, painted ears, brim, exclusion outlines, support, arc-end formatting, side-cut/beam/fuzzy extensions, and estimator/M73/belt/cache work are excluded. Existing ports and their data are not rerun or changed merely as broad regression coverage.

If the first responsible seam lies outside the cited painted dataflow, retain the exact public failure and name the owning upstream function and Rust destination. Do not extend this boundary silently. A production port requires its own included/deferred source behavior and tests before code. The existing older painted specs describe their own source revisions and do not establish current runtime attribution.

## Tests

1. Use the existing `paint_color_native_preflight` with all its original inputs in both modes before any Ares leg. Preserve positive model XY/extrusion activity for both painted materials, original setting checks and Native replay equality. A failed Native/activity premise is a failure to resolve at that premise, not permission to weaken it or store a new count/coordinate golden.
2. Run the existing `paint_color_split` separately in each mode. Retain every command status/stdout/stderr, both complete Native exports, public API and CLI raw products, input and executable identities, comparison JSON and actual nextest exit. A successful capture wrapper is not a successful test.
3. Run existing `paint_color_unpainted` and `paint_color_leaves` as distinct controls on the same tree and environment. Preserve all their assertions and distinguish inherited failures from changes. These controls do not establish parity for other painting compositions.
4. For every comparison, bind aliases and repeated input/Native/API/CLI occurrences explicitly. Keep original runtime fields and streams. Do not collapse repeated executions or compare a different Native input, executable, mode or Rust base.
5. Keep the target name, all helpers/comparators, fixtures and production bytes unchanged throughout this analysis. Any added private checker is diagnostic only and cannot turn a failed public test into a pass. No expected-error inversion, tolerance, field mask, alternate export or legacy branch is permitted.
6. A later source port must rerun the original target and controls with complete ordered Native/API/CLI comparisons and raw API/CLI equality. Rustfmt, strict workspace/all-target Clippy and WASM compilation establish build properties only. Native Linux execution and WASM compilation do not claim browser, Windows or macOS runtime parity.

## Failure modes

1. A wrong input, executable, resource set, Rust tree or Native replay can resemble a geometry regression. Bind these identities before reading the first output difference.
2. A Native activity/count or export failure can occur before the strict comparison. Preserve that actual failure instead of labelling it a painted graph difference or changing a numeric golden.
3. A formatter, header or estimator difference can precede painted motion. Do not delete that prefix or call a later matching motion subset whole-output parity.
4. A layer-wide versus per-island static difference may be inactive for the input or may not explain the consumed output. Do not implement a speculative island fix from source inspection alone.
5. A contour range can lose a hole, default state, empty slot or traversal order. Preserve the upstream membership and state axes without bounds-based reconstruction or sorting.
6. Colored facets, graph input, side faces, caps, merged regions and applied slices are different stages. Comparing unequal stages or ordinary versus strict facets produces a false attribution.
7. A configuration/state-cardinality or shared identity mismatch must not be hidden by resizing state arrays, remapping tools or changing graph ownership. Stop at that source-owned input boundary.
8. API/CLI disagreement, failed CLI status or missing output remains independently visible even if the API comparison fails first. The existence of raw files is not proof that equality assertions ran.
9. A private diagnostic or successful build can coexist with a failing complete public comparison. Report the failed test and actual exit without parity credit.
10. New normalization, replacement output snapshots, conditional compatibility paths or unrelated port changes can conceal the mismatch. Keep the original strict contract and restrict any subsequent fix to the demonstrated upstream slice.
