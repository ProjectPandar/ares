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

## Excluded behavior

Do not change wipe distance, other Type1 attributes, ordinary nozzle Wipe,
Type2, cooling, time estimation, configuration export, or comparators.
Wipe distance has a similar input pattern but no independent discriminating
output in this slice.
