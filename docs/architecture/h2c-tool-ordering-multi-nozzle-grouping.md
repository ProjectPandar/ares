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
Its red test is an H2C manual-map fixture.

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

Port only the upstream physical-nozzle expansion, context construction,
manual and Auto-For-Match multi-nozzle grouping, result validation, and
map write-back. Preserve X2D's existing one-nozzle-per-extruder behavior and
all existing scoring, ordering, emission, and fixture behavior outside this
boundary.

## Deferred behavior

Defer GCodeProcessor preheat, retraction-column behavior, UI arrangement,
new golden fixtures, production observation hooks, and unrelated kernels.
Do not weaken comparators or add fallback behavior for unsupported grouping.

## Validation gate before implementation

The implementation must rerun the H2C-derived public witness and the X2D
three-filament control through Native/API/CLI or the equivalent public
harness. The H2C result must expose the nozzle-granular map and complete
without the current unsupported-context error; X2D must remain unchanged.
