# Type2 no-ram purge-row lattice

## Upstream boundary

This slice rewrites the no-ram handoff and purge-box completion of
OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `GCode/WipeTower2.hpp:318–334`: `wipe_row_spacing`, `wipe_line_width`,
  and `wipe_start_offset_after_ram`.
- `GCode/WipeTower2.cpp:1395–1428`: `tool_change` captures the old-tool
  ramming predicate and forwards `fill_box` to `toolchange_Wipe`.
- `GCode/WipeTower2.cpp:1694–1720`: `toolchange_Unload` chooses a
  ram-aligned travel or a no-ram writer-position reset.
- `GCode/WipeTower2.cpp:1881–1980`: `toolchange_Wipe` traverses the
  allocated rows, with volume exhaustion conditional on `fill_box`.

The Rust destination is the existing Type2 implementation under
`crates/ares-core/src/project_slice/gcode_emit/wipe_tower/`:
`unload.rs`, `tool_change.rs`, and `wipe.rs`. A shared lattice expression
may live in the existing `generator.rs` when both source consumers need it.
These modules are a temporary compatibility shell around the cited upstream
functions, not an independently designed tower pipeline.

## State and arithmetic

The no-ram branch must not align to a ramming row that was never printed.
Its restart Y is the cleaning-box lower Y plus traversed depth, the source
wipe-start offset with zero ram depth, and perimeter width. The offset is
computed from the actual first-layer predicate, wipe flow, wipe spacing,
and perimeter width in the source expression order.

The ram-aligned expression remains exclusive to the existing ramming path.
A no-ram reset changes writer state without inventing a travel command.
The following tool-change template and load phase consume that state through
the existing writer/result handoff.

`fill_box` belongs to the old tool. Compute its source predicate before
`toolchange_Change` replaces the current tool, then transport the value to
the existing wipe consumer. Do not reconstruct it from the new material.
The no-ram path fills its already allocated box instead of terminating as soon
as the requested purge volume has been consumed. The source spatial end
condition, row pitch, direction changes, speed ramp, and outgoing wipe path
remain ordered as in `toolchange_Wipe`.

Preserve upstream float operations and casts. Do not fit offsets to emitted
positions, quantize the lattice early, adjust tolerances, or reconstruct
layer height from Z. Existing planning supplies the box; this slice does not
change purge ownership, volume allocation, tool order, or tower placement.

## Caller and platform boundary

The existing generator is also used by planning simulation. Its no-ram
unload/wipe behavior must be shared with emitted generation, not duplicated
as a planning-only correction. Audit `set_toolchange` and
`save_on_last_wipe` callers without independently changing their algorithms.

All changes stay in the platform-neutral core. No filesystem access, native
runtime dependency, terminal behavior, rendering, or adapter-owned slicing
logic is introduced. The arithmetic and state boundary are shared by browser
WASM, Windows, macOS, and Linux.

## Included and deferred behavior

Included behavior is the no-ram writer-position reset, old-tool `fill_box`
handoff, and the corresponding wipe-box completion condition. Already
ported entry openings, wall gaps, rib positioning, and planner flushing are
inherited unchanged.

Active ramming-band expansion, non-SEMM boundary-ram feature work, Type1,
final purge, tower lift, filament grouping/maps, interface feature changes,
configuration producers, and alternative geometry backends are deferred.
No fallback, alternate generator, diagnostic side channel, or fixed emitted
position is part of the API.

## Observable contract

The existing seven-case SEMM public suite observes complete ordered Native
output through the byte-oriented API and CLI. Ramming-off cases retain their
independent inactive-unload assertions; enabled-ramming siblings remain
controls. No fixture, registration, premise, or comparator changes are needed.

The contract includes M73, metadata, and configuration with only the existing
generator-line normalization. A later first difference is not equality.
The earlier first-layer difference of the opposite profile-order case is a
separate residual unless the shared source path establishes its cause.
