# Diagnostic: crosshatch sparse-infill path ordering divergence (the "micro-island")

Status: diagnosis amended after two instrumented-boundary rounds (below).
Supersedes the M73 placement drift amendment in `50366ffa2`: the M73
drift was a downstream symptom of this divergence, not an export-walk bug.

## AMENDMENT (second probe round — corrected owner)

The `chain_polylines` / `connect_infill` attribution above is RETRACTED by
boundary evidence:

1. `Fill::chain_or_connect_infill` boundary (`FillBase.cpp:1820`):
   instrumented Native (probe build, G-code byte-identical to official) and
   instrumented Ares agree exactly on all calls — input order AND output
   order (39 polylines in, 14/19/4/5/5 out; sets and sequences equal;
   Ares' second-instance calls are internally identical to the first).
2. `GCode::extrude_infill` boundary (`GCode.cpp:6150-6173`): the first
divergent record is `EI_PRE call=2 i=0` — the emission-stage input of the
SECOND object instance. Native's instance-2 entity orientations are exactly
instance-1's POST-emission orientations (verified entity-by-entity), while
Ares' instance-2 input equals its instance-1 PRE input.

Root cause: upstream `GCode::extrude_infill` calls
`chain_and_reorder_extrusion_entities(extrusions, m_last_pos.to_point())`
(`ShortestPath.cpp:1061-1070`) directly on the vector of POINTERS into the
shared layer entities. The greedy chain reverses chosen entities IN PLACE
(`entity->reverse()` on the shared objects), and later islands/instances of
the same PrintObject observe those mutated orientations. Ares'
`emit_infills` (`project_slice/gcode_emit/motion/layer.rs`) owns entities
per island, so reversal decisions do not carry across instances — the
second instance arrives with pristine orientations and the anchored greedy
chain then produces a different order (the RED witness's corridor).
`ExtrusionEntityCollection::chained_path_from`
(`ExtrusionEntityCollection.cpp:87-97`) is clone-safe and NOT implicated.

Upstream owner (corrected):

- `GCode.cpp:6150-6173` `GCode::extrude_infill` — the direct
  `chain_and_reorder_extrusion_entities` call over shared entities.
- `ShortestPath.cpp:1061-1070` — endpoint filtering + greedy chain whose
  reversals mutate shared entities.
- `ShortestPath.cpp` `chain_extrusion_entities`/`reorder_extrusion_entities`
  (used by the above) — pointer-vector reorder + in-place reversal.

Ares destination boundary (corrected):

- `crates/ares-core/src/project_slice/gcode_emit/motion/layer.rs`
  (`emit_infills`, `chain_and_reorder_entities` call site),
- `crates/ares-core/src/project_slice/perimeters/classic/shortest_path/entity_chain.rs`
  (reversal application), and the island entity ownership in
  `crates/ares-core/src/project_slice/extrusion_islands.rs` /
  `island_print_order.rs` — the fix must model shared-entity orientation
  carryover across instances/islands of the same object (reversal decisions
  persist into later instances' emission inputs).

GREEN requirements: literal port of the carryover semantics (no epsilon or
constant tuning); RED `532fbc1d7` pins unchanged; regression must include
the `auto-brim-material-pla` fixture (same-source divergence), plus the brim
suite over `polyline_chain.rs`.

## Reproduction

Standalone repro (all inputs derived from `tests/parity/per-object-skirt-brim-types/outer-only.3mf`):

- both build items scaled `5.5 5.5 10` (positions unchanged),
- project settings: `brim_type=no_brim`, `brim_width=0`, `brim_loops=0`,
  `skirt_loops=0`, `sparse_infill_density=24.5`, `sparse_infill_pattern=crosshatch`.

Native (`orca-official-242`) vs Ares (main `918ae452d`) first body diff at line
~1227, layer 3 (`Z .6`), immediately after the layer-change wipe:

```text
native: G1 X124.072 Y101.117 Z.6
ares:   G1 X120.374 Y104.072 Z.6
```

The same corridor of crosshatch loops is printed in a different order. In the
earlier `x24.5` probe (anchor defaults) the visible effect is the "micro-island":
ares emits one loop as a separate 13-line path that native folds into a
neighboring path (path start/end pairing shifted by one lane:
ares-only starts `(101.123,115.98)`, `(110.33,124.02)`;
native-only starts `(98.585,115.98)`, `(112.868,124.02)`).

## Evidence chain

1. **Polyline sets are identical.** Canonicalized (direction-normalized,
   coordinate-rounded) sparse-infill path sets of layer 3 are equal: 72 paths on
   each side, zero set difference. Pattern generation
   (`FillCrossHatch.cpp:28-176`), `intersection_pl` clipping, the
   `minlength` filter (`FillCrossHatch.cpp:210-218`) and `multiline_offset` are
   therefore faithful on this repro.
2. **The connect port is excluded.** With `infill_anchor=0/0.4/1` (upstream
   `FillParams::dont_connect()`, `FillBase.hpp:52`) the divergence persists
   unchanged (stripped bodydiff 664 for all three), and path sets remain equal.
   Under `dont_connect()` upstream runs `chain_polylines` only
   (`FillBase.cpp:1820-1828`).
3. **Conclusion:** the divergence is in polyline chaining order —
   `chain_polylines` / greedy chaining / two-exchange improvement — not in
   geometry generation or boundary connection.

## Upstream owner

- `OrcaSlicer/src/libslic3r/ShortestPath.cpp:1968-1994` — `chain_polylines`:
  `chain_segments_greedy2` greedy chaining, then
  `improve_ordering_by_two_exchanges_with_segment_flipping`
  (`ShortestPath.cpp:1918-1965`), which runs
  `reorder_by_two_exchanges_with_segment_flipping` over `FlipEdge`s
  (`ShortestPath.cpp`, 2-opt with segment flipping, earlier in the file).
- Call site: `Fill::chain_or_connect_infill` `FillBase.cpp:1820-1828`
  (`dont_connect()` branch), reached from
  `FillCrossHatch::_fill_surface_single` `FillCrossHatch.cpp:225-230`.

## Ares destination boundary

- `crates/ares-core/src/project_slice/perimeters/classic/shortest_path/polyline_chain.rs`
  (`chain_polylines`, claims port of `ShortestPath.cpp:1968-1994`):
  `advanced::chain` (greedy2) + `two_exchange::improve`.
- Reached for crosshatch via `crates/ares-core/src/fill/connect.rs:74`
  (`dont_connect` branch of the `chain_or_connect_infill` port).

## Hypotheses (RED will discriminate)

1. greedy2 port tie-breaking / epsilon: ares passes
   `1.0e-4 / scale.factor()` as epsilon where upstream compares scaled-integer
   `Point` distances exactly (`chain_segments_greedy2<Point>`); equal-distance
   ties may resolve differently.
2. `two_exchange::improve` port: divergent 2-opt acceptance, crossover
   selection, flip bookkeeping, or termination, reordering a tie cluster the
   upstream heuristic leaves in greedy order.
3. f64-vs-`coord_t` distance rounding in `FlipEdge` costs flipping the order of
   two near-equal candidate moves (upstream stores `Vec2d` endpoints but
   compares with doubles derived from exact integer points).

## Included / deferred

- Included: chaining order for open polylines (infill + brim lines), the exact
  upstream semantics of greedy2 and the two-exchange improvement, RED parity
  witness on the repro above.
- Deferred: `reorder_by_three_exchanges_with_segment_flipping{,2}` (disabled
  upstream behind `#if 0`), `chain_path*` item chaining, connect/anchor logic
  (already excluded as the cause).

## Relation to prior scaffolding

The M73 export walk (processor.rs) is a faithful port — decisive simulation of
the upstream walk over oracle per-block times reproduces both native and ares
placements exactly; total-time difference (+0.08 s, this divergence's extra
travel move) shifts every M73 R-boundary globally. No processor change belongs
in this slice; the M73 drift disappears once this ordering divergence is fixed.

## GREEN boundary addendum (upstream sharing group + ledger semantics)

The carryover's enabling boundary, added to the upstream owner list after the
first GREEN round:

- `Print.cpp:2200-2251` `is_print_object_the_same` / `do_slicing` grouping:
  the FIRST print object of an identical group slices; every later identical
  one calls `set_shared_object(slicing_obj)`.
- `Print.cpp:2093-2106` `copy_layers_from_shared_object`: `m_layers` and
  `m_support_layers` are literally the representative's layer pointers — the
  shared-entity graph the chained-emission reversals mutate.

Ares port: `ProjectObject::shares_print_toolpaths`
(`crates/ares-core/src/project/domain.rs`), representative = first equal
project object in import order; consumed by `EmitState::shared_toolpath_object`.

### Field-by-field equivalence with `is_print_object_the_same`

| upstream condition | ares condition | note |
|---|---|---|
| volume count | volumes count | direct |
| `ModelVolume::type()` | `ProjectVolumeType` | direct |
| `mesh_ptr()` equality | `geometry_identity()` (`Arc` pointer) | see below |
| volume transformation | `ProjectVolume::transform` | direct |
| volume `extruder` config | `RegionOptionOverrides` equality (incl. `extruder`) | ares surfaces extruder only at the volume level (project/load/assemble.rs:229 color-group mapping); the object-level `extruder` option has no ares representation and no ares behavior depends on it yet |
| `supported_facets.equals` | `supported_facets` (`FacetsAnnotation: PartialEq`) | direct |
| `seam_facets` / `mmu_segmentation_facets` / `fuzzy_skin_facets` equality | — | NOT MODELED: the ares 3mf loader parses only `@paint_supports` (project/model_xml.rs:202-203); seam/MMU/fuzzy facet annotations are not retained anywhere in the ares project domain, so two objects differing only in those annotations compare equal in ares where upstream would not. When those annotations gain ares fields they MUST join this predicate. |
| object `extruder` config | — | see volume row |
| layer height profile pointer | `layer_height_profile` value equality | value-equal profiles slice identically |
| object config string | `ObjectOptionOverrides` equality | the overrides ARE the ares model-object config |

### Mesh identity argument (`mesh_ptr()` vs `geometry_identity()`)

Upstream: `bbs_3mf.cpp:4929-4963,5025-5028` — each volume's
`shared_mesh_id` (metadata key `mesh_shared`, `bbs_3mf.cpp:397`, defaulting to
the referenced mesh object id) keys `m_shared_meshes`; volumes with the same
id REUSE the first `ModelVolume`'s mesh, so `mesh_ptr()` compares equal
exactly for loader-shared meshes (two separately imported copies of the same
file get distinct pointers).

Ares: `project/load/assemble/volumes.rs:56-60` derives the SAME id
(`mesh_shared` id, falling back to the referenced mesh object id), and
`project/domain/shared_mesh.rs::materialize_meshes` keys an owners map by it,
cloning one `Arc<MeshGeometry>` per id. `geometry_identity()` compares that
`Arc` pointer, so `geometry_identity() ==` ⟺ same `mesh_shared` id ⟺ upstream
`mesh_ptr()` equality. Neither side merges geometrically-identical but
separately imported meshes.

### Positional ledger alignment and cursor resets

Within one `(representative, layer)` the emission-side entity sequences of
the group's objects are structurally identical: the grouping predicate fixes
mesh identity, volume transforms, and overrides, so each object's ares slice
pipeline runs the same deterministic stages over identical inputs (only the
instance translation differs, which the fill stage is translation-invariant
to — the CI boundary probe of the diagnostic round showed byte-equal
chain_or_connect_infill inputs/outputs for both instances). The ledger's
positional index is therefore the ares analogue of upstream's shared pointer
identity, and `ReversalScope.session` (the emitting print object)
discriminates passes over the same key. The cursor resets whenever
`(session, representative, layer)` changes — i.e. at every print-object or
layer pass boundary — and otherwise advances across that pass's chained
groups (regions, islands, the ironing phase, and support's ordinary/ironing
groups in `extrude_support` order).

### ByObject verification

`crosshatch_ordering_target_by_object_matches_native`
(`target-byobject.3mf`: the RED target with `print_sequence = "by object"`)
runs the carryover under sequential per-object emission — instance 2's layers
read instance 1's reversal state only after instance 1's whole object —
against Native in both canonical modes; native output is never committed.
