# Infill emission: const chaining and local reversal ownership

## Authority and boundary

Port OrcaSlicer 2.5.0, revision
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `src/libslic3r/GCode.cpp:8974–9008`, `GCode::extrude_infill`.
- `src/libslic3r/ShortestPath.cpp:1513–1528`, the const-pointer
  `chain_and_reorder_extrusion_entities` overload.
- `src/libslic3r/ExtrusionEntityCollection.hpp:85–93`, collection reversal
  permission, including `no_sort`.

Rust destinations are `gcode_emit/motion/layer.rs` for infill/ironing emission,
`perimeters/classic/shortest_path/entity_chain.rs` for const entity chaining,
and removal of infill carryover from `motion/shared_reversals.rs`.
This is a rewrite of upstream ownership semantics, not a new Ares pipeline.

## Non-negotiable ownership decisions

1. Chain immutable source entities using the current local G-code position.
   Preserve the existing greedy/improvement algorithm and reversal permissions.
2. Borrow unreversed entities; clone and reverse only selected entities. The
   reversed copies live for the current emission call, not the shared layer.
3. A collection is a local copy before child ordering. Reuse an already-owned
   reversed collection instead of cloning it again. Skip inner ordering when
   `no_sort` is true, as at `GCode.cpp:8998–9000`.
4. Remove the infill ledger, infill cursor, and infill carryover entry point.
   Do not retain the old scaffold behind a new reversal guard or fallback.
5. Do not alter source geometry, tool ordering, configuration normalization,
   prime-tower emission, test fixtures, expected output, or comparators.

The ordinary infill and ironing calls at `GCode.cpp:7779–7786` and
`:8079–8083` both use `extrude_infill` and are included. Existing regional
selection, empty-group behavior, emission order, and density application stay
unchanged. The old consuming collection helper loses its only caller and is
removed; collection-local ordering now directly mirrors the upstream call.

## Failure modes and agreed test seam

The public slicing API and CLI must produce the complete Native G-code stream,
with only the harness's existing generator-line normalization. In particular:

- Reusing an earlier entity's positional reversal bit must not reverse a
  `no_sort` solid collection in a later tool pass.
- Choosing clone ownership must not change the chain's order, reversal
  permissions, loop treatment, improvement tolerance, or current-position input.
- Sorting a collection must not mutate original/shared geometry.
- Reversing a collection must not introduce a second unnecessary deep clone.
- Removing infill state must not change support carryover, print-object
  representative grouping, perimeter emission, or shared-mesh/copies behavior.

The existing public-API/CLI E2E witness
`stock_sparse_tool_enables_prime_tower_whole_output` exercises this defect.
At Z8.6 its second model instance enters emission with the same 13 vertices,
`no_sort=true`, and local last position `(4775001,4735001)` on both sides.
The stale carried reversal selects Y112.224 instead of Native Y112.184.
No private-unit or implementation-coupled tests are added.

## Explicitly deferred: perimeter and support

### Perimeters

`GCode::extrude_perimeters`, `GCode.cpp:8915–8970`, is dispatched at
`:7770`, `:7781–7783`, and `:8071`, `:8080–8081`. It preserves regional
wall order, deferred unsupported-wall passes, and inward-wipe prefix ownership
before calling `extrude_entity`. This is not the const infill chain call.
Its loop/seam/reversal behavior and the existing Ares perimeter/shared-mesh
behavior are untouched. Extending this change to wall emission would require
its own source boundary and runtime witnesses, outside this slice.

### Support

`GCode::extrude_support`, `GCode.cpp:9011–9091`, is dispatched at
`:7695–7701` and `:8094–8097`. It filters support roles, respects collection
`no_sort`, chains const entities at `:9059`, and recursively emits nested
collections at `:9084`. Although it also owns reversed clones, its role
partitioning, recursion, raft/support identity, and multi-instance carryover
need a separate parity rewrite and witness. Its existing Ares support ledger,
cursor, shared representative selection, and all call sites remain unchanged.
This preservation is a bounded deferral, not a claim that the support scaffold
already matches the pinned const-copy ownership semantics.

`EmissionPhase` and existing object/raft scope plumbing remain compatible with
the unchanged support/shared-mesh infrastructure; infill emission no longer
reads or updates its reversal ledger.

## Regression checks

The former shared-reversal implementation and print-object grouping changes
(`715de7ffd`, `de6daf6eb`) were introduced for multiple instances and shared
meshes. A passing single witness is insufficient for acceptance.

1. Before production edits, reproduce the unchanged witness failure in both
   official and seeded Native modes on the requested base.
2. Require whole-output equality in both modes without modifying the witness.
3. On both the exact base and changed revision, run the same relevant subset
   in official and seeded modes, with filter:
   `test(runner::) | test(shared_mesh) | test(instance) | test(copies) | test(infill) | test(sparse) | test(solid)`.
   Inventory all selected cases and compare per-case results in each mode.
   `instance` deliberately includes multiple-instance naming variations.
   Do not substitute a historical inventory from a different source tree.
   Complete official/seeded suites are separate integration checks.
4. On any base PASS-to-tip FAIL transition, stop immediately and report the
   first complete-output difference and artifacts. A base failure is not a
   regression. Do not broaden this port to fix a newly failing case.
5. Run rustfmt and workspace/all-target clippy. All changed files stay below
   400 lines. Keep the core platform-neutral for browser WASM, Windows, macOS,
   and Linux; this port introduces no filesystem, terminal, native-runtime,
   target-specific, or unsafe dependencies.

E2E output files, Native/Ares comparison reports, per-case inventories,
commands, environment provenance, and test logs are the repeatable artifacts.
