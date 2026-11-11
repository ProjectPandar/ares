# H2C ToolOrdering multi-nozzle grouping parity specification

## Decision

The X2D class-D premise is closed. The stock X2D fixture
`tests/parity/filament-group-flush-mechanisms/x2d_pla_other_layers_middle_last_custom_order.3mf`
contains three logical filaments and two extruders with effective
`extruder_max_nozzle_count = [1,1]`. Its ToolOrdering fields match the Native
expectations exactly: `filament_map = 1,2,1`, `filament_nozzle_map = 0,1,0`,
all volume maps, retraction lengths, AMS counts, and custom ordering fields
are equal. No X2D ToolOrdering divergence is admitted.

The true multi-nozzle witness is H2C 0.4: the stock profile
`resources/profiles/BBL/machine/Bambu Lab H2C 0.4 nozzle.json` inherits
`extruder_max_nozzle_count = [1,6]`. A public H2C-derived fixture with the
same three logical filaments reaches the Ares ToolOrdering admission and
returns `InvalidInput("unsupported automatic filament grouping context")`.
The first divergence is therefore the missing multi-nozzle grouping context,
not GCodeProcessor preheat, retraction columns, or output formatting.

## Slice split

Slice 1 is limited to `build_nozzle_groups` (`ToolOrdering.cpp:1503-1527`),
`build_filament_group_context` (`1537-1672`), the manual multi-nozzle path
through `calc_filament_group_for_manual_multi_nozzle` and
`get_recommended_filament_maps`, and derived map write-back (`3035-3087`).
Admission is restricted to manual maps with at most one used logical filament
assigned to each multi-nozzle extruder. Used means present in the collected
ToolOrdering layer usage, not merely configured. Two or more used filaments on
any multi-nozzle extruder retain
`InvalidInput("unsupported automatic filament grouping context")`.
The manual-map witness covers the admitted boundary; a Native-first
three-filament fixture with manual map `1,2,2` covers deliberate rejection.
Manual multi-nozzle grouping also rejects a Hybrid nozzle-volume declaration
on either extruder with the same unsupported error. A Native-first fixture
changes only `nozzle_volume_type` to `Standard,Hybrid`; Native slices it, while
Ares API and CLI must reject it rather than apply the uniform-nozzle proof.
`PrintConfig.hpp:618-623` and `PrintConfig.cpp:724-731` define the Hybrid
volume value as 2. The Rust typed enum and `nozzle_volume_type` registry retain
that value without substitution. `PrintConfig.cpp:10889-10892` maps Hybrid to
Standard only for preset lookup; the flush-dataset and canonical variant
readers follow that rule, while static volume-map publication preserves 2.
Project loading and config inspection remain available for Hybrid declarations.
Only the manual multi-nozzle ToolOrdering context rejects the typed Hybrid
volume; no loading-time slicing admission is applied.

Slice 2 is separate: `calc_filament_group_for_match_multi_nozzle` and its
Auto-For-Match dispatch and validation. It requires its own red test and is
deferred until Slice 1 is complete.

## Upstream source boundary

Source pin: `345efec64c7b95a05ce5cf96df0dde66a158835c` in
the pinned upstream checkout.

The rewrite slice is the following upstream boundary:

- `src/libslic3r/GCode/ToolOrdering.cpp:1503-1527`,
  `build_nozzle_groups`: expand each extruder into physical nozzle groups,
  including H2C's six-nozzle second extruder, volume type, count, diameter,
  and owning extruder.
- `ToolOrdering.cpp:1537-1672`,
  `build_filament_group_context`: construct the nozzle-centric context,
  including flush matrices, physical/geometric unprintables, AMS capacities,
  filament metadata, volume restrictions, nozzle status, and the expanded
  nozzle list. The H2C path must not be rejected merely because a configured
  nozzle count is greater than one.
- `ToolOrdering.cpp:1752-1872`,
  `ToolOrdering::get_recommended_filament_maps`: dispatch manual and
  Auto-For-Match multi-nozzle paths to
  `calc_filament_group_for_manual_multi_nozzle` and
  `calc_filament_group_for_match_multi_nozzle`, then construct and validate
  `LayeredNozzleGroupResult`.
- `ToolOrdering.cpp:1890-1930`,
  `build_default_nozzle_list`: preserve the existing one-nozzle-per-extruder
  compatibility path for X2D/H2D-like machines.
- `ToolOrdering.cpp:3035-3087`: derive and publish extruder, volume, and
  nozzle maps from the nozzle-centric result.
