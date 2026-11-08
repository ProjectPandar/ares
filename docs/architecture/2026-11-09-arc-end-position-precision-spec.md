# Arc end-position precision

## Rewrite boundary

Authority: OrcaSlicer v2.5.0-beta, commit
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

| Upstream | Behavior | Rust destination |
| --- | --- | --- |
| `GCode/GCodeProcessor.cpp:6077` (`process_G2_G3`) | the arc end position comes from `extract_absolute_position_on_axis` for X, Y, Z and E | `processor/motion/parse.rs` (`MotionState::motion`), `processor/arc_accounting.rs` (`arc_internal_g1_lines`) |
| `GCode/GCodeProcessor.cpp:8180` (`extract_absolute_position_on_axis`) | reads `line.precise_value(axis)`, a double, in every flavor | `processor/motion_util.rs` (`precise_word`) |
| `GCode/GCodeProcessor.cpp:5190` (`process_G1(const GCodeLine&)`) | a linear move takes `(double)line.z()`, and X, Y, E the same way; Klipper alone keeps the precise value (`:5195-5202`) | `processor/motion/parse.rs`, unchanged |
| `GCodeReader.hpp:78` (`GCodeLine::z`) | `float(m_axis[Z])`, against `precise_value` at `:39` | `processor/motion_util.rs` (`word`) |
| `GCode/GCodeProcessor.cpp:6154` (`process_G2_G3`) | the arc feedrate reads `line.f()`, a float | `processor/motion/parse.rs`, unchanged |
| `GCode/GCodeProcessor.cpp:6100-6102` (`process_G2_G3`) | `I` and `J` read through `has_value` into a float | `processor/motion/arc.rs`, unchanged |
| `GCode/GCodeProcessor.cpp:5273` (`process_G1`) | a move with any nonzero delta queues a block | `processor/motion/parse.rs`, unchanged |

This is a port of how the upstream time estimator reads an arc, not a change to the
emitted motion.

## Measured owner

Fixture: `lightning_initial_overhang_polygons_a1`. Its non-`M73` lines already match
Native. A few `M73` lines sit a few lines away from Native with the same values, the
first of them inside the start G-code, before any layer. So the G-code matches and the
estimate behind it does not.

`g1_times_cache` and `machine.time`, compared by raw bits at the start of the
post-process pass, show Native holding extra entries that Ares does not. Every extra
entry belongs to the same line: `G1 Z<layer>` from the layer-change G-code, following
the spiral lift `G3 Z<layer> ... P1` to the same height. There is no extra entry at
heights a float represents exactly.

At the `process_G1` displacement test for those entries, the start Z is the decimal
double and the end Z is the same decimal narrowed to a float, for example `0.6` and
`0.6000000238418579`. The delta is nonzero, in either direction depending on the
height, so upstream queues and caches a real block. It adds about a millisecond each
time and shifts the total slightly, which moves the line where a percentage or minute
value changes.

## Rust divergence

| Site | Rust today | Upstream |
| --- | --- | --- |
| `parse.rs:57` | `precise` only for Klipper `G0` and `G1`, so an arc reads X, Y, Z, E through `word` | an arc reads them precisely in every flavor |
| `parse.rs:78-79` | the comment says every path but Klipper linear keeps the float word | the float word belongs to linear moves only |
| `arc_accounting.rs:15` | end X and Y for the internal line count read through `word` | one end position per arc |
| `motion_util.rs:9-10` | `precise_word` documented as Klipper linear only | also the arc end position |

An arc therefore ends on the same float as the following `G1 Z`, the delta is zero, and
no block is queued.

## Change

1. In `MotionState::motion`, `G2` and `G3` read X, Y, Z and E through `precise_word` in
   every flavor. `F` keeps its current read.
2. In `arc_internal_g1_lines`, read end X and Y through `precise_word`, so the internal
   line count and the generated segments derive from the same end position. `F` keeps
   its current read.
3. Correct the comments at `parse.rs:78-79` and `motion_util.rs:9-10`.
4. Update the `GCode/GCodeProcessor.cpp` row of
   `2026-10-30-upstream-baseline-v2.5.0-beta-inventory-processor-platform.md`: the
   status stays `M`, and the scope records the arc end-position read as updated to the
   target while the rest of the file stays a bounded earlier slice.

`I`, `J` and `R` are not touched. No API changes.

## Included and deferred

Included: the end-position read of `process_G2_G3` for all flavors, in both Rust sites
that read it.

Deferred:

- Some `g1_times_cache` entries still differ from Native by one float unit in the last
  place, starting before the first arc, and the total differs by about ten
  microseconds. The cause is not identified. It flips no percentage or minute value on
  the failing case, but it could move an `M73` line elsewhere, so it needs its own
  measurement.
- The rest of `GCode/GCodeProcessor.cpp`.

## Failing case and acceptance

No test code is added. The failing case already replays through the public 3MF path and
compares the complete stream with a Native reference produced at runtime, with no stored
coordinate or count:

- `lightning_initial_overhang_polygons_a1`.

It needs the lightning per-tree polyline walk, so this change stacks on it.

Acceptance, in official and seeded modes:

- the case fails on the unchanged tree and passes with the change,
- no test moves from pass to fail in the
  `lightning | tree | support | ironing_pattern | coplanar_outer_wall` selection or in
  the `arc | spiral | m73 | lift | time | klipper | marlin | helix | scarf` selection,
- every other test that moves from fail to pass shows the same `M73`-only first
  difference on the unchanged tree.

Arcs appear throughout the parity fixtures, so the full suite is the final check.
