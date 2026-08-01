# Task 22O Implementation Plan: KSR-Reached Classic Perimeter Generator

## Objective and gate

Implement the approved Task 22O specification as the exact KSR-reached fixed
Orca Classic generator after Task 22N. Produce ordered fixed-coordinate wall
loops and paths, overhang splits, variable-width perimeter gap fill, internal
fill surfaces, and fill-no-overlap polygons. Preserve Task 22N and stop before
seam placement, infill paths, moves, extrusion scheduling, or G-code.

No production or tracked-test edit begins until the specification and this
plan are frozen into one exact frame and both independent document reviewers
approve it. Package 0 may create ignored oracle evidence after document
approval. Production remains blocked until the complete behavioral oracle and
its semantic manifest are independently approved.

## Working rules

Each package follows RED, minimal GREEN, focused verification, and independent
read-only specification and quality review before the next dependent package.
Expected values exist independently before production. Tests exercise public
archive loading or the crate-private stage boundary, never a fixture-specific
branch or a private helper shape alone.

Manual edits use `apply_patch`. Parallel agents receive disjoint leaf paths;
shared registration roots stay with the main thread. Preserve and exclude the
user's untracked `main.obj`, ignored oracle builds, downloaded Orca binaries,
and generated output from every stage and commit.

Use Cargo Nextest, never `cargo test`, as the default Rust test runner. Every
Rust source/test file remains below 400 physical LOC. Tests are separate `mod`
files. No source-splitting macro, generated textual Rust, `include_bytes!`
checkpoint, unsafe, broad lint allowance, fixture identity branch, reference
G-code production read, or old rectangular project fallback is permitted.

An activated deferred option returns `UnsupportedProjectFeature` with the
specific key during global preflight. Internal helpers trust preflight and do
not repeat impossible defensive checks.

## Planned path manifest

The current approved implementation surface is limited to these existing or
new paths:

- workspace `Cargo.toml` and `Cargo.lock` only if the qualified Voronoi
  dependency is accepted;
- `THIRD_PARTY_NOTICES.md` if either the qualified dependency or fixed BSL
  subset is accepted;
