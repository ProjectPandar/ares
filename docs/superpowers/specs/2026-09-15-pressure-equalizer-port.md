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

## Slice 4 + wiring plan (2026-09-15, #103)

The pass driver landed (`pass.rs`): layer text → parsed lines →
continuous extrusion segments with the ≤3mm small-gap bridge →
look-back-windowed `adjust_volumetric_rate` → marker re-emission, with
the upstream previous-layer buffering.

**Critical wiring discovery**: ares already has a `VolumetricRateSmoothing`
(`speeds/volumetric.rs:113-190`) — a pre-emission, acceleration-only
speed adjuster that never splits lines. It is NOT the upstream pass
(one-directional, no deceleration, no F-ladder re-emission,
layer-agnostic) and explains the 78-case divergence exactly. Wiring
plan (next slice):

1. Remove `VolumetricRateSmoothing` from speed generation (would
   double-apply).
2. When `max_volumetric_extrusion_rate_slope > 0`, the emission side
   inserts the markers (`GCode.cpp:6774` `;_EXTRUSION_ROLE:n` on role
   change; the `;_EXTRUDE_SET_SPEED`/`;_EXTRUDE_END` cooling markers).
3. The layer text runs through `PressureEqualizerPass` (previous-layer
   buffering + flush after the last layer).
4. The pass output has its `;_EXTRUSION_ROLE/;_EXTRUDE_SET_SPEED/
   ;_EXTRUDE_END/;_EXTERNAL_PERIMETER` tags stripped (upstream consumes
   them in the processor; the final exported G-code carries none —
   verified on the 2bDWHY oracle output).

## First wired sweep (2026-09-15, #107)

865 PASS / 108 DIVERGENT / 18 ORCA_ERROR (baseline 866/107/18) — the
pass is wired end-to-end and processing (78 slope-enabled cases flow
through it), but net -1: the modified-line F-emission structure still
diverges. Trace on the 2bDWHY layer-0 skirt: the limiter legitimately
fires on sub-unit f32 rate ties (forward-pass start clamp 4897.361 vs
4897.696 — the mm3/mm is constant but the f32 ratios differ), each
modified line re-emits through push_line_to_output → an F line per
segment; the oracle output shows ONE F per block for the skirt (its
lines appear unmodified) yet full F ladders on the inner wall.

Open question for the next slice: whether the oracle's raw input rates
are exactly equal (no ties) because its emission splits lines
differently, or whether the trivial-delta path should not re-emit F
when the quantized feedrate is unchanged. Also: revert-or-keep the -1
regression case (identify it via HEAD~1 comparison) pending the
structure fix.

## Decisive oracle experiment (2026-09-15, #108)

Re-sliced 2bDWHY with `max_volumetric_extrusion_rate_slope` 100 vs 0
(everything else identical):

- **slope=0**: layer-1 inner wall = raw emission (4 long lines, one
  F6000, no Z words, full-line E 4.39657); the ONLY differences vs
  slope=100 are E last-digit flips (~86 lines total).
- **slope=100**: layer-1 inner wall = the equalizer rewrite (split
  segments X498.92→X501.08→… each carrying interpolated E 1.46552, a
  Z1.2 word, and the F ladder F4320→4560→4980→5040→5280→5580→5700→6000)
  — the exact structure my emitter port produces.
- **The skirt stays UNREWRITTEN at slope=100**: orca emits one F3000
  for the whole skirt block, raw lines passthrough. My port rewrites it
  (sub-unit f32 rate ties → clamps fire). Identical input text,
  identical deterministic math — contradiction unresolved by reasoning.

Also: the wired-sweep net -1 is pure oracle-flake churn (Flashforge
Creator 5 ×4, WonderMaker ×3, MyToolChanger, Prusa XL 5T families;
+5 PASS −6 PASS +1 ORCA_ERROR) — NO stable-case regression from the
equalizer wiring.

Next slice: compile the vendored upstream PressureEqualizer
(tools/chain-probe has the sources) as probe6 and feed it the exact
ARES_DUMP_PEINPUT layer text — the byte-level verdict on where the
clamp decisions diverge.

## Post-dedup state (2026-09-15, #109)

Sweep 866/107/18 — the 866 baseline restored with the equalizer live
on all 78 slope-enabled cases. The full-fixture 2bDWHY (slowdown ON)
still diverges through the timing family: the pre-equalizer cooling
feedrate is F629 in ares vs F843 in orca (the #94 estimator gap —
10.26s vs ~14.1s internal layer-time belief), which then propagates
into every downstream F. The equalizer structure itself is byte-exact
(vindicated by probe6). Next slice: close the estimator gap or
continue on the other buckets.
