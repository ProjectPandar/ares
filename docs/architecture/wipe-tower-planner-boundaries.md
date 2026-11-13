# Wipe-tower planner boundaries and template line termination

## Upstream boundary

Port OrcaSlicer commit `34c8e78e31a7ec9278070d881b25a25c3b171a88`
from the beta revision `345efec64c7b95a05ce5cf96df0dde66a158835c` to
final revision `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
The following line numbers refer to the final revision:

| Upstream owner | Included behavior | Rust destination under `crates/ares-core/src/project_slice/gcode_emit/wipe_tower/` |
|---|---|---|
| `GCode/WipeTower.cpp:719–722`, `flush_planner_queue_command` | Raw flavor-specific command: Klipper `M400\n`, other flavors `G4 S0\n` | `writer/commands.rs` shared command spelling |
| `GCode/WipeTower.cpp:1302–1308`, `WipeTowerWriter::flush_planner_queue` | Suppress only Klipper tool-change boundary synchronization | shared writer boundary operation used by `type1/{data,tool_change}.rs` |
| `GCode/WipeTower2.cpp:410–414`, `WipeTowerWriter2::switch_filament_monitoring` | Retain raw synchronization before `M591` | `writer/commands.rs` monitoring operation |
| `GCode/WipeTower2.cpp:650–661`, `WipeTowerWriter2::set_extruder_temp` | Temperature synchronization is explicit and normally enabled | `writer/commands.rs` temperature operation |
| `GCode/WipeTower2.cpp:707–713`, `WipeTowerWriter2::flush_planner_queue` | Suppress Klipper boundary waits, not raw temperature/unload waits | shared writer boundary operation used by `prime.rs` and `tool_change.rs` |
| `GCode/WipeTower2.cpp:1333–1452`, `WipeTower2::tool_change` | On Klipper only, skip the non-timing-critical interface cooldown sync in both cooldown placements | `tool_change.rs` |
| `GCode/WipeTower2.cpp:1463–1723`, `WipeTower2::toolchange_Unload` | Retain the final raw synchronization before filament-change macros and tool selection | `unload.rs` |
| `GCode/WipeTower2.cpp:1727–1854`, `WipeTower2::toolchange_Change` | Terminate `[deretraction_from_wipe_tower_generator]` with a newline before later tower moves | `tool_change.rs` |
| `GCode.cpp:1667`, `:1892–1935`, `WipeTowerIntegration::append_tcr2` | When removing the interface `M109`, remove its immediately preceding complete raw planner-sync suffix | `interface_temperature.rs`, called by `emit.rs` with the configured flavor |

The existing Rust generator/writer/integration modules are compatibility shells
around these upstream owners. This slice corrects their beta behavior in place;
it does not introduce an Ares-owned tower pipeline or a general G-code cleanup
pass. Keep the raw synchronization command distinct from the boundary operation.
Never globally remove `M400` or whitespace from generated output.

## Included and deferred behavior

Include both tower writers, the two interface cooldown placements, the unload
macro ordering, monitoring/temperature synchronization, and exact template line
termination for all flavors. Non-Klipper boundary commands remain unchanged.
`WipeTower.cpp:1447–1455` (`format_line_M104`) retains its unconditional BBL
precool `M400`; that path is not the changed boundary operation.

Defer tower geometry, travel routing, purge allocation, layer/tool ordering,
filament selection normalization, source-probe recipes, and all unrelated
configuration/G-code changes. In particular, tree support, scarf, TPMS joining,
inward wipe and retraction-column candidate changes require separate Native
confirmation and separate source-cited slices. No beta fallback, version switch,
comparison mask, golden update or cache-protocol change belongs in this port.
The core remains platform-neutral for browser WASM, Windows, macOS and Linux.

## Observable contract and test sequence

Use the public byte-oriented slice API and the Native CLI as the observation
boundary. Preserve the existing strict whole-output generator-only comparison.
The already failing 48 parity cases attributed to this source change cover
Klipper boundary waits and the extra newline, including non-Klipper and SEMM
controls. Run them unchanged before and after implementation in both final-release
Native modes.

The existing `power_loss_recovery/queue_flush.rs` checks only the collected queue
command sequence; it cannot distinguish necessary unload/temperature waits from
redundant boundary waits and has no explicit interface-cooldown coverage. Before
implementation, extend E2E coverage using the same public API/Native CLI seam and
retained input/native/Ares/result artifacts. Follow the upstream contract in
`tests/fff_print/test_wipe_tower.cpp:198–289`: identify filament-change markers,
retain the unload sync, permit temperature/monitoring syncs, reject redundant
Klipper boundary syncs, and exercise interface cooldown with both placements.
Retain a non-Klipper control and check that emitted tower moves remain processed,
not glued to the deretraction placeholder. Do not assert a private helper's shape.

Failure modes to distinguish are: deleting required synchronization with the
boundary waits; retaining an orphan sync after removing an interface `M109`;
synchronizing a Klipper interface cooldown; disabling non-Klipper waits; and
losing the placeholder newline so a move bypasses tower-coordinate processing.
Record the pre-implementation failures before changing production code.

Acceptance requires all 48 affected cases plus directed controls to pass under
`parity-250.env` and `parity-seeded-250.env`, `cargo fmt --check`, and
`cargo clippy --workspace --all-targets -- -D warnings`. Keep repeatable input,
Native/Ares outputs, comparison results and command receipts outside the source
tree. Commit the specification first, additional tests second when needed, and
the production port last. No full-suite baseline rerun is required.
