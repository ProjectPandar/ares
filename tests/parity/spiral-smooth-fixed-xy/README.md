# Smooth Spiral fixed-XY input pair

Source rewrite slice: `GCode/SpiralVase.cpp:182–183` smooth X/Y setters →
`GCodeReader.hpp:57` float/default3 → `GCodeReader.cpp:323–326` fixed3.
Rust boundary is project.rs::format_axis; no overlap-state rewrite.
See `docs/architecture/2026-10-21-spiral-smooth-fixed-xy.md`.

These are INPUT3MFs only, not expected G-code snapshots. Closed8-sided
cylinder radius6mm/height1.2mm, ordinary6layer GUI-importable mesh. Both
spiral_mode1; GUI Smooth Spiral/spiral_mode_smooth is the only changed
setting,0→1. Both actual official-exported archives are copied unmodified.
`manifest.json` freezes archive and every uncompressed ZIP payload SHA256,
records export provenance, and proves identical ZIP inventory and all
payloads except Metadata/project_settings.config (whose sole-key delta
is spiral_mode_smooth). No Native output/content constants in tests or
manifest. All ordinary shared settings remain part of the frozen input.

Discovery provenance/launches preserved in
`${COORD_ROOT}/evidence/w5-spiral-smooth-fixed-xy/`.
No post-export JSON/mesh editing; export source generation is retained
externally in original-export-receipts and w5-discover-smooth2.py.

Two tests in orca_parity/spiral_smooth_fixed_xy.rs call existing
native_stream_case::check DIRECTLY: two Native launches per case with
full repeat comparison, then main native_api_cli public validated API
and bare CLI comparisons. Full ordered output allows ONLY existing
generator identity/date normalization, not M73/XY/E/header/trailing-line
filtering or tolerances. All outputs/launches/errors are retained by
that helper; no duplicated orchestration or manual-normalization test.

Formal RED must show off-controlPASS/on-positiveFAIL in both reference
modes and stable Native replays. This is a textual-formatting gap, not
changed numeric motion/material; nine numeric low-bit input differences
are deferred. No numeric normalization may turn the positive into PASS.
Run CPU11/thread1/job1 with ARES_REQUIRE_ORCA=1, appropriate official or
seeded Native reference environment, artifact roots, and filter:

```
cargo nextest run -p ares-cli --test orca_parity --no-fail-fast \
  --test-threads 1 -E 'test(spiral_smooth_fixed_xy_)'
```

Do not treat an environment-disabled test as parity evidence. RED after
SPEC requires review before any production change. Future GREEN freezes
these inputs/tests/helpers; no widening to geometry/Point2/header/
processor/overlap state or shape-specific workaround. No FULL here.
