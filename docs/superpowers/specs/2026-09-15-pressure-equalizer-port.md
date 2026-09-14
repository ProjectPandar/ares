# PressureEqualizer Port (extrusion-rate smoothing)

Status: PLANNED — the highest-impact remaining parity slice.

## Motivation (measured 2026-09-15)

**78 of the 107 DIVERGENT printers** in the printer smoke sweep set
`max_volumetric_extrusion_rate_slope > 0`. Upstream gate
(`GCode.cpp:2542-2545`): a positive slope instantiates
`PressureEqualizer`, a per-layer G-code text post-pass that re-chains
extrusion segments into rate-transition sub-moves (per-segment F
ladders, z-carrying lines, sub-print-resolution E deltas). The 2bDWHY
no-slowdown experiment isolated the emission signature (layer-1 inner
wall: orca 8 short F-laddered Z1.2-carrying segments vs ares 3 long
segments). The pass shifts every per-line time (M73 P/R sequences) and
the segment chain (geometry families) — porting it addresses the
majority of the remaining divergence in one slice.

## Upstream boundary (source-cited)

- `OrcaSlicer/src/libslic3r/GCode/PressureEqualizer.cpp` (955 LOC):
  the state machine — G1/G92/M-cmd parsing, position/E/F tracking,
  volumetric rate computation per extrusion role, the look-back window
  (`max_look_back_limit = 128`), gap tolerance
  (`max_ignored_gap_between_extruding_segments = 3` scaled units),
  rate-slope-limited segment splitting with F interpolation, and the
  layer-flush semantics (`process_layer(LayerResult&&)` returns the
  PREVIOUS layer's G-code).
- `PressureEqualizer.hpp` (211 LOC): `ExtrusionLine` ring buffer,
  `ExtrusionRateSlope` per-role limits (ironing excluded — slopes
  zeroed), `M73`/role-marker passthrough.

## Ares destination

A post-emission layer pass in `crates/ares-core/src/project_slice/`
(new module `pressure_equalizer/`, files <400 LOC, tests in `mod`):
the ares layer content pipeline already emits per-layer G-code text
(the cooling buffer rewrites feedrates in place); the equalizer runs
after emission and before the processor pass (matching upstream's
CoolingBuffer → PressureEqualizer ordering in
`DoExport::export_file`).

Included: the full state machine, the three option inputs (all already
in ares's registry: `max_volumetric_extrusion_rate_slope`,
`..._segment_length`, `extrusion_rate_smoothing_external_perimeter_only`),
relative/absolute E handling, per-role slopes, layer flush.

Deferred: `PRESSURE_EQUALIZER_STATISTIC` debug blocks.

## Verification gates

1. Golden + full ares-core suite green (comments-off fixtures
   byte-stable: the pass is a no-op when the slope option is 0 —
   matching upstream's constructor gate).
2. 2bDWHY no-slowdown fixture: layer-1 inner wall reproduces the
   F-laddered segment chain byte-for-byte.
3. Full printer sweep: expect a large PASS jump from 866 (78 enabled
   cases move; some may retain other divergences).
