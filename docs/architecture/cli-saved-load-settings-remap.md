# Direct CLI preset replacement on a saved project

Native reference: OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).

## Upstream behavior

- `src/OrcaSlicer.cpp:1624–1640,1682–1690` loads the saved 3MF and its
  configuration and checks embedded post-process scripts before reading
  replacement preset files. `:2280–2358` classifies loaded machine and
  process presets, and `:2420–2457` reads the loaded filament config and
  remembers its preset name and slot. `:2925–2953` checks process/machine
  compatibility. `:3099–3193,3227–3256,3376–3406` overlays supplied machine
  and process settings onto the saved configuration, while `:3518–3615`
  handles the selected filament names, variants, and per-slot settings.
- `:2528–2535` keeps the number of saved filament slots unless it was zero
  and enables the post-mapping tower disable on a BBL-format project when
  the nonempty loaded filament list contains only one distinct preset
  name. This is not the count of physical extruders. The project's plate
  filament count for bounds is taken from the assigned object extruders
  (`:4540–4571`), rather than treating every loaded preset as used.
- `:3874–3891,4286–4320` can cancel that disable for different explicit
  colors, wrapping, mixed filament, or the CLI Smooth timelapse condition.
  Otherwise `:4322–4342` sets `enable_prime_tower=false` and records the
  process dirty key once. `:6924,6986` applies the resulting settings to
  the plate's `Print`. The adapter does not implement tower geometry or
  force on a disabled tower.

## Rust boundary and verification

Connect the direct CLI saved-project request to the existing
`slice_with_load_settings_processed` entry for one machine, one process,
and explicit filament presets on one BBL-format plate. That entry already
uses `compose_project_load_settings`, variant and slot overlays in
`Project::from_3mf_with_profiles`, and `mapped_prime_tower` after CLI
option bookkeeping. Retain the saved-project CLI's existing output,
reporting and placement handling; do not create a parallel merge of raw
settings. The `ares slice` subcommand already has its own adapter for the
same core composition and is not being replaced.

Use an exported synthetic, single-extruder block. Distinct machine and
filament G-code markers and a changed process setting establish that all
three replacements reached the slice. Use an enabled prime tower and a
Smooth process without the CLI timelapse switch so that the single-preset
mapping disable is observable even on one filament; compare with the
same saved project without replacement. Compare identical Native and Ares
argv, complete G-code, exit, stderr, complete result JSON, and recursive
working-directory inventory with the usual volatile metadata normalization.

## Deferred behavior

Automatic rearrangement, machine or bed changes, multi-plate projects,
partial preset lists, catalog refresh, mixed filaments, and the CLI timelapse
switch itself are outside this slice.

The synthetic replacement filament uses JSON arrays for its string vectors.
Native also accepts scalar-string representations: `OrcaSlicer.cpp:2205–2217`
loads preset JSON; `Config.cpp:984–987` forwards a string to `set_deserialize`.
`PrintConfig.cpp:3053,3069,3468` define `default_filament_colour`,
`filament_notes`, and `filament_ramming_parameters` as `coStrings`.
`Config.hpp:1225–1229` and `Config.cpp:173–235` decode semicolon-separated
members, recognize double-quoted members, and remove the quotes and expand
escapes inside them. `GCode.cpp:8370–8378` writes the typed value using
`opt_serialize`, which calls the vector serializer (`Config.hpp:1207–1210`,
`Config.cpp:99–148`) to escape the decoded members. Ares currently retains the
input quote characters in these three scalar-encoded vector members and then
escapes them again in the G-code configuration footer. That decoder difference
is separate from saved-project preset dispatch.
