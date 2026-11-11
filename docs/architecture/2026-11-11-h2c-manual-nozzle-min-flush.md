# H2C manual multi-nozzle nozzle-level minimum-flush parity

## Boundary and dependency

This document specifies the next source-cited rewrite slice only. It does not
implement Rust code, add tests, or change the public API.

The implementation must be based on main `7cdea9d1625ac942a318ae8337f6a16df36e30f9`
and then rebase onto the narrowed manual multi-nozzle admission slice and its
typed `NozzleVolumeType::Hybrid` rejection (`PrintConfig.hpp:618-623`,
`nvtHybrid`) when that work lands. The typed rejection is a precondition: this
slice must preserve rejection of a manual multi-nozzle `NozzleVolumeType::Hybrid`
declaration while removing only the current `used_filaments <= 1` admission
restriction.

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

The Native-first plan is a behavior gate, not an implementation shortcut. No
footer filtering, status-only substitute, output normalization, or weakened
comparison is allowed.

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
Implementation starts only after the narrowed admission slice and typed
rejection are available for a clean rebase.

## Spec exit criteria

Before implementation begins, this spec must define the boundary for one
upstream rewrite slice. The implementation plan must cite the pinned
`FilamentGroup.cpp` locations above, name every Rust destination file it will
change, and keep included and deferred behavior separate. The first code
change must be preceded by the Native-first H2C witness plan being executable
through the existing public harness.
