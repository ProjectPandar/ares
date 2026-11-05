# G29 bed-leveling time counted once

Rewrite slice of `GCodeProcessor::process_G29` at OrcaSlicer v2.5.0-beta `345efec64`. The processor adds its fixed 260 s bed-leveling time for the first measured `G29` of a print only. Later `G29` subcommands add no time and do not synchronize the planner.

## Problem
Prusa MK4's stock `machine_start_gcode` levels the bed with six `G29` subcommands: `G29 P9 …`, `G29 P1`, `G29 P1 X0 Y0 W50 H20 C`, `G29 P3.2`, `G29 P3.13` and `G29 A`.
- **Native:** adds 260 s once.
- **Ares:** adds 260 s for each of the six, 1300 s (21m40s) too much.

For example, in `spiral-layer-ranges` R1:
- **Native:** `; estimated first layer printing time (normal mode) = 4m 32s`, total `12m 25s`.
- **Ares:** `26m 12s` and `34m 4s`.

Every `M73` progress line after the start G-code shifts with the estimate. Across the official-mode `orca_parity` cases:
- 23 MK4 cases have more than one `G29`.
- In 9 of them, the `M73` and estimated-time lines are the only differences from Native.

## Upstream boundary
- **`GCodeProcessor::process_G29`** (`GCodeProcessor.cpp:6367-6375`). If `!s_IsBBLPrinter || m_measure_g29_time`, and `m_g29_time_added` is still false, it calls `simulate_st_synchronize(260.0f)` and sets `m_g29_time_added = true`. Every other `G29` returns without any effect.
- **`m_g29_time_added`** (`GCodeProcessor.hpp:1349`) is cleared in `GCodeProcessor::reset` (`GCodeProcessor.cpp:3808`). That makes it one latch per processed print.
- **Dispatch** is by the exact command word `G29` (`GCodeProcessor.cpp:2849`). `G29.1` and `G29.2` are separate commands and never reach `process_G29`.
- **`process_M622` / `process_M623`** (`:6929-6943`) set and clear `m_measure_g29_time`. `s_IsBBLPrinter` comes from the printer model (`:3879`). This gate is already ported and stays unchanged.

## Ares destination
`processor/estimate.rs`, the processor pass that turns commands into flush events:
- **Add the `m_g29_time_added` latch:** a local `g29_time_added` in that pass, which starts false for each print, as `reset()` does.
- **Gate the 260 s delay and the planner flush:** a `G29` gets them only when it passes the existing BBL/`M622 J1` gate and the latch is still clear. That `G29` sets the latch.
- **`delays.rs` keeps the 260 s value.** Its citation, and the one in `estimate.rs`, move to the v2.5.0-beta lines above.

## Included and deferred
- **Included:** `GCodeProcessor.cpp:6367-6375`, `:3808` and `GCodeProcessor.hpp:1349`.
- **Deferred:**
  - **The remaining 1-18 s total-time difference in the same MK4 cases.** It falls after the first layer, so it is not part of the `G29` time.
  - **The Bambu Lab X2D cases.** They have two measured `G29`s in one `M622 J1` block, plus a separate model-time difference.
  - **The exact-word `G29` dispatch versus Ares's `G29` prefix test.** No witness has a `G29<digit>` command.

## Oracle and RED
A section oracle with no whole-output parity claim, because of the deferred total-time residual. It compares the `; estimated first layer printing time (normal mode)` line against Native, with Native and the Ares CLI slicing the same project. Its input fixtures already exist:
1. `tests/parity/spiral-layer-ranges/active.3mf`: Native `4m 32s`, Ares `26m 12s` on main.
2. `tests/parity/rear-seam-visibility/aligned-control.3mf`: Native `4m 32s`, Ares `26m 12s` on main.

Both are RED on main. With the latch, both print `4m 32s`.