- Supporting upstream behavior is in
  `src/libslic3r/MultiNozzleUtils.cpp:316-369` and
  `src/libslic3r/FilamentGroupUtils.cpp:346-358`.

The existing Ares recommendation seam for
`ToolOrdering.cpp:1107-1215` is not the missing boundary and must remain
unchanged unless a later witness proves otherwise.

## Rust destination boundary

The implementation destination is private
`crates/ares-core/src/project_slice/gcode_emit/tool_ordering/`:

- `filament_group/context.rs`: replace the current multi-nozzle rejection
  with the source-owned nozzle inventory and context construction;
- `filament_group/{enumeration,sequence}.rs`: carry the nozzle-centric group
  and sequence result without collapsing physical nozzles into extruders;
- `recommend.rs` and `filament_group/mod.rs`: dispatch the manual
  multi-nozzle mode, build the private context, preserve the manual extruder
  assignment, and publish global physical-nozzle IDs.
- `options/config_export/full_config.rs`: serialize the derived nozzle map in
  the existing ToolOrdering config-block publication path.

A new private sibling module is permitted only if needed to represent the
upstream `MultiNozzleUtils` result. No public API or dependency changes are
in scope. Do not derive a separate Ares-owned fallback map in
`options/project_variants/filament_maps.rs`.

## Included behavior

Port only physical-nozzle expansion, context construction, the restricted
manual multi-nozzle case, and map write-back. This is not the complete upstream
manual grouping algorithm; Auto-For-Match is not included. Preserve X2D's existing one-nozzle-per-extruder behavior and
all existing scoring, ordering, emission, and fixture behavior outside this
boundary.

## Restricted manual equivalence

`FilamentGroup.cpp:1495-1505` adds manual-map restrictions to the other
extruder, removes group capacity limits, then calls
`calc_filament_group_for_flush`. That function (`1253-1268`) invokes
`calc_min_flush_group` and then `select_best_group_for_ams`; it does not simply
copy the manual extruder IDs. General nozzle-level min-flush and AMS candidate
selection therefore remain an independent future slice.

For the admitted static inventory, nozzles on a given extruder have the same
diameter and configured non-Hybrid volume, with no loaded-filament state or
switcher. `ToolOrdering.cpp:1503-1527` constructs that uniform group;
`MultiNozzleUtils.cpp:25-40` sorts groups and emits increasing global nozzle
IDs. In the H2C Standard witness these are nozzle 0 on extruder 0 and nozzles
1 through 6 on extruder 1.

With at most one used filament on a multi-nozzle extruder, moving that filament
among its equivalent nozzles changes neither flush cost nor load/unload timing.
The single-nozzle extruder has no alternative assignment. Enumeration in
`FilamentGroup.cpp:279-311` traverses increasing base-k masks and hashes
same-volume, unloaded nozzles identically (`nozzles_hash`, `fnv_hash_nozzle` at
`69-90,270-275`). The first admissible assignment uses the lowest global nozzle
ID on that extruder; equivalent later assignments are deduplicated.
`FilamentGroup.cpp:359-363` updates the optimum only for a strict score
improvement at equal preference. `1258-1259` puts this optimum first in the
AMS candidate list; `427-459,508-512` groups AMS costs by extruder and keeps
the first equal-cost candidate. Thus the Standard witness selects nozzle 1,
not nozzle 6, for its single filament on the six-nozzle extruder, giving
`0,1,0`. The existing static adapter supplies empty AMS inventory and no
loaded nozzle state; live inventory and Hybrid-volume inventories are not
covered by this equivalence argument.

For two used filaments on that extruder, distributing them among nozzles can
change flushing. The proof no longer applies, so the adapter must reject
rather than return the first nozzle for both. The rejection witness first
requires Native slicing to succeed and demonstrate all three logical
filaments are used with map `1,2,2`, then requires the exact unsupported error
from API and a failing CLI with the same diagnostic.

## Deferred behavior

Defer complete nozzle-level manual min-flush/AMS selection and its multi-used-
filament admission, Auto-For-Match, GCodeProcessor preheat, retraction-column behavior, UI arrangement,
new golden fixtures, production observation hooks, and unrelated kernels.
Do not weaken comparators or add fallback behavior for unsupported grouping.

## Validation gate before implementation

The implementation must rerun the H2C-derived public witness and the X2D
three-filament control through Native/API/CLI or the equivalent public
harness. The H2C result must expose the nozzle-granular map and complete
without the current unsupported-context error; X2D must remain unchanged.
