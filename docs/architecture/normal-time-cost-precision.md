# Normal-mode time precision for print cost

## Upstream boundary

This slice rewrites OrcaSlicer v2.5.0
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `src/libslic3r/GCode/GCodeProcessor.hpp:94-97`: the estimated mode's
  `time` is `double`, while `prepare_time` is `float`.
- `GCodeProcessor.hpp:843`: `TimeMachine::time` is `double`.
- `GCodeProcessor.cpp:4032-4034`: `GCodeProcessor::get_time` returns the
  machine's double time without narrowing.
- `src/libslic3r/GCode.cpp:2564-2599`:
  `DoExport::update_print_estimated_stats` reads the double normal-mode
  time and adds `config.time_cost.getFloat() * (normal_print_time / 3600.0)`
  after accumulating the used writer extruders' material cost.

The Rust destination is `PrintEstimatedStatistics::normal_time` in
`ares-core/src/gcode_processor.rs`, its assignment in
`project_slice/gcode_emit/processor.rs`, and its consumption in
`processor/estimated_stats.rs`. The existing timing estimator is the
compatibility shell for this upstream boundary; its already-double total
must not be narrowed before entering the statistics model.

## Non-negotiable decisions

Normal-mode accumulated time remains `f64` through statistics and cost.
Preparation time remains `f32`. Per-block durations, move-vertex times,
planner arithmetic, and other upstream float fields are not widened.
The browser estimation adapter carries the resulting double time directly;
its existing float preparation time is still widened for JSON transport.
There is no new pipeline, compatibility fallback, or rounding adjustment.

The current narrowing loses low-order time bits even when material volume,
filament length, and weight agree exactly. Converting that rounded float
back to double cannot recover them. A cost tolerance would hide this loss
rather than port the upstream types.

## Verification boundary and failure inventory

Use the existing public processed-slice API comparisons
`print_estimated_stats_time_cost` and
`print_estimated_stats_unused_filament` against the runtime Native
`print.print_statistics()` producer. The two synthetic 3MF inputs cover
distinct used-filament densities/costs and a configured-but-unused filament.

The relevant failures are narrowing accumulated time before charging it,
changing material accounting while repairing time, charging an unused
filament, or accidentally widening preparation/block time. Preserve the
existing 64-bit bitwise comparisons for all four totals and the shared
volume and preparation-time controls. Persist the actual totals beside the
Native receipt location so each result is independently inspectable.

The existing Native JSON producer prints normal time with `%.9g`, not
with the `%.17g` needed to round-trip a double. Keep that source control
at its existing `f32` projection on both sides; the `%.17g` total-cost
comparison remains exact. Do not rebuild or change the producer, add an
epsilon, or add a precision oracle whose expected value is calculated by
Ares itself.

Run the affected comparisons in both Native modes. Also check the public
processor API controls, browser estimation serialization, workspace
formatting and Clippy, and wasm32 compilation. Mechanical consumer updates
must preserve their existing comparison precision and behavior.

## Deferred behavior

Unrelated configuration projection, toolchange and cooling differences,
other timing-model rewrites, and additional precision changes are outside
this slice. No new crate or independent estimation API is introduced.