- `crates/ares-core/Cargo.toml`;
- `crates/ares-core/src/lib.rs`;
- `crates/ares-core/src/geometry.rs`;
- `crates/ares-core/src/geometry/expolygon.rs`;
- `crates/ares-core/src/geometry/polygon.rs`;
- `crates/ares-core/src/geometry/clipper.rs`;
- `crates/ares-core/src/geometry/clipper/boolean_ex.rs`;
- `crates/ares-core/src/geometry/clipper/offset.rs`;
- `crates/ares-core/src/geometry/clipper/offset/expolygon.rs`;
- `crates/ares-core/src/geometry/bounding_box.rs`;
- `crates/ares-core/src/geometry/line.rs`;
- `crates/ares-core/src/geometry/polyline.rs`;
- `crates/ares-core/src/geometry/polyline_clip.rs`;
- `crates/ares-core/src/geometry/polyline_clip/intersections.rs`;
- `crates/ares-core/src/geometry/polyline_clip/ordering.rs`;
- `crates/ares-core/src/geometry/medial_axis.rs`;
- `crates/ares-core/src/geometry/medial_axis/diagram.rs`;
- `crates/ares-core/src/geometry/medial_axis/inside.rs`;
- `crates/ares-core/src/geometry/medial_axis/validation.rs`;
- `crates/ares-core/src/geometry/medial_axis/chaining.rs`;
- `crates/ares-core/src/geometry/medial_axis/endpoints.rs`;
- `crates/ares-core/src/geometry/thick_polyline.rs`;
- `crates/ares-core/src/geometry/tests.rs`;
- `crates/ares-core/src/geometry/tests/bounding_box.rs`;
- `crates/ares-core/src/geometry/tests/polyline.rs`;
- `crates/ares-core/src/geometry/tests/polyline_clip.rs`;
- `crates/ares-core/src/geometry/tests/polyline_clip/intersections.rs`;
- `crates/ares-core/src/geometry/tests/medial_axis.rs`;
- `crates/ares-core/src/geometry/tests/medial_axis/diagram.rs`;
- `crates/ares-core/src/geometry/tests/medial_axis/branches.rs`;
- `crates/ares-core/src/geometry/tests/thick_polyline.rs`;
- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/checkpoints.rs`;
- `crates/ares-core/src/project_slice/perimeters.rs`;
- `crates/ares-core/src/project_slice/perimeters/types.rs`;
- `crates/ares-core/src/project_slice/perimeters/classic.rs`;
- `crates/ares-core/src/project_slice/perimeters/classic/preflight.rs`;
- `crates/ares-core/src/project_slice/perimeters/classic/prelude.rs`;
- `crates/ares-core/src/project_slice/perimeters/classic/top_surfaces.rs`;
- `crates/ares-core/src/project_slice/perimeters/classic/onion.rs`;
- `crates/ares-core/src/project_slice/perimeters/classic/hierarchy.rs`;
- `crates/ares-core/src/project_slice/perimeters/classic/traversal.rs`;
- `crates/ares-core/src/project_slice/perimeters/classic/overhang.rs`;
- `crates/ares-core/src/project_slice/perimeters/classic/gap.rs`;
- `crates/ares-core/src/project_slice/perimeters/classic/fill.rs`;
- `crates/ares-core/src/project_slice/perimeters/classic/types.rs`;
- `crates/ares-core/src/project_slice/perimeters/extrusion.rs`;
- `crates/ares-core/src/project_slice/perimeters/extrusion/types.rs`;
- `crates/ares-core/src/project_slice/perimeters/extrusion/coverage.rs`;
- `crates/ares-core/src/project_slice/perimeters/extrusion/ordering.rs`;
- `crates/ares-core/src/project_slice/perimeters/variable_width.rs`;
- `crates/ares-core/src/project_slice/task22o_oracle.rs`;
- `crates/ares-core/src/project_slice/tests.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/oracle.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/task22n_synthetic.bin`
  (delete);
- `crates/ares-core/src/project_slice/tests/perimeters/classic.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/classic/preflight.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/classic/prelude.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/classic/top_surfaces.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/classic/onion.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/classic/hierarchy.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/classic/overhang.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/classic/gap.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/classic/fill.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/classic/oracle.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/classic/fixture.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/classic/fixture/archive.rs`;
- `crates/ares-core/src/project_slice/tests/perimeters/classic/fixture/vectors.rs`;
- `crates/ares-wasm/Cargo.toml`;
- `crates/ares-wasm/src/lib.rs`;
- `crates/ares-wasm/tests/browser/task22n-edge-vectors.mjs` (delete);
- `crates/ares-wasm/tests/browser/task22n-vectors.mjs` (delete);
- `crates/ares-wasm/tests/browser/task22o-vectors.mjs`;
- `crates/ares-wasm/tests/browser/task22o-exports.mjs`;
- `crates/ares-wasm/tests/browser/server.mjs`;
- `crates/ares-wasm/tests/browser/project-slice-page.mjs`;
- `crates/ares-wasm/tests/browser/project-slice.spec.mjs`;
- `.github/workflows/tier1.yml`;
- `docs/architecture/option-parity-v4.md`;
- `docs/roadmap.md`; and
- this Task 22O specification and plan.

The ignored `.superpowers/sdd/task22o-oracle/` tree is evidence, never staged.
No generic directory wildcard authorizes unrelated edits. Each package freezes
an exact leaf manifest before RED. If implementation needs another path or a
listed file would reach 400 LOC, amend and refreeze both documents and reacquire
both approvals before editing that path.

## Package 0: fixed behavioral oracle and document freeze

Verify baseline identities, fixture hashes, Task 22N checkpoint bytes, fixed
source blobs, and the official v2.4.2 portable release digest. Create a detached
ignored fixed-source worktree and an out-of-tree instrumented Orca build that
exports only after the `PrintObject::make_perimeters()` TBB join and before its
completion flag. The exporter iterates objects, layers, and regions in fixed
vector order; it never writes from parallel `LayerRegion::make_perimeters()`
workers and does not mutate generator state. The separately qualified runtime
below changes only scheduling while oracle mode is active.

The ordinary unchanged-scheduler control is retained as a required RED:
4,842,892-byte primary payloads with SHA-256
`fe2f6523772f175484a93fad3899e9fa35a2ee08d6ce89939f08d1289284a78e`
and
`b33bbe42c5ef9a0a8d30dade183f3cde7417f5ab52af59e708ab6fcba5464bb3`
have equal structure, identities, and totals but 8,158 differing coordinate
fields. Do not select either output or repair this by sorting, normalization,
field deletion, or run selection.

Add isolated `Ares22OOracleRuntime.hpp/.cpp` files so the existing exporter
remains below 400 physical LOC. Patch fixed `src/OrcaSlicer.cpp` to construct an
explicit RAII runtime guard as the first statement of exported
`orcaslicer_main`, after `LoadLibraryExW` has returned and before argument
conversion, `CLI` construction, or any slicing work. The env-on guard captures
a nonempty startup `ORCA22O_PAYLOAD_PATH` exactly once. Before spawning any TBB
worker, it calls unchanged fixed
`set_current_thread_name("orcaslicer_main")` on the calling main thread. This
reproduces fixed `CLI::run:1191` early and satisfies fixed `Thread.hpp:14-15`'s
requirement to initialize Windows thread-description lookup on the main thread
before workers call it concurrently. It neither reads arguments nor constructs
`CLI`; the later fixed CLI call may repeat the same name.

The guard then calls unchanged fixed
`name_tbb_thread_pool_threads_set_locale()` synchronously at ordinary arena
concurrency, before constructing scheduler control. This completes fixed
`Thread.cpp:222-246`'s arena-sized worker naming/locale barrier and sets that
function's fixed local `initialized` state before fixed `Print.cpp:2181` can
call it. These two fixed calls are the only pre-control operations after path
capture. They may only initialize thread naming, name workers, and set worker
per-thread C locales; they run before model, Option, argument, CLI, or slicing
access. An exception produces the dedicated nonzero fail-closed exit. A call
that does not return is boundedly terminated by the external supervisor and
remains negative evidence. The later pool call in `Print::process` must be an
immediate no-op.

After successful priming, the same guard constructs a process-wide
`tbb::global_control(max_allowed_parallelism, 1)`, verifies the active value is
1 before argument conversion or `CLI().run`, spans every argument-derived
algorithm and the complete `CLI().run`, and is destroyed before
`orcaslicer_main` returns. Guard construction, main-thread naming, priming,
control construction, and destruction therefore all run outside the DLL loader
lock. The exporter uses only the captured path and cannot be enabled later. The
env-off guard owns no control, performs neither early naming nor priming, and
reads no scheduler state. Do not modify
`utils.cpp`, `Thread.cpp`, `Print.cpp`, or any scheduler source, and add no
legacy TBB fallback. The allowed isolated fixed-source instrumentation paths
are exactly
`src/OrcaSlicer.cpp`, `src/libslic3r/CMakeLists.txt`,
`src/libslic3r/PrintObject.cpp`, `src/libslic3r/Ares22OOracle.hpp`,
`src/libslic3r/Ares22OOracle.cpp`,
`src/libslic3r/Ares22OOracleRuntime.hpp`, and
`src/libslic3r/Ares22OOracleRuntime.cpp`.

Define `ARES22O` wire v1 before Rust production exists. Its exact little-endian
envelope is `ARES22O\0`, `u64 N length`, exact released `ARES22N` bytes,
`u64 payload length`, exact fixed-Orca payload, then EOF. The payload begins
`ORCA22O\0`, `u32 version=1`, then the structured body. A neutral ignored packer
owns the envelope and predecessor; the fixed-source exporter owns only payload
bytes. That payload contains complete object/layer/region slots, nested
collections, loops, paths, roles, inset depths, coordinates, width/height/
volume bits, gap entities, fill surfaces, and fill-no-overlap polygons. The
parser validates complete inner N and payload frames, binds their identities,
and requires each declared boundary plus outer EOF. The encoder must be
deterministic and use canonical enum values.

Retain the superseded unprimed env-on KSR run as required negative evidence: it
consumed only 0.25 seconds of CPU, emitted no payload or G-code, and was
boundedly terminated after 141.808 seconds. Do not retry, select, overwrite, or
count it toward qualification. Its source and binary manifests remain attached
to the failed run.

Rebuild the Release slicer from the exact fixed commit and the newly approved
seven-file instrumentation manifest. Use a new clean slicer build directory
after the priming patch; dependency outputs may be reused only when their hashes
and successful build ledger are retained. Verify fixed `Thread.cpp`,
`Thread.hpp`, and `Print.cpp` remain byte-identical to commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

First run the rebuilt binary with `ORCA22O_PAYLOAD_PATH` explicitly absent,
using a fresh datadir clone and output directory. Require successful G-code
completion and absence of the payload path, its temporary path, and any oracle
sidecar. This proves the new binary's env-off path remains inert; it does not
require byte equality with another inherently nondeterministic G-code run.

Then run two fresh qualified processes on the supplied archive and on every
supported synthetic oracle-payload case. Set a distinct absolute
`ORCA22O_PAYLOAD_PATH` before each process launch, use distinct output
directories, and use fresh clones of one content- and metadata-hashed datadir
base. Every oracle-payload case is a deterministic 3MF with all Options inside
the archive and its own archive and N hashes. Require the runtime's fail-closed
active-parallelism check, complete payload parsing, exact payload EOF, complete
`ARES22N` parsing and identity binding, outer EOF, byte-identical payloads, and
byte-identical composite wires. Freeze their common length, SHA, and semantic
totals only after both runs pass.

Every qualified run must also finish G-code. Record raw and timestamp-only
G-code hashes and diagnostic differences, but never use G-code equality to
accept or reject the payload. Retain the ordinary-scheduler pair, official
portable runs, and historical log as negative provenance evidence. This
deterministic qualification does not weaken the later exact Ares-to-reference
G-code goal.

Required supported synthetic oracle-payload cases are:

- precise width-average versus spacing-average;
- one wall, two walls, and collapsed inner wall;
- dynamic top-one-wall split with and without lower support;
- normal and smaller external width;
- contour, hole, nested island, and single-contour/single-hole traversal;
- fully supported, fully unsupported, and alternating supported path spans;
- `raft_layers=0`, proving layer zero is unsplit and layer one is the first
  overhang-split layer;
- zero and positive gap speed independent of `gap_fill_target`;
- zero and positive gap-length filtering;
- first, middle, and top fill-overlap modes.

Required Ares-only preflight-probe definitions cover every activated deferred
mode named by the specification. Each archive changes exactly one deferred
Option or topology condition, keeps every Option inside the 3MF, and records
the exact expected `UnsupportedProjectFeature` key. These cases are not launched
through fixed Orca. Package 0 records `ARES22N` as complete with length/SHA or
absent with the exact existing earlier gate, while `ORCA22O` payload and
`ARES22O` composite creation remain false. Existing positive `raft_layers` and
`multi_region_layer_slices` gates may record an observed key; every newly
specified Task 22O key records `observed_key: null` and
`execution_status: pending_package_b`. Package 0 does not implement or claim a
new rejection. Package B must turn each pending definition into a RED and then
an observed exact-key GREEN before any record advances. No probe authorizes
implementation of the deferred branch.

The ignored manifest schema is version 2 and contains these mandatory groups:

- `source`: fixed commit, tree, tag, and every cited blob identity;
- `instrumentation`: sorted allowed paths, per-file SHA-256, canonical combined
  source-manifest SHA-256, normalized complete diff SHA-256, capture seam, and
  proof that no other fixed source changed;
- `build`: exact command, generator, compiler/linker, CMake cache, oneTBB
  version, build log, executable SHA-256, DLL SHA-256, and dependency ledger;
- `ordinary_scheduler_negative_control`: both final payload length/SHA values,
  structural and semantic equality, 8,158 coordinate differences, input and
  datadir identities, and explicit `selected_run: null`;
- `execution_mode`: name
  `fixed-v2.4.2-primed-global-control-max-parallelism-1`, activation
  environment, guard timing
  `orcaslicer-main-first-statement-after-dll-load-before-args`, fixed
  main-thread naming function/source identity and call result, priming function
  and fixed source identities, priming timing
  `ordinary-arena-before-control-before-args-cli-slicing`, successful barrier
  return, later fixed-call no-op, requested value 1, observed active value 1,
  control lifetime `post-thread-locale-priming-to-orcaslicer-main-return`,
  algorithm lifetime `before-argument-conversion-through-cli-return`,
  `constructor_under_loader_lock: false`, `destructor_under_loader_lock: false`,
  `late_activation_allowed: false`, and `wire_schema_changed: false`;
- `unprimed_deadlock_negative_control`: source/build/run identities, bounded
  duration 141.808 seconds, process CPU 0.25 seconds, no payload, no G-code,
  bounded termination, and explicit `selected_run: null`;
- `env_off`: input/datadir/command identities, successful completion, G-code
  hash, and `payload_created: false`;
- `qualified_runs`: per input and run, absolute payload/output paths, process
  environment relevant to TBB, input and datadir hashes, payload length/SHA,
  composite length/SHA, parser result, semantic totals, G-code raw and
  timestamp-only hashes, and completion/stability evidence;
- `preflight_probe_definitions`: per probe, archive and Options-entry SHA, exact
  single active Option/topology delta, expected `UnsupportedProjectFeature`
  key, current `ARES22N` predecessor status (complete length/SHA or absent with
  an existing earlier gate), `fixed_orca_run: not_applicable`,
  `payload_created: false`, `composite_created: false`, observed key, rejection
  seam, and execution status; Package 0 records `pending_package_b` for newly
  specified Task 22O gates, and Package B replaces that status with the observed
  exact key and seam;
- `wire`: WIRE document SHA, payload version, packer source/binary SHA, exact
  parser/EOF result, predecessor SHA, and identity-binding result; and
- `reviews`: exact manifest SHA reviewed by both reviewers and their verdicts.

The behavioral payload remains `ORCA22O` version 1. Manifest version 2 records
the execution qualification without contaminating Ares behavior bytes.

Before tracked implementation, two fresh reviewers inspect the same qualified
payloads, composite wires, source manifest, diff, build, and run manifest. One
checks the fixed-source field mapping, first-statement guard, main-thread-first
thread-description initialization, unchanged thread-pool priming barrier,
post-priming control lifetime, loader-lock exclusion, active-parallelism
fail-closed gate, capture seam, and serialization completeness. The other
checks source/build provenance, env-off behavior, datadir isolation, exact
payload/composite repeatability, predecessor binding, and absence of
fixture/reference-derived expectations. Neither reviewer may accept sorting,
normalization, run selection, or G-code byte equality as payload evidence.
Both must verify the deadlocked unprimed run is retained only as negative
evidence and that neither fixed `Thread.cpp` nor `Print.cpp` was modified.
Both reviewers also verify every preflight-probe archive, expected key, current
predecessor status, absence of O output, and honest pending or existing-gate
status; fixed-Orca execution is not evidence for a deferred probe. Package B
reviewers later verify every observed exact key and rejection seam.
Tracked tests retain only behavioral vectors and hashes, never source-pinning
assertions.

## Package A: output model and geometry qualification

Freeze an exact leaf manifest. Add compile-RED tests for the absent
fixed-coordinate line, polyline, ThickPolyline, nested extrusion collection,
loop role, inset depth, width/height/volume metadata, and complete Classic layer
output. Prove clone/equality/order without flattening logical loops.

Add behavior REDs for bounding boxes, point containment, opening/closing,
extrusion coverage, and open-polyline clipping. Expected ordered coordinates
come from Package 0 synthetic wires. Reuse released Clipper, simplification,
and chain-points code; extend only the exact missing source boundary.

In ignored tooling, compare `boostvoronoi 0.12.1` with the fixed oracle on the
approved segment corpus. Compare ordered cells/edges/source categories,
finite-primary selection, vertices, neighbor rotation, and endpoint widths.
Also run a minimal default and `wasm32-unknown-unknown` compile. If any semantic
or platform comparison fails, stop and amend the documents; do not add the
dependency or substitute another skeleton algorithm.

GREEN introduces only the approved output/geometry vocabulary and qualified
Voronoi seam. It does not yet run Classic processing.

If either the crate or a fixed BSL Voronoi subset is accepted, update
`THIRD_PARTY_NOTICES.md` with the exact component, source/version or fixed
source identity, owned Ares paths, copyright, and BSL-1.0 provenance. Reuse
`LICENSES/BSL-1.0.txt`; do not duplicate its license text.

Focused gates:

```text
cargo nextest run -p ares-core task22o_geometry
cargo nextest run -p ares-core task22o_output_contract
cargo check -p ares-core
cargo check -p ares-core --target wasm32-unknown-unknown
```

Independent package reviewers verify fixed-coordinate arithmetic, collection
shape, no public old-entity adapter, dependency qualification, errors only at
real boundaries, and all LOC limits.

## Package B: Classic preflight and prelude

Write public archive and crate-private REDs for the complete option inventory,
transactional preflight order, first/middle/top records, precise spacing,
smaller external Flow, lower support masks, sampled lower polygon series,
counterbore-none return, fuzzy-disabled no-op, 0.0024 mm simplify, bbox-center
surface ordering, loop-number derivation, and `raft_layers=0` printable/overhang
boundaries.

Mutation REDs must show that each supported value is loaded from the 3MF. A
public 3MF mutation to positive `raft_layers` must retain the existing
`UnsupportedProjectFeature("raft_layers")` result before any record advances;
Task 22O does not remove the capability gate or pretend to generate raft layers.
Activated deferred values fail with the exact option key before any record is
advanced. `gap_fill_target` mutation alone does not change prelude state or
disable gaps. Invalid geometry/scale errors preserve every Task 22N object.

For every `pending_package_b` probe definition, first freeze a RED showing the
exact key is not yet observed. GREEN must produce that exact key before any
Task 22O generator or checkpoint runs, leave `ORCA22O` and `ARES22O` absent,
and update the ignored manifest with the observed key, rejection seam, and new
review identity. Existing earlier-gate probes remain unchanged.

GREEN adds immutable validated Classic config and prelude state. It consumes
the four Task 22N Flows and typed resolved object/region/print Options; it does
not reparse JSON, consult filament maps, recompute Flow, or generate walls.

Focused gates:

```text
cargo nextest run -p ares-core task22o_preflight
cargo nextest run -p ares-core task22o_prelude
cargo nextest run -p ares-core task22n
```

## Package C: dynamic top-one-wall split

Write REDs from oracle vectors for upper-only, upper/lower, hole, minimum-width,
no-exposed-top, and KSR representative layers. Freeze `top_fills`,
`non_top_polygons`, and `fill_clip` as complete ordered polygon wires. Include
Option mutations for `only_one_wall_top`, `interface_shells`,
`min_width_top_surface`, `sparse_infill_line_width`, nozzle selection, and arc
simplification.

GREEN ports only fixed `split_top_surfaces()` behavior reached by nonbridge KSR
surfaces. It reuses exact Clipper operations and preserves 0.9 factors, safety
offsets, bridge exclusion margin, and operation order. Counterbore bridge
surfaces and active interface-shell multi-region behavior stay behind explicit
preflight gates.

Focused gates:

```text
cargo nextest run -p ares-core task22o_top_split
cargo nextest run -p ares-core task22o_geometry
```

## Package D: onion shells, smaller width, and gap masks

Write REDs for normal external offset, smaller-width selection, first precise
inner offset, later inner spacing, inner collapse, hole preservation, the
`-1/+1` correction, positive/zero gap-speed discovery, positive/zero sparse
density termination, and deterministic gap-only extra iteration. Freeze full
ordered offset, contour, hole, and gap-mask wires.

GREEN ports `process_classic():1235-1386` over the Package B/C state. Loop
nodes retain contour/hole, depth, and smaller-width identity. No traversal or
medial-axis materialization is added yet.

The KSR checkpoint must prove geometry-selected smaller-width paths rather than
a layer/hash list. The later complete oracle must observe 152 smaller-width
physical external paths; seven G-code line-width comments are not the package
count.

Focused gates:

```text
cargo nextest run -p ares-core task22o_onion
cargo nextest run -p ares-core task22o_top_split
```

## Package E: hierarchy, traversal, and overhang splitting

Write REDs for hole-first nesting, deep-contour nesting, internal-contour role,
children-before-contour and hole-before-children order, nearest-neighbor entity
chain, reversal eligibility, contour/hole direction, split path chain, and
supported/overhang roles. Include exact boundary-touching and alternating
inside/outside cases for open clipping. Freeze the special single-contour/
single-hole `reverse_thin_wall_hole` case separately: with KSR counterclockwise
wall direction its lone hole is counterclockwise, and its role and fixed reverse
ordering must match the oracle; ordinary holes remain clockwise.

GREEN ports the KSR-reached `traverse_loops()` and hierarchy blocks. Fuzzy is
an option-driven no-op. Paths split against the role-specific lower polygon
series only when `layer_id > raft_layers`, which specializes to after layer zero
for KSR, and retain exact coordinates and Flow metadata. KSR InnerOuter ordering
remains unchanged after traversal. Inactive reverse, sandwich, outer-first, and
outer-only-brim branches remain preflight errors when activated.

The real checkpoint freezes logical loop counts, normal/overhang path counts,
roles, and path ordering. Reference observations of 3,272 external strands,
1,971 inner strands, 56 split wall strands, and 148 Overhang feature markers
are secondary consistency gates, not replacements for exact wire identity.

Focused gates:

```text
cargo nextest run -p ares-core task22o_hierarchy
cargo nextest run -p ares-core task22o_overhang
cargo nextest run -p ares-core task22o_open_clip
```

## Package F: medial-axis gap fill and variable width

Write REDs for gap opening/difference, finite primary-edge selection, inside/
outside classification, width validation, neighbor chaining, endpoint flags,
boundary extension, short-branch pruning, loop reconnection, filter threshold,
0.05 mm width grouping, open-versus-loop entities, extrusion coverage, and
subtraction from the fill remainder. Include concave, holed, T-junction,
degenerate-close, and KSR representative geometries.

GREEN ports only the fixed medial-axis and variable-width behavior needed by
Classic gaps. Keep the Voronoi engine behind a small source-compatible adapter
so the Ares output model does not depend on third-party container shape.
Geometry construction errors are mapped at the stage boundary; trusted
traversal has no speculative fallback.

The full KSR checkpoint must match Package 0 gap bytes and totals. Reference
observations are 470 Gap feature markers, 758 continuous paths, 2,344 extrusion
moves, and approximately 0.0933202..0.70341 mm widths; only the oracle defines
the exact pre-G-code entities.

Focused gates:

```text
cargo nextest run -p ares-core task22o_medial_axis
cargo nextest run -p ares-core task22o_variable_width
cargo nextest run -p ares-core task22o_gap_fill
```

## Package G: fill remainder and complete Classic checkpoint

Write REDs for first/middle/top overlap, top-fragment 25% overlap, full
resolution simplify, narrow-fill collapse, internal surface metadata, top-fill
union, both no-overlap branches, and the inactive extra-overhang gate. Then add
the full `ARES22O` encoder/parser tests and public archive lifecycle test.

GREEN ports `process_classic():1628-1692`, assembles every prior package into a
transactional `prepare_classic_perimeters()` stage, retains the entire Task 22N
predecessor, and advances `slice_project` through Task 22O before returning
`ProjectSlicingIncomplete`.

The supplied archive must produce exactly the independently frozen wire length,
SHA, and semantic summary twice. Mutation archives independently change
precise spacing, top-one-wall, overhang detection, gap speed, gap filter,
overlaps, and relevant widths without changing unrelated ZIP entries. A
separate positive-`raft_layers` archive must return the exact existing
unsupported key without producing an O checkpoint. No test or production code
branches on fixture identity.

Focused gates:

```text
cargo nextest run -p ares-core task22o_fill
cargo nextest run -p ares-core task22o_oracle
cargo nextest run -p ares-core task22o_ksr
cargo nextest run -p ares-core task22n
```

After GREEN, freeze the exact implementation manifest and obtain independent
specification and quality approval for Packages A-G, then whole-frame approval.
Any correction invalidates affected package and whole approvals.

## Package H: WASM, docs, six-axis review, and release

Replace the nondefault Task 22N browser checkpoint feature/vector with Task 22O
without retaining two parallel public checkpoint APIs. Browser tests load the
real 3MF, compute the same versioned wire in WASM, and assert exact length/SHA
and semantic summary. Run two independent Chromium passes.

Delete the tracked `task22n_synthetic.bin` checkpoint and remove its
`include_bytes!` use from the existing perimeter oracle tests. Preserve needed
Task 22N parser and predecessor behavior through ordinary Rust fixture builders,
module-based tests, and the complete Task 22O composite tests; do not replace the
binary with generated textual Rust or another embedded byte fixture. Delete the
two old Task 22N browser vector modules after moving still-relevant behavioral
vectors into the Task 22O modules. The final tree has neither old browser vector
file and no `task22n_synthetic` reference, while the private Task 22N encoder
remains available in release builds only as Task 22O's predecessor and remains
directly testable under `cfg(test)`.

This replacement updates the feature declarations in both crate manifests, the
core re-export in `crates/ares-core/src/lib.rs`, and the gated checkpoint
entrypoints in `crates/ares-core/src/project_slice/checkpoints.rs`; it removes
the Task 22N feature-gated core exports and WASM bindings rather than retaining
aliases. The Task 22N encoder remains crate-private and is compiled as Task
22O's exact predecessor. Existing Rust predecessor behavior tests may continue
to call `task22n_browser_input_oracle` and `task22n_browser_oracle` only through
`cfg(test)` crate-private seams; those names must not be crate-root exports,
feature-gated release APIs, or WASM bindings. Their existing test modules need
no edit solely for the public feature replacement.

Only after implementation approval, update architecture and roadmap documents
with actual shipped behavior, exact evidence, remaining deferrals, and the next
fixed source boundary. Explicitly retire the old `gap_fill_target=nowhere`
perimeter-gap statement and the old `1144-2092` Classic-range wording.

Run the complete release matrix:

```text
cargo fmt --all -- --check
cargo nextest run --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace
cargo check --workspace --all-features
cargo check -p ares-core --target wasm32-unknown-unknown
cargo check -p ares-wasm --target wasm32-unknown-unknown
cargo check -p ares-core --target wasm32-unknown-unknown --features task22o-browser-oracle
cargo check -p ares-wasm --target wasm32-unknown-unknown --features task22o-browser-oracle
cargo build -p ares-wasm --target wasm32-unknown-unknown --release --target-dir target/wasm-default
wasm-bindgen target/wasm-default/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser-default
cargo build -p ares-wasm --target wasm32-unknown-unknown --release --features task22o-browser-oracle --target-dir target/wasm-task22o
wasm-bindgen target/wasm-task22o/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
node --check crates/ares-wasm/tests/browser/task22o-vectors.mjs
node --check crates/ares-wasm/tests/browser/task22o-exports.mjs
node crates/ares-wasm/tests/browser/task22o-exports.mjs target/wasm-browser-default/ares_wasm.js target/wasm-browser/ares_wasm.js
node --check crates/ares-wasm/tests/browser/server.mjs
node --check crates/ares-wasm/tests/browser/project-slice-page.mjs
node --check crates/ares-wasm/tests/browser/project-slice.spec.mjs
```

The export audit requires zero `task22*` bindings from the default artifact and
exactly `task22oBrowserInputOracle` plus `task22oBrowserOracle` from the feature
artifact. Playwright serves the isolated feature output in
`target/wasm-browser`; it never reuses an older checkpoint build.

Run repository-native browser setup and the real-project Playwright test twice.
Audit every Rust source/test physical LOC, module declarations, forbidden
source-splitting macros, unsafe, lint allowances, source-pinning tests, fixture
branches, reference-G-code production reads, and old project-path perimeter
fallbacks. Verify the deleted Task 22N binary checkpoint and browser vector
paths have no remaining references, and `main.obj` remains untouched and
unstaged.

Start one independent reviewer thread on the exact complete candidate. It is
read-only and must report findings across requirement completeness, logic,
edge cases, code quality, test coverage, and actual execution. The main thread
turns every finding into a repair checklist, applies only those repairs,
reruns proportionate and complete gates, and sends the new complete frame to
the same reviewer. Repeat until literal approval or a specific external
blocker.

After approval, refreeze the exact path manifest and normalized patch, run the
complete matrix once more, commit with Conventional Commits, push normally, and
wait for the exact-SHA Tier-1 workflow across format, Linux, Windows, macOS,
and WASM/browser. A failure returns to repair and review; never force-push a
substitute frame. Task 22O completion starts the next source-cited slice but
does not complete the persistent exact KSR G-code goal.
