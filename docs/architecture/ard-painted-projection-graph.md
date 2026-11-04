# ARD: source-faithful painted MMU as one consumed landing unit

Status: proposed; execution decisions recorded; paired SPEC follows.
Base 3e862584d86bd1e00f19f5b581773dd9e20fd6c2, upstream 8500fcdcc.
[Source boundaries and destination](painted-projection-graph-spec.md).

Default VD repair is integrated, but that does not supply the painted producer.
PostRegion and shared registry types remain temporary upstream PrintObject/
LayerRegion compatibility shells, not an Ares-owned painting pipeline. Because
partial producers lack the complete real consumption chain on main, source-order
checkpoints are documentary; production lands only as a fully consumed unit.

## Non-negotiable decisions

1. Port MultiMaterialSegmentation.cpp's projection, painted_lines/contour_color,
   MMU graph, top/bottom1181–1472, merge1825–1870, producer1955–2194/wrapper2197–2207
   and PrintObjectSlice.cpp apply874–1030/caller1207–1224 as ONE integration unit.
   Stage discovery/design notes retain source order. Branch stage commits may be
   separate, but each must cargo check on its own committed tree. No allow/expect/
   cfg parking, fake consumer, dormant public entry, unused facade or uncommitted
   later-stage build proof. Each implementation/document file <=400LOC; only the
   already-oversized roadmap retains its existing exception.
2. Use the actual post-conical/pre-compensation handoff before compensation.rs
   prepare_post_compensation142–151 for both simple and complex paths. Retain
   actual shared config/slicing registry, parent/volume/instance/range/filament
   identities and region replacement; never substitute staged print_apply records,
   a painted-only registry, discarded producer output or test-only core caller.
3. MMU carrier/ordinary selected-leaf exporter must be independent of support.
   Strict slab/support exporter remains distinct. No premature public admission,
   parser bypass, empty helper or side-only feature while top/bottom/merge/apply wait.
4. Acceptance RED is public NativeCase complete ordered output through existing
   native_stream_case::check: Native double replay, validated generator-only line,
   API success/actual CLI success, full comparisons and raw API==CLI. Frozen
   leaves/split plus unpainted control retain global Native-first admission and
   positive model XY/extrusion for both painted materials, not merely T0/T1 tokens.
   A success-required test failing at the current paint_color rejection is LEGAL
   RED, not an inverted error-PASS. No private-entry phase acceptance tests.
5. Projection/MMUGraph/side Native GDB observation is only read-only GREEN-time
   diagnostic localization, never an acceptance gate. Current base's seeded-only
   ctor observations and old runs retain actual hash/identity;
   they do not become phase GREEN or official VD/DWARF equivalence.
6. MUST port Point.hpp506–659 ClosestPointInRadiusLookup radius multimap semantics:
   duplicate-preserving equal_range, source corner/neighbors/ties, find strict
   versus find_all inclusive radius, erase-first-equal. Do not reuse overwriting
   PointHashMap or substitute scans/nearest-color/index algorithms. Preserve f32
   projection, cast/round, ordered Grid visits/arcs/left-most/rollback. Reuse the
   admitted effective Voronoi wrapper unchanged, never raw fallback or vendor fixes.
7. Exact complete integration tip receives fmt, workspace/all-target Clippy with
   -D warnings, core/vgcode/adapter wasm32 and actual official+seeded canonical
   full parity; old/phase hashes are not credit.
   SPEC precedes RED; an
   accepted RED and separate GREEN precede implementation.
   No suppression, field ignore, tolerance or fallback.

Cut/width-depth1144–1168, painted+beam, fuzzy, UI/cache and unused debug/legacy
helpers stay deferred; active source behavior cannot be a zero-only stub.
Preserve ordinary/strict facet separation and unpainted interlocking placement,
keep unsupported compositions fail-closed, and stop on any FINAL discrepancy.
Pure Rust byte/data core remains browserWASM/Windows/macOS/Linux portable;
no runtime filesystem/debugger/native/viewer/terminal dependencies.
