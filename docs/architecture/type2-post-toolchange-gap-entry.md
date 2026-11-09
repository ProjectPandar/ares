# Type2 post-toolchange gap entry

## Source boundary and destination

This prerequisite is a Rust rewrite of the missing post-toolchange branch in
`WipeTowerIntegration::append_tcr2`, pinned OrcaSlicer
`345efec64c7b95a05ce5cf96df0dde66a158835c`, based on commit
`dcf54f59d9de1209eb659cdb2032fc1efb408685`. It is not a new routing policy or
independent tower pipeline. Existing Rust tower integration is a temporary shell
around this upstream integration; extend it rather than adding another emitter.

| Upstream file under `src/libslic3r/` | Included owner |
| --- | --- |
| `GCode.cpp:1696-1734` | Existing pre-toolchange approach conditions: force travel, no tool change, lower tower layer, ramming, explicit tool-change-on-tower override, or pending layer-change lift. Preserve this branch. |
| `GCode.cpp:1760-1794` | After `set_extruder`, when no approach occurred, not priming and gap-wall enabled, recover the route start, route through the existing wall opening and append travel to the custom-change substitution before Z restoration/deretraction. This is the missing branch. |
| `GCode/GCodeProcessor.cpp:4310-4407` | `get_last_position_from_gcode` scans complete newline-terminated G0/G1/G2/G3 lines for parseable X/Y/Z, seeded with the live writer's float position; otherwise the integration uses its last defined object point. |
| `GCode.cpp:11496-11503` | `gcode_to_point` converts the recovered point using origin and active extruder offset; preserve its truncating grid conversion and the caller's plate-origin addition. |
| `GCode/WipeTower2.cpp:1395-1411,1723-1737,1827-1848` | The generator inserts `[change_filament_gcode]` and later its tagged, mandatory positioning move before `[deretraction_from_wipe_tower_generator]`; these remain intact. |

Rust destination:
`crates/ares-core/src/project_slice/gcode_emit/wipe_tower/emit.rs::append`,
with a small sibling `entry.rs` if needed for the source-owned position scan and
post-change entry. Reuse `tower_gap::waypoints`, the existing object/grid mapping
and live writer travel operations; do not rewrite the route or manually emit
fixture coordinates. No Type1 generation changes are included.

## Read-only causal witness

At `dcf54f59d9de1209eb659cdb2032fc1efb408685`, every non-SEMM case has matched Native/API/CLI positive-XY-E
wipe-volume aggregates after the accepted minimum floor; the skirt first
failure is gone. Official and seeded each retain five whole-byte failures and
one SEMM whole-output PASS. In the min-zero control, immediately after
`; filament start gcode`, Native emits `G1 X144.8 Y150.25` then
`G1 X150.25 Y150.25`; Ares has an empty line before the separate
`G1  X150.250 Y150.250` generator positioning form. The original first mismatch
is byte 14742, line 649, column 1.

The applied non-SEMM, no-ramming, gap-wall case selects the missing conditional
post-change branch; SEMM's existing pre-change approach is why that control
already passes. The wall opening route's existing bounding-box clearance
produces the waypoint; no constant `144.8` belongs in the implementation.
This source-localized branch gap does not indicate that formatting or the
mandatory generator move should be changed.

## Included and deferred behavior

- Capture whether the pre-change approach actually happened before live state
  changes; only the upstream no-approach/non-priming/gap-wall branch may run.
- Append travel to the change-filament replacement after installing the incoming
  tool, before Z restore and generator deretraction; keep its newline boundary.
- Recover a custom-change G-code motion endpoint using the cited scanner,
  including partial axis updates; otherwise use the last defined position.
  If neither exists, upstream emits no route. Do not invent a fallback point.
- Preserve current generated tower start transform, plate/nozzle offsets, gap
  routing, bed clipping, motion/retraction/lift behavior and mandatory tagged
  positioning. Do not force pre-change travel for all non-SEMM printers.
- Defer temperature parking/waits, ramming, automatic finishing selection,
  mixed-filament `Print.cpp:2443-2447`, depth/rib/width producers, Type1, carousel
  and geometry algorithms. No decoder, comparator, reference or fixture waiver.
- Keep core platform-neutral; no filesystem, UI, terminal or native runtime
  dependency enters this Rust destination.

## Verification

The existing six public Native-first E2Es are already failing before this fix;
retain their complete Native/API/CLI ordered-byte comparisons and generator-only
normalization. Add a public control for custom tool-change XY movement before
implementing its route-origin recovery; preserve real geometry and matched
catalog inputs. The SEMM control must stay unchanged. Run official and seeded
six-case comparisons plus the custom-move control and the same 17-ID adjacent
regression filter; completion requires whole-output PASS, not a waypoint-only
assertion, volume equality or removal of the first mismatch. Preserve and report
any new first failure instead of absorbing another upstream boundary.
