# TiQ2 first-layer travel probe

Read-only `seed-replay` instrumentation extends `tiq2_order.nix`, which
reconstructs the exact seeded-oracle base derivation. No route, retraction,
simplification, or emission predicate is changed.

## Qualified travel probe

Package:
`/nix/store/f6x1jr6wqd5lzwwp4c8h8cjpybjka8wr-orca-slicer-seed-replay-2.4.2`.
The complete output is byte-identical to the seeded oracle after replacing
only the generated-by banner. No stream filtering or tolerance was applied.

The first-layer target has origin165,165, role1, raw start
(6834938,-5461327), end(4301461,4301461), external=0/support=0,
spacing437079 and one intersection. Lslices are the square±5000000,
safe zone±4712000, selected internal boundary±4400000.
Planner and final emission both contain exactly
[start,(4399900,4301461),end]. The intermediate world X is169.3999;
PRE_RETRACT and FINAL paths/last_pos are identical. Native wipe replanning
and final emission rounding therefore do not introduce this waypoint.

Next localize internal-boundary construction/cache history: current spacing
437079 would suggest a different ordinary1.5-spacing offset, so do not infer
the cache's construction input from its current lookup's spacing. Capture
first-layer get_boundary/init_layer events before selecting a Rust fix.

## Boundary and hypotheses

The single-apply phase candidate removes all 140 unexpected tower sections,
while the complete-byte TiQ2 test still differs at line 440:
`G1 X169.4 Y169.301 Z.6` Native versus `G1 X169.344 Y169.301 Z.6` Ares.
The preceding 439 lines match. Both footers retain enable=1.

Owner: `libslic3r/GCode.cpp:7350-7518::GCode::travel_to` calls active
`GCode/AvoidCrossingPerimeters.cpp:1233-1319::travel_to`; do not interpret
its later `#if 0` implementation as the active boundary. Rust destination:
`project_slice/gcode_emit/motion/path/{avoid_crossing,start_travel}`.

Ranked, falsifiable hypotheses:

1. Internal/external/once flags differ after support or wipe: the qualified
   trace will select a different boundary than the Rust caller.
2. Polygon construction differs: the selected Native boundary will differ
   from Rust's safe zone/internal/external input, or Rust will use its existing
   temporary rectangle shell. Compare exact integer polygons, not rounded XY.
3. Wipe re-planning or simplification differs: ENTER, PRE_RETRACT and FINAL
   raw points/last_pos reveal a changed start or changed routed interior point.

Only first-layer travels ending near G-code XY=(169.301,169.301) are logged.
The trace includes GCode raw and world-coordinate points, before/after
retract; active router start/end, support/external selection, spacing,
lslices/safe-zone/internal/external polygons and output before/after the
maximum-detour gate and external-coordinate translation.

```sh
probe=$(nix-build tools/orca-probes/tiq2_travel.nix --no-out-link | tail -n 1)
# Use a NEW output directory for every run.
ORCA_REPLAY_BIN="$probe/bin/orca-slicer" ORCA_REPLAY_SEED=0 \
  ORCA_REPLAY_CPU="$cpu" ORCA_DUMP_TIQ2_TRAVEL="$out/native-travel.txt" \
  scripts/orca-seeded-parity.sh --slice 0 --outputdir "$out" \
  tests/parity/validation/tiq2-type2-enabled.3mf
```

Select CPU from `/proc/self/status`, record source/derivation/package/input
identities, and require complete generated-by-only byte equality with oracle
`1k0p5nlbs3zsfdcy08g3y5fb7bcm1qrv` before accepting any probe state.
Instrumentation alone is not an accepted oracle expectation or a GREEN claim.
Input SHA256 is `0435346ed81fff32acdae7a3a3f2c9e880f4f1efe37d99d3427833836b901942`.
