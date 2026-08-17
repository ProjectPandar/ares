# Spec: Task 22O.152 retained-polyline fitted-arc wipe

## Observable contract

After the first-layer fitted outer-wall arc ending at `G2 X145.533 Y94.206 I-2.179 J-1.06 E.02577`, retraction follows the source polyline rather than the reduced arc-command endpoints:

```text
; WIPE_START
G1 X145.621 Y94.523 E-.1318
G1 X145.756 Y94.862 E-.14599
G1 X145.814 Y95.162 E-.12221
; WIPE_END
G17
G3 Z.6 I-.612 J-1.052 P1  F60000
```

The points come from generated perimeter geometry and the 3MF `wipe`, `wipe_distance`, retraction, role-based wipe speed, and lift options. Production code must not inspect fixture names or reference G-code.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:426-496,5978-5991`: fitted G2/G3 commands reduce emitted motion, but `m_wipe.path` retains every clipped `ExtrusionPath::polyline` point. Wipe-distance clipping and proportional retraction then traverse that retained polyline from the actual post-extrusion position.

Variable-speed processed paths, cooling, timing, object identifiers, and later G-code differences are deferred.
