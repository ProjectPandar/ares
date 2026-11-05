# Wipe-tower planner-queue flush

## Upstream boundary

This is a Rust rewrite slice of OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`, `src/libslic3r/GCode/WipeTower.cpp:719–722`, `flush_planner_queue_command(GCodeFlavor)`. It returns `M400\n` for Klipper and `G4 S0\n` for every other flavor. `WipeTowerWriter::flush_planner_queue` delegates at `WipeTower.cpp:1302–1305`; `WipeTowerWriter2::flush_planner_queue` delegates at `WipeTower2.cpp:706–709`.

## Rust destination

`crates/ares-core/src/project_slice/gcode_emit/wipe_tower/writer/commands.rs` owns one `Writer::flush_planner_queue` associated appender accepting the output buffer and flavor. Both existing writer representations route their queue-flush sites through this appender. Type2 writer commands use their stored flavor; generator call sites use their existing generator/settings flavor. Type1 does not acquire duplicate flavor state or a second command-selection implementation.

The existing Type1/Type2 generators remain temporary compatibility shells around the cited upstream wipe-tower classes. This change replaces hardcoded queue-flush bytes, not tower planning, tool selection, extrusion accounting or a new Ares slicing pipeline.

## Complete call-site mapping

Rust paths below are relative to `crates/ares-core/src/project_slice/gcode_emit/wipe_tower/`. Rust line numbers identify the unmodified `2af5c375f` base.

| Rust site | Upstream caller | Meaning |
|---|---|---|
| `writer/commands.rs:45,47`, `switch_filament_monitoring` | `WipeTower2.cpp:411` | Flush before either `M591` branch. |
| `writer/commands.rs:54`, `set_temperature` | `WipeTower2.cpp:652` | Flush before `M104`/`M109`. |
| `prime.rs:85` | `WipeTower2.cpp:1314` | Priming completion, before E reset and end marker. |
| `tool_change.rs:132` | `WipeTower2.cpp:1445` | Tool-change/final-purge completion, before E reset. |
| `unload.rs:177` | `WipeTower2.cpp:1719` | Resume preview and flush after unload. |
| `tool_change.rs:155` | `WipeTower2.cpp:1848` | Flush after changing the current tool. |
| `type1/data.rs:43` | `WipeTower.cpp:2433` | Legacy `tool_change(-1)` final-purge result, before E reset. Preserve the existing rule that finalization does not emit this disabled-unload result. |
| `type1/tool_change.rs:89` | `WipeTower.cpp:2745` | Flush after the filament-start slot. |
| `type1/tool_change.rs:163` | `WipeTower.cpp:3604` | `tool_change_new` completion, before E reset. |
| No active Rust unload body | `WipeTower.cpp:2709` | The upstream Type1 unload body and this caller are inside `#if 0` (`:2446–2710`). Do not enable or invent this behavior. |

Every listed Rust `G4 S0` is a queue flush, not an intentional flavor-independent dwell. Split combined strings at the flush boundary while preserving every surrounding byte and its order. Non-Klipper output remains `G4 S0\n`.

## Included and deferred behavior

Included: flavor-aware queue flushing at every mapped active call site, including Type1 bookkeeping results, monitoring, temperature, priming, unload and tool-change completion.

Deferred: `wait_command` duration spelling and rounding (`WipeTower.cpp:724–729`), the deliberately forced BBL pre-cooling `M400` (`:1447–1449`), disabled Type1 unload behavior, thermal configuration defaults, retraction ownership, estimator changes, and unrelated motion behavior. No global G-code substitution, postprocessing fallback or fixture-specific production branch is permitted.

## Public boundary and failure model

The public boundary is Native slicing versus the public Ares project-slicing API. The existing strict `power_loss_recovery_empty_klipper_matches_native` whole-output assertion remains unchanged.

The Klipper fixture must expose its first differing queue flush as Native `M400` versus pre-port Ares `G4 S0`. A separate section oracle compares the ordered flush commands inside named CP priming/tool-change sections; it must reject a temperature-only fix that leaves other flush sites unchanged. An authored non-Klipper control changes only the fixture's flavor to Marlin and verifies Native and Ares retain `G4 S0` at those sections. It must reject an unconditional `M400` replacement.

Each section run retains the input, Native and Ares G-code, the unchanged whole-output comparator result and the observed flush sequences. Section parity does not waive a failing whole-output assertion. Neither empty sections nor a wrong Native flavor/command premise establish success. Static call-site review covers the disabled/non-emitted Type1 result and confirms all active mapped literals route through the single appender; it does not claim runtime execution of every branch.
