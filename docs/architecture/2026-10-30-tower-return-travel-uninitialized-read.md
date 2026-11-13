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

At upstream `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `src/libslic3r/GCode.cpp:1521-1524` declares `Vec3f gcode_last_pos;`
  without initialization, then reads its X/Y components after parsing.
- `src/libslic3r/GCode/GCodeProcessor.cpp:4331-4393`
  (`get_last_position_from_gcode`) writes X/Y only when it parses the
  corresponding ` X` / ` Y` words in motion commands; absent axes
  retain uninitialized memory.
- `GCode.cpp:1538` passes that position into
  `generate_path_to_wipe_tower`, where the segment from the parsed
  start to the ray hit decides the entry and around-the-box polyline.

## Observed Native behaviour

Earlier repeated runs across both oracle binaries observed a stable
leftover value: the Y component alternated between exactly minus
132.0 mm and minus 120.0 mm by toolchange parity, and the X component
converts to the most negative scaled integer, consistent with a
NaN/garbage float cast. The resulting routes are byte-identical
between the two Native builds; the only inter-build difference is a
one-second estimate variance that shifts remaining-time lines.
Reproducing these routes byte-exactly would require encoding that
stack garbage into Ares, which this decision forbids.

That earlier stability is not a determinism guarantee. Later official
2.5.0 self-replays of the BBL no-selection fixture selected different
return routes and M73 positions, including under single-CPU affinity.
A retained Prusa macro replay pair also differed at the tower-return
route. These observations and the source read identify the UB boundary;
they do not measure the official binary's uninitialized operands or
establish a load/thread cause. M73 remains part of the comparison.

Tower-host scripts must provide parseable values for both X and Y to
avoid this particular uninitialized read. A towerless host avoids the
Type1 return path without changing the script.

## Consequences

- No travel-fix slice; `wipe_tower/type1/travel.rs` stays as the
  semantically defined port.
- Parity fixtures for toolchange scripts on tower hosts must initialize
  both X and Y in the change script, or use a genuinely towerless host.

## Towerless fixture repair

`Config.cpp:985,1008,1060-1063` (`load_from_json`) accepts string and
array values but rejects a numeric scalar. Both
`tests/parity/m1020-filament-change/towerless-change.3mf` and
`tests/parity/template-macro/prusa-xl-change-filament.3mf` encoded
`enable_prime_tower` as numeric `0`; retained Native output exported
`enable_prime_tower = 1`. Their no-XY scripts therefore reached this UB
boundary despite the intended towerless premise.

Change only that field's wire type to string `"0"` in each fixture.
Keep both scripts byte-identical and synchronize all three frozen-hash
consumers. The Prusa fixture is an X1 Carbon host carrying the verbatim
XL script, not a complete XL machine profile; it retains the no-XY,
towerless contract in `placeholder-macro-block-statements.md:77-85`.
The BBL selection and Ender3 sibling fixtures already use string `"0"`
and remain unchanged. M1020/nozzle, tool-selection, macro-block,
API/CLI and whole-stream assertions and comparators remain unchanged.

Validation requires six consecutive consistent official Native outputs
per new input under the existing generator-line normalizer, effective
`enable_prime_tower = 0` with no tower segments, and all five related
cases in official and seeded modes. Seeded output cannot establish
official Native stability. This is a test-input repair, not a travel
implementation change or an emulation of upstream UB.
