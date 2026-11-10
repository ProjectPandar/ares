# Block time precision of buffered delays in the time estimate

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/GCode/GCodeProcessor.cpp`, inside `GCodeProcessor::TimeMachine::calculate_time`
(`:576`):

- `:613-636`: each processed block's motion time is a `double motion_time`. Outside the
  Klipper cruise integration (`:617-627`) it is the float `TimeBlock::time()`
  (`GCodeProcessor.hpp:743-746`). It is widened into `double block_time` (`:630`), and the
  next entry of the merged additional-time buffer, a `float` (`AdditionalBufferBlock`,
  `GCodeProcessor.hpp:847`), is added to it in double (`:631-636`). This holds for every
  flavor.
- `:639`: `time += block_time` in double (`TimeMachine::time`, `GCodeProcessor.hpp:822`).
- `:646`: the move's preview time is `block_time` narrowed to float.
- `:647`: `gcode_time.cache += block_time`. The cache is a `float` (`GCodeProcessor.hpp:777`),
  so each step adds the double `block_time` to the widened cache and narrows the sum once.
- `:649-650`: `prepare_time += block_time` for blocks of the prepare stage, the same
  float-plus-double step (`GCodeProcessor.hpp:839`).
- `:748-753`: the `g1_times_cache` entry and the stop-time entry take `float(time)` from that
  double total.

The delays reach the buffer through `simulate_st_synchronize` (`:8135`), which forwards its
additional time to `calculate_time`. The buffering is at `:587` and the merge at `:594-600`,
both in float. The callers include `process_G4` (`:6356-6364`), `process_G29` (`:6367-6376`,
260 s on the first counted `G29`), the `M400` and `SYNC` delays, and the tool-change extra
times (`:7364`, `:7488`).

`run_post_process` (`:1167`) reads the cache through `process_line_move` (`:1445-1512`). The
`M73 P… R…` pair is `int(100.0f * elapsed_time / machine.time)` and
`time_in_minutes(machine.time - elapsed_time)` (`:1463-1464`). The progress line therefore
lands on the first cached G1 whose float elapsed time crosses the next percent or minute.

## Rust destination

- `crates/ares-core/src/project_slice/gcode_emit/processor/schedule.rs`, `record_batch`: the
  port of `:630-636`. Today the non-Klipper branch adds the delay in f32
  (`*time = f64::from(*time as f32 + delay.seconds)`); only the Klipper branch adds it in
  f64. After the slice, both flavors add `f64::from(delay.seconds)` to the f64 block time.
  The `klipper` parameter then has no reader and is removed.
- `crates/ares-core/src/project_slice/gcode_emit/processor/estimate/custom_time.rs`
  (`custom_times`), the port of `:647`: each step becomes
  `(f64::from(cache) + time) as f32` instead of `cache + time as f32`. The trailing
  leftover (`:766-772`) stays a float sum and is unchanged.
- `crates/ares-core/src/project_slice/gcode_emit/processor/estimate.rs`, the `prepare` fold,
  the port of `:649-650`: the same change.
- Unchanged: the f64 accumulation of `cumulative`, the `as f32` cache and stop-time
  snapshots (`:748-753`), the preview time narrowing in `estimate/options_z.rs` (`:646`),
  and the `M73` lookup in `estimate/export_cache.rs`, `estimate/elapsed.rs` and
  `processor.rs`.

## Premises

Measured on `41a736e2e` in the official and seeded modes, on the single-nozzle base of
`ironing-single-nozzle-legs-spec.md`: the KSR model of `x1c-wall-generator-arachne.3mf` with
the MK4 configuration of `rear-seam-visibility/aligned-control.3mf` (Marlin 2 flavor).

1. The block durations already match. Native's processor and Ares, both run on the Ares
   output of the `top_surface_expansion` `inward_and_outward` leg, dump 511880 moves with
   the same float `time` on every move. The sums agree to within 2e-6 s.
2. The base has one additional-time delay: the first `G29` of the start G-code
   (`G29 P9 X10 Y-4 W32 H4`, 260 s, `:6370-6374`). No `G4`, `M400` or `SYNC` delay appears
   in the output. Near 260 s an f32 step is about 3e-5 s, so Ares's f32 sum moves every
   later cache entry by up to half of that step.
3. That shift is the only difference left on the base. With ironing off and expansion 0,
   Native and Ares differ only by `M73 P4 R137`, one G1 apart. With the f64 addition from
   above applied as a temporary change, the outputs are byte-equal apart from the producer
   comment, in both modes. The same holds for the `cubic` and `tpmsfk` ironing legs
   (`M73 P6 R168`, `M73 P4 R163`) and the `inward_and_outward` expansion leg (`M73 P41 R84`).
4. Suite effect of that temporary change, all 2525 parity tests in both modes, against
   `41a736e2e`: no PASS turns into FAIL, and exactly the three legs of premise 3 turn from
   FAIL into PASS (official 2255 → 2258 PASS, seeded 2245 → 2248 PASS). The temporary
   change covered the `record_batch` addition only, not the two float folds.

## Observable behavior

On a non-Klipper flavor, a delay that is added to a block with a fractional motion time
keeps the double precision of upstream. Every later `M73` progress line, every stop-time
`M73` and the remaining-time comments derived from the cache use the same elapsed times as
Native. With no delay, or a delay whose sum is exactly representable in f32, nothing changes.

## Failure modes

- Rounding the sum to f32 before adding it to `cumulative`, which is the current behavior:
  one rounding offsets all later progress lines.
- Adding the delay to the f32 motion time first and widening afterwards (the same defect in
  another order).
- Fixing only the block time and keeping `cache + time as f32` or the f32 `prepare` fold:
  a delayed block's time is no longer an f32 value, so narrowing before the addition gives
  a different float than upstream's single narrowing of the double sum.
- Changing the Klipper branch: it already adds in f64.
- Moving the delay to another block, or changing the buffer merge: both are already ported
  and out of scope.

## Test plan

A new test, `derived_base_places_progress_like_native`, in
`crates/ares-cli/tests/orca_parity/option_coverage/ironing_pattern/single_nozzle.rs`, builds
the single-nozzle base unchanged and compares Native with the validated API and the CLI:
whole output, API equal to CLI, `M73` lines included, nothing masked. Runtime premises, no
stored Native values:

- the derived configuration has `ironing_type = no ironing` and no
  `top_surface_expansion` key;
- the Native output has a `G29` line, whose first occurrence adds 260 s, and no
  `G4 S`/`G4 P`, `M400 S`/`M400 P` or `SYNC` line.

| Test | Before the port | After the port |
|---|---|---|
| `derived_base_places_progress_like_native` (new) | F | P |
| `cubic_ironing_matches_native` | F | P |
| `tpmsfk_ironing_matches_native` | F | P |
| `expansion_grows_top_surfaces_inward_and_outward_like_native` | F | P |

The test commit adds one FAIL per mode. The port commit is expected to turn the four FAILs
above into PASS in both modes, with no PASS turning into FAIL. Any further F→P from the
suite is listed with the port.

## Deferred

- `first_layer_time += block_time` (`:651-652`): Ares does not compute the first layer
  time, and no exported output reads it.
- `skippable_part_time` (`:644-645`): it has no reader in the exported output.
- The move dump differences found during the analysis (the fan speed of 7579 moves and the
  width and height of 4 moves). They do not affect timing and may come from how each side
  dumps moves rather than from the processor.
