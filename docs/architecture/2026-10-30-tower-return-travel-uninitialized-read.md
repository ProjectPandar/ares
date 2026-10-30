# ARD: Type1 tower return travel reads uninitialized upstream state

Date: 2026-10-30
Status: accepted
Scope: `change_filament_gcode` scripts that contain no X/Y motion
words, on prints with a prime tower (`is_used_travel_avoid_perimeter`).

## Decision

Ares does not reproduce the upstream uninitialized-memory read that
drives the Type1 tower return travel for scripts without X/Y words.
Fixtures in that class are an upstream-UB no-witness boundary. Ares
keeps its semantically defined behavior: the parsed start is
zero-initialized and, when the segment to the ray hit does not cross
the avoid box, no entry is found and the travel is the direct
two-move route.

## Upstream boundary

- `GCode.cpp:1010` declares `Vec3f gcode_last_pos;` without
  initialization (Eigen default constructor leaves memory untouched).
- `GCodeProcessor.cpp:3021` (`get_last_position_from_gcode`) writes
  X/Y only when the toolchange text contains literal ` X` / ` Y`
  words; the remainder of the value is whatever the stack slot held.
- `GCode.cpp:1034` passes the value into
  `generate_path_to_wipe_tower` (`:594-710`), where the segment from
  the parsed start to the ray hit decides the entry point and the
  around-the-box polyline.

## Observed Native behaviour (deterministic in practice)

Across repeated runs and across both oracle binaries, the leftover
value is stable: the Y component alternates between exactly minus
132.0 mm and minus 120.0 mm by toolchange parity, and the X component
converts to the most negative scaled integer, consistent with a
NaN/garbage float cast. The resulting routes are byte-identical
between the two Native builds; the only inter-build difference is a
one-second estimate variance that shifts remaining-time lines.
Reproducing these routes byte-exactly would require encoding that
stack garbage into Ares, which this decision forbids.

Where the toolchange text does contain X/Y motion words (semantic
case), upstream is fully defined and Ares matches it byte-exactly,
including estimates and remaining-time lines.

## Consequences

- No travel-fix slice; `wipe_tower/type1/travel.rs` stays as the
  semantically defined port.
- Parity fixtures for toolchange scripts on tower hosts must include
  an X/Y motion word in the change script (towerless hosts are
  unaffected either way).
