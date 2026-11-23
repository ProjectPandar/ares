# Web plater slice: no one-colour prime-tower rule

The browser slices the plater's `full_fff_config` (see `web-explicit-selection-gui-config.md`).
That document deferred one CLI-only step that still runs on this route: the one-colour
prime-tower rule. This slice removes it from the plater route. The CLI route keeps it unchanged.

Upstream is OrcaSlicer v2.5.0 (40ecc66f7).

## Upstream boundary

The rule lives only in `CLI::run` (`src/OrcaSlicer.cpp`):

- 2372: `disable_wipe_tower_after_mapping` starts false.
- 2455, 2468: `load_filaments_set` collects the preset names from `--load-filaments`.
- 2531-2535: the flag turns on only for a BBL 3MF loaded with at least one `--load-filaments`
  file, when every loaded file names the same preset.
- 3874-3893: more than one distinct non-empty CLI `filament_colour` turns it off again.
- 4293-4296, 4309-4314, 4316-4320: wrapping detection, a mixed filament or a smooth timelapse
  turns it off again.
- 4321-4343: otherwise `enable_prime_tower = false`, and `enable_prime_tower` is appended to
  `different_settings[0]`.

The plater has no such step:

- `Plater::priv::update_background_process` applies `PresetBundle::full_config` straight to the
  print (`src/slic3r/GUI/Plater.cpp:11429-11442`). For FFF printers that is `full_fff_config`
  (`src/libslic3r/PresetBundle.cpp:5463-5468`). Nothing in between reads the filament mapping.
- The only GUI code that turns the tower off based on the filament count is compiled out
  (`#if 0`, `src/slic3r/GUI/ConfigManipulation.cpp:537-594`).
- The tower then follows `Print::has_wipe_tower` (`src/libslic3r/Print.cpp:5189-5208`):
  `enable_prime_tower` with more than one `filament_diameter` entry.
  - With one filament the setting changes no toolpath.
  - With several filaments of one preset the plater keeps the tower.
- `Print::apply` normalizes the config before it prints it: `DynamicPrintConfig::normalize_fdm_2`
  (`src/libslic3r/PrintConfig.cpp:10478-10505`) clears `enable_prime_tower` when one filament is
  used, there is no mixed filament, the timelapse is not smooth and wrapping detection is off.
  - `PrintApply.cpp:2045, 2231` re-slice on the changed key. Neither changes
    `different_settings_to_system`.
  - This runs on both routes. So with one used filament the printed `enable_prime_tower` line is
    `0` on both, whatever `full_fff_config` says. The only visible difference the CLI rule makes
    is `enable_prime_tower` in dirty column 0.
- The saved project's config is `full_config_secure()` (`Plater::export_3mf`,
  `src/slic3r/GUI/Plater.cpp:20493`). A Native slice of it without `--load-filaments` never sets
  the flag (2531-2535).

## Current Ares behaviour

`Project::from_3mf_with_profiles` calls `mapped_prime_tower::disable_after_mapping` on both
routes (`crates/ares-core/src/project/domain/profile_input/three_mf.rs:129-135`).

- Every wasm entry uses `GuiProfileSliceRequest`, so the plater route is the one where
  `full_fff_config` is `Some`.
- On that route, one preset in every slot, with no distinct `filament_colour` overrides, turns
  `enable_prime_tower` off and records it.
- The explicit-selection slice added a branch for this route: a missing
  `different_settings_to_system` becomes `filament_count + 2` empty columns
  (`mapped_prime_tower.rs`, the `None` arm). Its `filament_count` parameter exists only for that
  arm.

Browser outputs at 3c24c3fef (GUI slice pairing) and 2f1e06b45 (GUI slice web suite) where the
rule fired. Each writes `enable_prime_tower = 0` and a dirty column 0 that contains
`enable_prime_tower`:

| Spec | Leg | Filaments | Plater session `full_fff_config` |
|---|---|---|---|
| `presetSlice.spec.ts` | 3MF, selected system presets | 1 × LHS PLA | `enable_prime_tower = 1`, no `different_settings_to_system` |
| `presetSlice.spec.ts` | 3MF, process edit | 1 × LHS PLA | same |
| `profileSlice.spec.ts` | 3MF override | 1 × LHS PLA | same |
| `printerSwitchFilaments.spec.ts` | U1 slice | 4 × Generic PLA @U1, one colour | same |
| `bedDefault.spec.ts` | browser slice | 1 × QIDI PLA Rapido | asserts only the bed commands |

The web references for the first four rows carry the rule:

- `cliSwitchedExport` (`web/e2e/platerExport.ts`) writes it into the plater export that Native
  slices.
- `withDifferences` keeps the `different_settings_to_system` line for the presetSlice and
  profileSlice legs. Their Native references are CLI `--load-filaments` slices of a BBL 3MF, where
  2531-2535 turns the rule on.

