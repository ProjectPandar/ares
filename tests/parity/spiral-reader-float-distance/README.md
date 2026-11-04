# Spiral reader float-distance INPUT pair

Source: GCodeReader.hpp47–51 dist_XY(float operands/differences/products/
sum/sqrt), consumed by SpiralVase.cpp84–96/143–159 cumulative progress.
Rust boundary ONLY Movement::from::xy_distance. See
`docs/architecture/2026-10-21-spiral-reader-float-distance.md`.

These are actual official-exported INPUT3MFs only, copied unchanged:
control spiral_finishing_flow_ratio1, positive0. Both spiral1/smooth0,
ordinary six-layer24-sided tapered cylinder radius6→5.4mm/height1.2mm.
GUI Spiral finishing flow ratio is a coFloat with valid0..1 range.
No entry/final overlap, no Smooth XY formatting involved.

The manifest freezes whole archive/every uncompressed ZIP payload SHA256
and export provenance. Same ZIP inventory; only changed payload
Metadata/project_settings.config; only changed setting
spiral_finishing_flow_ratio1→0. All other payloads/settings identical.
No post-export JSON/mesh editing, expected G-code snapshots or Native
output content constants/assertions in tests/manifest.

Two tests directly call native_stream_case::check. Main's
existing shared helper double-replays Native and compares complete
validated API/bare CLI via native_api_cli. No duplicate replay/runner/
normalizer/JSON code or private output shortcut. Strict ordered bytes
allow ONLY existing generated-by identity/date normalization, never
M73/time/header/E/motion/config/trailing-line masking or tolerances.

ARES_REQUIRE_ORCA=1 and proper official/seeded Native
reference environment, repeatable artifact roots. Filter:

```
cargo nextest run -p ares-cli --test orca_parity --no-fail-fast \
  --test-threads 1 -E 'test(spiral_reader_float_distance_)'
```

Formal RED requires controlPASS/positiveactualFAIL with stable repeats
in both modes before production code. All streams/launches/
comparison errors remain in shared-helper artifacts. No ignored or
expected-failure test decoration. Disabled environment is NOT evidence.
Freeze tests/inputs/helpers for any future GREEN; field types/parsing/
reader storage/dist_E/other consumer rewrites and both previous spiral
points remain outside scope.

External source arithmetic replay/GDB discovery evidence is diagnostic, not
substituted for formal shared-checker public API/CLI RED evidence.
