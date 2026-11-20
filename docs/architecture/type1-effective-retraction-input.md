# Type1 effective retraction inputs

## Upstream boundary

This slice rewrites OrcaSlicer v2.5.0 Type1 tower configuration admission:

- `PrintApply.cpp:295–317` selects filament-prefixed retract overrides.
- `PrintConfig.cpp:12356–12387` applies nullable overrides to cloned machine
  options with the filament mapping.
- `PrintApply.cpp:1665–1677` installs effective values in `Print::m_config`
  while retaining the original full configuration separately.
- `Print.cpp:5291–5295,5332–5333` constructs Type1 `WipeTower` and calls
  `set_extruder` with that effective configuration.
- `GCode/WipeTower.cpp:2080,2206–2207` stores each filament's effective
  retraction length and speed.
- `GCode/WipeTower.cpp:4252–4253,4356–4389` consumes the pair at wipe entry;
  `:5359–5382` and writer `:1367–1423` consume it between wall paths.

## Rust destination

`options/project_config_views/retract.rs` already produces runtime effective
length and speed. Reuse it without duplicating nullable or mapping logic.
No Type2 dependency is needed.

Prepare Type1 filament state where both full and runtime views are available
in `wipe_tower/type1/planning.rs`, then transfer it to the generator. Supply
only the runtime length/speed pair to `type1/filament.rs::Filament::new`;
all other filament attributes retain their full-config input. Entry and
wall consumers continue using the same stored fields. This replaces the
Type1 input compatibility shell with the upstream effective admission,
not a separately designed retraction algorithm.

## Public output checks

Derive two synthetic inputs from the public dual-cube Type1 tower project
`tests/parity/bbl-type1-tower/enabled.3mf`. Scale both cube instances to
one 0.2 mm layer to isolate entry and gap-wall behavior from later-layer
tower scheduling differences. Both use machine length 0.8 and speed 30. One uses filament overrides 4 and 17; the other uses nil overrides.
Compare the complete ordered Type1 tower retract/unretract line sequence
from live Native slicing with public Ares API slicing, including every
entry and wall-path retract, not selected samples. Exclude the toolchange
template and ordinary travel retraction before `CP_TOOLCHANGE_WIPE` inside
`CP TOOLCHANGE START`/`END`; those are not the tower writer's consumers.
Outside that toolchange preamble, collect every E-only move between
`WIPE_TOWER_START`/`END` markers. Require nonempty tower
output, paired retractions and restorations, and actual override activation
in Native. Retain the input, both outputs and extracted sequences.

These checks do not claim whole-stream equality: existing estimated-time
and other differences in the base project are outside this slice. The
sequence assertion is additional behavior-specific coverage, not a change
to any existing comparator or whole-stream assertion. Run both cases with
official and seeded Native configurations, plus the existing tower-related
case inventory with no passing case becoming a failure.

## Effective wipe distance extension

`Print.cpp:5332–5333` also admits effective wipe distance into
`GCode/WipeTower.cpp:2080,2208`. The consumers are `finish_layer:3984–3985`
and `only_generate_out_wall:5296`, which call `add_wipe_path:1355–1365`.
That writer collects polygon points for a future wipe, stopping immediately
for zero distance. `GCode.cpp:1652–1657` installs the resulting path and
retracts after leaving the tower. Reuse the existing merged
`runtime.project.print.wipe_distance` at the Type1 filament constructor;
do not change polygon path generation or ordinary motion retraction.

A separate public API test derives a single-layer smooth Type1 project from
the same public input. Machine distance zero plus filament distance five
must produce the complete Native first departure wipe block. Machine five
plus nil filament distance is the equal-effective control. Select the first
`WIPE_TOWER_END`, then collect the entire `WIPE_START` through `WIPE_END`
block before the next ordinary XY travel. This excludes object wipes and
later tower results with unrelated scheduling differences. Require Native
nonempty, equal Native/API line counts, and complete ordered line equality;
retain inputs, outputs and extracted blocks. Do not claim whole-stream
parity. Run both cases in official and seeded modes and the existing Type1
and tower inventory without a passing case becoming a failure.

Derive the synthetic archives at test runtime; do not alter the existing
fixture or commit a generator script. Use fixed ZIP timestamps, require
empty creation/modification dates, and reject local source paths in archive
metadata. Stock printer-template revision dates are not fixture provenance.

## Excluded behavior

Other Type1 attributes, ordinary nozzle Wipe, Type2, cooling, time estimation,
configuration export, polygon wipe generation and comparators are unchanged.
