# H2C manual multi-nozzle nozzle-level minimum-flush parity

## Boundary and dependency

This document specifies the next source-cited rewrite slice only. It does not
implement Rust code, add tests, or change the public API.

The prerequisite boundary is narrowed manual multi-nozzle admission with typed
`NozzleVolumeType::Hybrid` rejection (`PrintConfig.hpp:618-623`, `nvtHybrid`).
This slice must preserve rejection of a manual multi-nozzle
`NozzleVolumeType::Hybrid` declaration while removing only the current
`used_filaments <= 1` admission restriction.

The existing narrowed boundary is intentionally incomplete. Its manual path
admits a multi-nozzle extruder only when at most one used logical filament is
assigned to that extruder, then does not execute the complete upstream flush
grouping algorithm. This slice replaces that temporary narrow admission with
the upstream manual flush path for multiple used filaments.

## Upstream source boundary

Source authority is the pinned OrcaSlicer checkout at revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`. The source file is
`src/libslic3r/FilamentGroup.cpp`. The implementation must remain a rewrite of
these upstream functions and data flow, not an Ares-owned grouping design:

| Upstream location | Included source behavior |
| --- | --- |
| `FilamentGroup.cpp:901-922`, `calc_min_flush_group` | Collect sorted used filaments; use the physical nozzle-list cardinality as `k`; rebuild nozzle-level unprintable limits; choose enumeration or k-medoids from the upstream estimate; retain memoryed candidates. |
| `FilamentGroup.cpp:279-311`, `calc_group_by_enum` | Enumerate base-`k` assignments for used filaments, deduplicate equivalent assignments, evaluate preference and flush score, and retain the best assignment plus memoryed alternatives. |
| `FilamentGroup.cpp:69-90`, `fnv_hash_nozzle` | Hash nozzle volume type, right-extruder identity, and optional loaded-filament identity for equivalence classes. |
| `FilamentGroup.cpp:270-275` | Build the per-nozzle hash input from the physical nozzle list and nozzle status before assignment deduplication. |
| `FilamentGroup.cpp:359-363` | Update the optimum only on a strict preference/score improvement, retaining first-minimum behavior for equal candidates. |
| `FilamentGroup.cpp:427-459`, `select_best_group_for_ams` | Group each candidate by owning extruder, calculate AMS color-distance matching with type/support unlink constraints, and account for unavailable inventory. |
| `FilamentGroup.cpp:508-512` | Preserve the upstream minimum-cost AMS candidate choice and first-equal tie behavior. |
| `FilamentGroup.cpp:1253-1268`, `calc_filament_group_for_flush` | Run `calc_min_flush_group`, prepend its result to the retained candidates, and pass the complete candidate set to `select_best_group_for_ams`. |
| `FilamentGroup.cpp:1495-1505`, `calc_filament_group_for_manual_multi_nozzle` | Apply manual-map unprintable restrictions to the other extruder, remove group-capacity limits, and enter `calc_filament_group_for_flush`. |

The source-level contract is therefore nozzle-level: a six-nozzle extruder is
six assignment groups for enumeration and scoring, not one extruder bucket.
The manual map remains an input restriction; it is not the returned
nozzle-level result and must not be copied directly to the output map.

## Rust destination boundary

The destination remains private code under
`crates/ares-core/src/project_slice/gcode_emit/tool_ordering/filament_group/`
and its existing private caller in
`crates/ares-core/src/project_slice/gcode_emit/tool_ordering/recommend.rs`.
The likely affected seams are:

- `context.rs`: retain the existing nozzle inventory, volume restrictions,
  unprintable reconstruction, and typed `NozzleVolumeType::Hybrid` rejection;
  remove only the rejection for more than one used filament on a multi-nozzle
  extruder once the complete source path is available.
- `enumeration.rs` and the existing sequence/score support: represent the
  physical nozzle cardinality, nozzle equivalence hashes, base-`k` assignment
  enumeration, strict preference/score selection, and retained candidates
  without collapsing nozzles to extruders.
- `filament_group/mod.rs` and `recommend.rs`: route manual multi-nozzle input
  through the private equivalent of
  `calc_filament_group_for_manual_multi_nozzle` and publish the resulting
  global physical-nozzle IDs through the existing ToolOrdering views.
- Existing private context/result types may be extended only to carry the
  upstream data required by these functions. No new public API, independent
  filament-map fallback, exporter path, geometry path, or motion path is in
  scope.

The existing X2D compatibility behavior and one-nozzle-per-extruder path
remain unchanged. The implementation must not solve this slice by editing
`options/project_variants/filament_maps.rs` or by adding a second grouping
algorithm outside the ToolOrdering seam.

## Included behavior

This slice includes the complete upstream manual flush-group path for a
manual H2C-style multi-nozzle configuration:

1. Preserve the manual extruder assignment as an unprintable constraint for
the opposite extruder, as in `1495-1505`.
2. Expand the selected H2C extruder to its six physical nozzle groups and use
those groups as the assignment domain.
3. Enumerate all used-filament-to-nozzle assignments when the upstream
estimated search is below its enumeration threshold, including the upstream
nozzle hash and filament-mask deduplication. Preserve the upstream k-medoids
fallback when enumeration is too large.
4. Preserve source-owned unprintable limits, flush-matrix scoring, preference
levels, strict score improvement, and memoryed candidate retention.
5. Select among the minimum-flush and retained candidates through the existing
AMS path, including owning-extruder grouping, color-distance matching,
type/support unlink limits, missing-inventory cost, threshold behavior, and
first-equal candidate selection.
6. Publish the chosen global physical-nozzle IDs through the existing manual
ToolOrdering result and preserve the existing downstream order/map consumers.
7. Remove the temporary admission rule that rejects two or more used
filaments on a multi-nozzle extruder. Keep the typed
`NozzleVolumeType::Hybrid` rejection and all unrelated unsupported-context
checks.

The slice must preserve the source's behavior for empty or unavailable AMS
inventory and for loaded-nozzle identity in equivalence hashing. It must not
assume that equivalent physical nozzles remain equivalent when volume type or
loaded-filament state differs.

## Native-first failure witness plan

The first runtime witness must be Native-first and must use the stock H2C
machine shape with `extruder_max_nozzle_count = [1,6]`. The fixture must use
at least two logical filaments on the six-nozzle extruder; the planned compact
fixture is a three-filament manual assignment shaped like `1,2,2`.

The witness sequence is:

1. Run the exact H2C-derived fixture through Native before changing Ares and
retain the complete Native result or exact rejection. Confirm which logical
filaments are used and record the complete filament/extruder/nozzle maps and
whole public output when Native accepts it.
2. Run the same bytes through the current narrowed implementation to retain
the expected red admission failure caused specifically by the more-than-one
used-filament restriction. Do not treat this failure as proof of Native
behavior.
3. After implementation, run Ares through the same public API/CLI path and
compare the complete output and result fields to Native. The successful case
must show a nozzle-granular result; it must not manufacture a direct copy of
`1,2,2`.
4. Re-run the existing H2C single-used-filament Standard control, the X2D
three-filament control, and the typed `NozzleVolumeType::Hybrid` rejection
control. Native is run first for each new or changed witness, and the
comparison retains exact failure diagnostics when Native rejects.

The Native-first plan is a behavior gate, not an implementation shortcut.
Use the existing `NativeCase::compare` complete-output seam and its generator-only
helper: only generator identity and clock lines receive the existing treatment
(`GCode.cpp:4217`). Time estimates, footer fields, maps, and all remaining bytes
are compared strictly. No additional normalization, footer filtering, status-only
substitute, or weakened comparison is allowed.

## Deferred behavior

Defer the following adjacent upstream behavior:

- `calc_filament_group_for_match_multi_nozzle` and Auto-For-Match dispatch;
- unrelated TPU/time-optimization grouping paths;
- new AMS hardware modeling beyond the data already exposed by the existing
  context adapter;
- UI arrangement, public data-model changes, and new public APIs;
- GCodeProcessor, extrusion, geometry, motion, exporter, and configuration
  membership changes;
- broad replacement of the existing ToolOrdering scoring or sequence logic;
- fixture families beyond the H2C `1,2,2` witness and its required controls.

The typed `NozzleVolumeType::Hybrid` rejection, existing X2D behavior, and
current unsupported-context checks are retained rather than deferred.

## Spec exit criteria

Before implementation begins, this spec must define the boundary for one
upstream rewrite slice. The implementation plan must cite the pinned
`FilamentGroup.cpp` locations above, name every Rust destination file it will
change, and keep included and deferred behavior separate. The first code
change must be preceded by the Native-first H2C witness plan being executable
through the existing public harness.

## Per-file implementation plan

This plan is the source-cited boundary for the implementation. It ports only
the manual multi-nozzle flush path exercised by the H2C witness. It does not
port Auto-For-Match, TPU grouping, dynamic per-layer regrouping, new AMS
hardware models, or unrelated ToolOrdering paths.

### `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/filament_group/context.rs`

- Extend the private `Context` with the physical nozzle domain and the
  nozzle-level restrictions required by `FilamentGroup.cpp:1495-1505` and
  `FilamentGroup.cpp:901-922`: physical nozzle IDs, owning extruder IDs,
  per-nozzle volume types, manual opposite-extruder restrictions, and the
  source-owned enumeration threshold/budget inputs.
- Keep the existing matrix construction, printable/geometric restrictions,
  declaration volume restrictions, typed `NozzleVolumeType::Hybrid` rejection,
  AMS capacity parsing, and X2D single-nozzle behavior unchanged.
- Remove only the `used`-filament-count rejection for a multi-nozzle manual
  extruder. Build the manual restriction as a per-filament set of physical
  nozzles rather than copying the configured nozzle map into the result.
- Preserve the existing two-head destination and the current `Context`
  construction boundary; no public context or allocator is added.

### `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/filament_group/context/inventory.rs`

- Keep `build_nozzle_groups` as the source-cited physical expansion from
  `ToolOrdering.cpp:1503-1527` and expose only the private inventory data
  needed by enumeration.
- Do not change the existing stats validation or typed Hybrid handling.

### `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/filament_group/enumeration.rs`

- Replace the current two-extruder bit-mask solver with the bounded manual
  nozzle solver corresponding to `FilamentGroup.cpp:199-224`, `279-311`,
  `359-363`, and `901-922`.
- Use the physical nozzle count as `k`, estimate `k^n` with the pinned
  `ENUM_THRESHOLD = 10000` and early-exit bound, build FNV nozzle hashes from
  volume type, right-extruder identity, and loaded-filament identity, and
  deduplicate equivalent assignment candidates by sorted nozzle/filament
  masks.
- Apply manual opposite-extruder and volume restrictions at the physical
  nozzle level. Score every retained candidate through the existing flush
  matrix and layer-order support, then apply strict preference/score
  improvement and the source absolute flush-gap tolerance.
- Preserve the source memory heap semantics and retained candidates. Add the
  source k-medoids fallback boundary from `FilamentGroup.cpp:374-414` and
  `901-922`; for the bounded witness, the enumeration branch is selected, but
  the fallback remains explicit and is deferred from this slice unless its
  dependency can be represented without inventing a second grouping model.
- Return a private result containing both the coarse owning-extruder map and
  the selected global physical-nozzle map. The returned nozzle map must be the
  solver result, never the input `filament_nozzle_map` copied as output.

### `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/filament_group/sequence.rs`

- Keep the existing Hamiltonian layer sequence/flush scoring as the Rust
  destination for the single-nozzle ordering support used by the witness.
- Add only the nozzle-aware transition inputs needed by the manual solver:
  physical-nozzle ownership and the source loaded-nozzle state. Do not change
  the X2D score or custom-sequence behavior.

### `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/filament_group/mod.rs`

- Change the private `apply` result assembly so manual H2C uses the solver’s
  physical-nozzle IDs and derives its owning-extruder and volume maps from the
  selected nozzle records.
- Keep the existing `refresh_filament_map` writeback and the manual view
  override, preserving synchronized map/volume/nozzle identity for all
  downstream consumers.
- Keep the non-manual solver and its post-AMS master preference path
  unchanged.

### `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/recommend.rs`

- Keep the existing private caller and static ToolOrdering seam. Pass the
  manual-map restriction and returned nozzle-level result through
  `filament_group::apply`; do not add a second grouping entry point.
- Preserve the existing X2D branch and all unsupported-context behavior except
  the narrowed multi-used manual admission restriction.

### Deferred upstream source boundaries

`FilamentGroup.cpp:427-459,508-512,1253-1268` AMS color-distance matching,
`ToolOrderUtils.cpp:676-800` flow/matching solvers,
`MultiNozzleUtils.cpp:467-666` change-time/nozzle-state simulation, and
`ToolOrdering.cpp:2093-2270` per-layer nozzle rematching/order are adjacent
upstream dependencies. They remain deferred unless the H2C witness proves
that the existing context and sequence seam cannot produce the strict public
Native output without them. No comparison normalization or output filtering
may substitute for these behaviors.
