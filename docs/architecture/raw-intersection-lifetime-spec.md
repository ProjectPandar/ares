# Mesh-local raw intersection lifetime

## Upstream boundary

This is a Rust rewrite slice of OrcaSlicer `libslic3r/TriangleMeshSlicer.cpp`
at `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `slice_make_lines`, lines603–635, dispatches facets to eligible slicing planes
  and canonically sorts each plane's intersections.
- `make_loops`, lines1616–1666, chains and repairs each layer independently.
- `slice_mesh`, lines1997–2038, owns the raw intersection vectors for one mesh,
  classifies its coplanar edges, constructs layer polygons and returns them.
- `slice_mesh_ex`, lines2142–2186, consumes the resulting layer polygons to
  construct and simplify expolygons.

There is no global raw-intersection capacity cap in these functions. Storage
scales with the supplied mesh and slicing planes; intersection cardinality is
not an admission rule. The multi-plane upstream implementation collects raw
lines for a mesh before per-layer loop construction: it does not promise that
only one layer's raw lines exist at a time.

## Rust destination and invariant

`crates/ares-core/src/mesh_slicer.rs` retains facet dispatch, eligible-plane
lookup and canonical line order, but removes `RawIntersectionBudget`, its
project-wide counter and the fixed one-million-line rejection. No larger cap,
configurable limit or replacement fallback is introduced.

`project_slice/raw_intersections.rs` currently materializes every object's and
volume's raw vectors before `chained_intersections.rs` materializes all chained
layers and `looped_intersections.rs` repairs them. These project-wide raw and
chained stages are temporary Ares scaffolding, not the upstream lifetime.
Replace them with mesh-local slicing and immediate per-layer chaining/repair.
Finish a volume's raw-line consumption before slicing the next volume. Store
only the resulting `LoopedLayer` values in `ProjectSliceState`; no raw-line
collection crosses that state boundary. Delete obsolete raw/chained wrappers
and their unused stage module rather than retaining compatibility shells.

This preserves the existing two-millimeter loop-repair radius, source-volume
identity, volume ordering, coplanar classification, mirrored winding, active
height-range filtering and layer ordering. Existing downstream slicing modes,
closing, simplification, region composition and G-code generation retain their
order and behavior. Other mesh-admission restrictions, coordinate validation
and the separate dense layer-slot preflight are outside this slice.

Core remains platform-neutral on browser WASM, Windows, macOS and Linux. It
performs no file I/O and adds no dependency. Triangle-to-eligible-plane dispatch
must remain; repeatedly scanning every triangle for every layer is not an
acceptable lifetime adjustment.

## Observable behavior

Use the tracked public stock project
`tests/parity/machine-acceleration-cache-rounding/stock-control.3mf` as the
control. Densify its stock mesh at runtime by conforming subdivision along a
horizontal axis without changing its boundary. Write the subdivided mesh into
a runtime 3MF, retaining the stock project settings. Do not commit generated
archives. Record subdivision parameters, mesh counts and a reproducible
intersection-count calculation demonstrating more than one million raw
intersections across its slicing planes.

Native must slice the runtime-dense project successfully. Compare its complete
G-code with the public API and CLI outputs in official and seeded modes using
the existing generator-only allowance. Preserve the Native replay and stock
control comparisons. The same case must expose the former fixed-cap rejection
before the implementation changes.

Measure peak resident memory for separate CLI runs with the dense case. State
the source and executable identities, runtime context and measurement method;
a natural process exit is not a substitute for successful slicing. Inspect the
Rust ownership path as well as RSS: process-wide peak memory also includes
archive loading, mesh indexing, retained polygons and later slicing stages.
Verify formatting, strict workspace/all-target Clippy and browser WASM
compilation after the change.