## Change

Rust destination: `crates/ares-core/src/project/domain/profile_input/`.

1. `three_mf.rs`: call `disable_after_mapping` only when `full_fff_config` is `None`, the CLI
   route.
2. `mapped_prime_tower.rs`: restore the dirty-column read to its shape before the
   explicit-selection slice. `different_settings_to_system` is always a vector here because the
   CLI resize creates it (OrcaSlicer.cpp:3202-3205). Remove the `None` arm and the
   `filament_count` parameter. The predicate and the dedupe stay byte-for-byte.

The CLI route does not change. `ProfileInput.enable_timelapse` stays `false` on the plater route.
This function no longer reads it there.

## Web references

Each reference follows the plater.

- `printerSwitchFilaments.spec.ts`, U1 slice:
  - `cliSwitchedExport` stops writing the rule and its guard on 4291-4320. It keeps the CLI
    machine-switch steps the plater route still runs (bed shrink, `curr_bed_type`).
  - Native slices the export as the plater saved it, so Native decides whether the four
    same-preset filaments print a tower (`Print.cpp:5205`).
- `presetSlice.spec.ts` 3MF legs and the `profileSlice.spec.ts` override leg: the Native CLI
  reference keeps its toolpaths. With one filament, `has_wipe_tower` is false
  (`Print.cpp:5205`).
  - A new helper takes the CLI reference to the plater's config lines. It rewrites one line:
    dirty column 0 loses the `enable_prime_tower` the rule appends last (4330-4340), and the line
    goes when every entry is empty (PresetBundle.cpp:5826-5838).
  - The `enable_prime_tower` line stays as it is: `normalize_fdm_2` prints `0` on the plater
    route too. The helper asserts that the reference has `0`, and that the session's
    `full_fff_config` meets every `normalize_fdm_2` condition: no mixed filament, timelapse not
    smooth, wrapping detection off. If any condition fails, the helper fails and does not
    rewrite.
  - It first asserts that the session has exactly one filament (`Print.cpp:5205`), so it never
    claims a toolpath it does not compare. It also asserts that the browser output differs from
    the reference only in the two tower lines.
  - An earlier version of this section took `enable_prime_tower` from the session's
    `full_fff_config` (`1`). That is the value before `normalize_fdm_2`, so it was wrong.
  - Every leg keeps `withDifferences(session)`, so the dirty line reaches the helper.
    `withoutCliGroups` with plain `session` would fail its empty-group assertion on the
    reference's `enable_prime_tower` entry before the helper runs. The helper's drop of an
    all-empty line gives the unedited leg the plater's missing line. The edited legs keep
    `layer_height`, because the process edit dirties column 0 (`PresetBundle.cpp:5586-5595`).
- The doc comments of `withDifferences`, `cliSwitchedExport` and `platerExport` drop the rule. The
  saved-project citation changes from Plater.cpp:19779 (that line is in `export_published_3mf`)
  to 20493 (`Plater::export_3mf`).

## Included and deferred

Included:

- the plater route without 2531-2535 / 3874-3893 / 4293-4343;
- the four web references above.

Deferred, unchanged:

- `shrink_to_new_bed` on a plater printer switch (as in `web-explicit-selection-gui-config.md`);
- the wrapping and mixed-filament branches on the CLI route (`mapped-prime-tower-fallbacks.md`);
- on the CLI route, Ares does not read `is_bbl_3mf` or the `--load-filaments` count (2531). Its
  predicate reads the selected profile's `filament_settings_id` instead. This is unverified.
  Whether a non-BBL 3MF, or a `--load-settings` run without filaments, differs from Native is a
  CLI-route question for its own slice.

## Verification

Order: docs, then test, then feat. Base is d651ba345.

- Test commit: the reference changes above.
  - At the base, the four browser legs fail on `enable_prime_tower` and
    `different_settings_to_system`. The U1 leg may also differ in toolpaths if Native prints the
    tower.
  - The feat commit makes them pass.
- Web pairing, base d651ba345 against the tip:
  - the three specs above, plus every spec that imports `platerExport.ts`;
  - P→F = 0; F→P only on the four legs.
  - `bedDefault.spec.ts` is in the set and its assertions do not change.
- CLI byte pairing, base against the tip:
  - `orca_parity` (load_settings / direct_stl / cli_saved_load_settings / project_catalog_cli,
    which includes `mapped_tower_keeps_tower_for_distinct_cli_colours`) and
    `cli_load_settings_progress`;
  - P→F = 0, and every Ares-written byte equal.
- `project_preset_selection` and ares-wasm Rust tests; fmt, clippy `--all-targets`, tsc, and lint
  equal to the base.
