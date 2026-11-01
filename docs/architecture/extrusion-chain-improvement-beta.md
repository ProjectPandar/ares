# Extrusion entity chain improvement — slice A2

Authority: OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`ShortestPath.cpp::chain_extrusion_entities_impl:1429–1462`, with
`NearestPoints:1097–1190`, `improve_chain_by_local_search:1198–1403`,
`chain_nearest_next:1406–1425`. Rust destination:
`project_slice/perimeters/classic/shortest_path/entity_chain.rs` and private
improvement modules. Existing greedy entity chaining is a temporary shell
missing the upstream improvement phase.

## Caller inventory and non-negotiable routing

- `GCode.cpp:8952,8962`: infill/ironing outer entities and sortable collection
  children use a reference start point; improve=true.
- `GCode.cpp:9021`: support extrusion uses a reference start point;
  improve=true. Rust `motion/shared_reversals.rs::chain` serves both infill and
  support, preserving its existing reversal ledger while improving the chain.
- Rust `FillExtrusionCollection::chained_path_from` is called only from
  `gcode_emit/motion/layer.rs::emit_infills`; despite its name, this is
  GCode.cpp:8962's explicit reference-start reordering of collection children,
  not ExtrusionEntityCollection.cpp:100. Use improve=true and respect no_sort.
- `ExtrusionEntityCollection.cpp:100` calls the non-improving pointer overload.
  No global optional-start behavior change; keep that distinction rather than
  infer overload ownership from the Rust helper name.
- `Support/TreeSupport.cpp:1865`: no-start support preparation stays
  improve=false. Rust `tree/non_organic/toolpaths.rs` calls the optional-start
  entry with None and must retain that behavior.
- Generic geometry/loop/polyline chaining and extrusion-path materialization
  are not entity reference-start overloads and remain unchanged.

## Included behavior

After the unchanged constrained greedy chain, improve=true chooses exact
chaining for up to eight entities. Larger chains use the source nearest-32
endpoint candidate grid, constrained 2-opt and or-opt moves, queue/sweep order,
50*n move bound, and SCALED_EPSILON acceptance. With a start point, compare the
nearest-next candidate against the locally improved greedy chain, improve and
select it only under the source travel comparison. Keep index ties, reversal
constraints and loop orientation behavior. Do not replace the bounded spatial
candidate search with quadratic scans.

A1 ports complete `FillBase.cpp::join_close_ends:3001–3117` and its
non-Arachne sparse/no-sort gate. A1+A2 must close the existing ordered honeycomb
RED; geometric similarity or unordered segment equality is not acceptance.

## Deferred and verification

Crosshatch connector arc ordering (`FillBase.cpp:1880–1897`) remains slice B.
No changes to cooling, travel formatting, connector trim flags or thresholds.
No redefinition of optional-start or shared-reversal ownership in this slice.
Run the strict honeycomb RED, earlier owned witnesses, then the authorized
fill/support parity set, reporting inherited/harness failures separately.
Clippy must have zero warnings; no lint suppressions. No Native goldens or
observation binaries are committed. External evidence retained outside the
repository.

## Local qualification

A1+A2 closes the ordered honeycomb witness; hook, object-ID and Hilbert
witnesses also pass (4/4). The authorized fill/support inventory selects 421
parity tests by fill/support/bridge/pattern/anchoring/raft vocabulary, excludes
child-only entry points, comparator unit tests and unrelated sweeps. On the
same base/environment, baseline is 190 PASS / 231 FAIL; candidate is
204 PASS / 217 FAIL: 14 FAIL-to-PASS and zero PASS-to-FAIL. Improvements cover
honeycomb and 13 combined-infill/solid-clearance/flow-ratio/seam-gap controls.
The 217 remaining failures are not claimed fixed or byte-identical; the result
is a paired test-status comparison, not full-output parity for the collection.
No assertions or comparison rules changed. Final directed witnesses pass
again after verifying the G-code collection-child routing; rustfmt and
all-target Clippy pass with zero warnings.
