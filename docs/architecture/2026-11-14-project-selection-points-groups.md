# Project preset selection: points-groups options

## Problem

The Always selection (`{"from_project": true, "load_and_select": "always"}`,
`docs/architecture/project-preset-selection-always.md`) fails on a printer whose project
config stores more than one group in a `coPointsGroups` option. Two-nozzle printers store
one printable area per extruder in `extruder_printable_area` (Bambu Lab H2D:
`["0x0,325x0,325x320,0x320", "25x0,350x0,350x320,25x320"]`; X2D:
`["0x0,256x0,256x256,0x256", "20.5x0,256x0,256x256,20.5x256"]`). The request is rejected
with `extruder_printable_area: Orca point contains an invalid coordinate`; OrcaSlicer's
plater opens the same project.

The selection compares preset and project values as typed options, as
`ConfigBase::diff` and `equals` do (`Config.cpp:531-555`; `diff` and `same` in
`crates/ares-core/src/profiles/project_selection/config.rs`, and `variants.rs`). It builds
each typed option from the Orca wire form (`config.rs`, `typed`), and for a list of groups
it joins the groups with `;`. The wire decoder (`decode_config_value`,
`options/decode.rs`; `split_groups` in `model_config_deserialize/wire.rs`) splits groups
on `#`, as upstream does, so the two groups are read as one and the point `0x320;25x0`
does not parse.

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f7`):

- `src/libslic3r/Config.hpp:1725-1735`, `ConfigOptionPointsGroups::serialize`: groups
  joined with `#`, the points of a group by `serialize_single_value` (`:1788-1796`) with
  `,`, each point `XxY`.
- `src/libslic3r/Config.hpp:1748-1773`, `ConfigOptionPointsGroups::deserialize`: splits
  groups on `#`, points on `,`, coordinates on `x`.
- `src/libslic3r/Config.hpp:1737-1746`, `vserialize`: one string per group.
- `src/libslic3r/Config.cpp:1032-1052`, `ConfigBase::load_from_json`: a JSON array of a
  `coPointsGroups` option is joined with `#` (`:1043-1044`) before `set_deserialize`;
  `coStrings` arrays are quoted and `;`-joined, other arrays `,`-joined.

## Destination

`crates/ares-core/src/profiles/project_selection/config.rs`, `typed`: a list of groups is
joined with `#`, the separator `ConfigOptionPointsGroups::deserialize` splits on. The
groups' points stay `,`-joined. The decoder and the Ares registry defaults are unchanged.

The test harness that writes a project config for the preset-select probe
(`crates/ares-cli/tests/project_preset_selection/support.rs`, `wire`) builds the request
values the way `load_from_json` reads the project's JSON arrays: `#` for points groups.

## Included

- Points-groups values of every collection the Always selection materializes.

## Deferred

- The CLI selection (`{"from_project": true}`): its refresh (`project_refresh/preset.rs`)
  does not build typed options through `typed`.

## Failure modes and how each is ruled out

1. **A multi-group value is rejected or merged into one group.** Synthetic projects whose
   printer is not installed carry the H2D and X2D `extruder_printable_area` values. The
   probe loads each; the test asserts the Native premise (the project-local printer is
   selected and its edited value is the two groups, `#`-joined, type 16394), then asserts
   that the Always API output holds one string per group, as `vserialize` lists them,
   equal to Native's.
2. **The other selection cases change.** The rest of the project preset selection cases
   run unchanged against the probe.

## Commits and acceptance

1. `docs(core)`: this document.
2. `test(core)`: the harness join and the H2D and X2D cases; both fail today with the error
   above.
3. `fix(core)`: the `#` join.

Checks: fmt, clippy, the wasm32 check, and the project preset selection cases with the
preset-select probe. The cases write the probe requests and results and the Always output
under `ARES_PARITY_ARTIFACT_ROOT`.
