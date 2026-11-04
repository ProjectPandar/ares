# FanMover mixed-precision timing INPUT pair

Source FanMover.hpp19/46/57,float BufferData.time and double speed/buffer;
FanMover.cpp284–305,double line time and norm/division after float reader
subtraction. Exact Rust future boundary fan_mover.rs/motion.rs ONLY,
not all-f64. See docs/architecture/2026-10-21-fan-mixed-timing.md.

Actual official-exported INPUT3MFs unchanged: control fan_speedup_time0,
target1second, GUI fan spin-up lead time. Same32-sided12x6mm cylinder,
30layers,PLA220C,0.4nozzle,0.2layers,Classic2walls,15%crosshatch,
3top/bottom,fan30–70%,kickstart0,commentsOFF(default),only-overhangsOFF.
No custom probe injection or pathological numeric entry. Manifest records
whole/per-entry SHA, export receipts, identical inventory/only one payload
and one JSON key changed. No Native evaluated/output constants/snapshots.

Two DIRECT crate::runner::native_stream_case::check cases reuse main
NativeCase double replay then complete validated API/bare CLI comparison.
No duplicated replay/normalization/orchestration/helper/expected failure.
Run with explicit Native environments, ARES_REQUIRE_ORCA=1, retained
artifact roots, official then seeded:

```
cargo nextest run -p ares-cli --test orca_parity --no-fail-fast \
  --test-threads 1 -E 'test(fan_mixed_timing_)'
```

Require controlPASS/targetactualFAIL with stable Native repeats and both
public attempts; environment-disabled/setup failure is NOT RED credit.
Only existing generated-by identity/date normalization. No M73/header/
time/fan/E/XY/comment/config/trailing-line masks/tolerance/sorting.

Pair/tests/helpers stay frozen for GREEN. GREEN covers only
mixed-precision timing/promotions/narrowing at exact upstream boundaries,
not scheduling/E formatting/relative-E model/text/comments/flavor/global
Reader/processor/travel/nearest changes. Other files remain deferred; if
private consumers need changes outside the two-file boundary STOP/report.
External discovery evidence is NOT substituted for formal shared-checker
API/CLI results.
