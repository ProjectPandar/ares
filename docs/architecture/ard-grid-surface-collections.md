# ARD / SPEC: Grid per-surface extrusion collection materialization

## Source ownership and non-negotiable contract

Upstream OrcaSlicer revision `8500fcdccaa10b5099ac20d252af3a7c560046f1`:
`src/libslic3r/Fill/Fill.cpp:1326-1357`, `Layer::make_fills`, iterates each
expolygon of a grouped `SurfaceFill` and invokes `fill_surface_extrusion`
separately. `Fill/FillBase.cpp:161-182`, `Fill::fill_surface_extrusion`, creates
one new `ExtrusionEntityCollection` per nonempty surface call and appends only
that call's generated paths. An empty surface call adds no collection.

This collection identity is observable: `GCode.cpp:5010-5068` assigns a whole
original collection to an island using its first point; `5434-5468` emits each
island's wall/fill phases. Do not repair an earlier collection-construction
mismatch by assigning flattened paths individually or changing island order.
Grid retains `no_sort=false` and non-reversible Oriented paths independently;
see [Grid ordering ARD](ard-grid-collection-ordering.md).

## Rust destination and temporary shell replacement

The owning destination is `ares-core::project_slice::fill_entities::grid`.
Currently `grid.rs:46-65` creates one collection after `polylines` concatenates
all grouped expolygons at `grid.rs:91-120`. That aggregation is an incomplete
compatibility shell around upstream per-surface materialization, not an
Ares-owned infill design. Replace only this lost collection boundary: retain
one collection per nonempty Grid expolygon invocation, in original order,
without changing generated vertices, flow, roles, reversal or sorting flags.

`fill_entities/surface.rs` also consumes `grid::polylines` as flat geometry.
Preserve that consumer's existing contract. No public API, new collection
metadata, registry, fallback, guard or generic configuration layer is needed.
The private core remains filesystem/terminal-free across browser WASM,
Windows, macOS and Linux; fixture I/O belongs only to the CLI parity adapter.

## Causal evidence

Qualified stock Creality K1 bridge, print_z0.8/upstream layer_id3:
Native `group_fills` and Rust Grid append consume one group containing the
same two ordered expolygons. Both generate identical complete six-point
polylines, first points `[-20891522,3764696]` and `[20891527,-3764691]`.
Native materializes two collections with entity counts `[1,1]`; Rust creates
one with `[2]`, assigned wholly to the left island. Right sparse infill is
therefore delayed until after left walls. Grouping and scanline geometry
already agree in this witness.

Repeatable external evidence (not committed Native output or probes)
contains Native/Ares live traces, exact vertex assertions,
reproduction commands and output-equivalence checks. Initial decoder/probe
failures are separately labeled and excluded from the final evidence.

## Scope and deferred behavior

Included: object Grid per-expolygon collection materialization, empty-call
behavior, original surface order and existing path semantics. Existing Grid
multiline algorithms remain unchanged, not a new multiline rewrite slice.
Deferred: group_fills/coalescing, other infill patterns, gap-fill algorithms,
support/raft surface behavior, island assignment/emission ordering, travel,
seam/rear, brim, scan algorithms, cooling, processor time and M73 generation.
No broad Ares pipeline, geometry workaround or comparator relaxation.

## SPEC → tests-only RED gate

Freeze two genuine stock exported projects without golden G-code: K1 bridge
as target and MK4 bridge as passing machine control. The MK4 control is not
claimed to isolate Grid granularity. Use unchanged `native_stream_case::check`
for each input: stable Native double replay before public API/actual CLI,
exit0, raw API/CLI equality and complete ordered bytes with generator-line-only
normalization. Register together under the existing Grid test group; keep
changed Rust modules below400 lines.

Run only the directed pair serially, in official and seeded
modes with separate fresh caches/artifacts. Expected RED: K1 fails strict
output comparison; MK4 passes. Preserve K1's leading M73 difference (known
byte2844/line124) rather than filtering it; independent collection trace
explains its physical cause. Tests require success, not expected-error or
expected-failure assertions. No copied harness, magic output constants or
Native output committed. GREEN and full parity are
a separate slice.
