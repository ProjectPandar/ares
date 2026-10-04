# ARD-0105: Source-owned timelapse state and Type2 wipe tower

## Source and Rust boundary

Pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1` owns this
slice in `Print::{has_wipe_tower,enable_timelapse_print,_make_wipe_tower}`,
`Print::wipe_tower_type`, `GCode/WipeTower2.cpp` (planning, writer, geometry,
same-tool layers and final unloading), and `GCode.cpp::WipeTowerIntegration`.
The existing Rust `project_slice` preparation and `gcode_emit` modules are
temporary compatibility shells around those upstream operations, not owners
of a newly designed pipeline. The planned Type2 destination is private
`gcode_emit::{wipe_tower,motion::tower}`, with existing `layers`, `footprint`
and `placeholders` as integration points for the upstream print boundary.

First isolate the transferred BBL motion-state discrepancy. Its candidate
owners are `GCode.cpp::{process_layer,_extrude,travel_to}` and
`GCodeWriter.cpp::{travel_to_xyz,travel_to_z,unlift}`. The Rust destination is
the existing timelapse tracking and `motion/path/start_travel` boundary.
The confirmed repair is in
`motion::path::start_travel::approach::move_emit::emit_approach_move`:
`GCodeWriter::travel_to_xyz`'s force-Z flag must bypass the lifted dead-band.
Restore source state transitions, not hard-coded duplicate Z instructions.

## Non-negotiable decisions

- Preserve original full G-code comparisons, including repeated commands,
  comments, M73, final purge and statistics. No comparator relaxation.
- Reuse WIP Type2 arithmetic selectively, not its single-object/material
  guards or fixed region selection. Current object, region, tool, coordinate
  and writer ownership must remain intact.
- Tower activation, occupancy, placeholders, travel/wipe handoff, final unload
  and Adaptive PA reset integration belong to the cited upstream boundary.
  A matching first tower block alone is not completion.
- Coordinator selected complete Type2 activation before integrated review.
  Type2 multi-tool partitions, raft-gap tower layers and rectangle/cone/rib
  branches are included, not deferred behind the historical same-tool plan.
  Any activated source branch must be accounted for explicitly; no new
  rejection guard, silent omission or legacy fallback may hide it.
- Independent Type1 algorithms remain outside this slice. Shared upstream
  `WipeTower.hpp` result/geometry data needed by Type2 are included. Preserve
  source printer-type dispatch rather than introducing an Ares capability
  filter. Same-tool fixtures alone cannot validate the integrated slice.
- Writer/geometry commits are not independently reviewable completion:
  preserve their private Rust boundary, connect their consumers before a
  review request, and verify complete E2E output. Do not manufacture a public
  API or retain dead code merely to publish an isolated component.
- Tests use exported project bytes and complete output through the existing
  CLI parity harness. Keep official and seeded evidence separate.
- Every green slice is pushed for coordinator Claude review. Stop after
  `REVIEW-REQUEST`; only a matching approval permits rebase/revalidation and
  fast-forward integration. No external review programs.
