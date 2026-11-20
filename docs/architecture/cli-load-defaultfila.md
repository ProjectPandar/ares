# Default filament for explicit list gaps in direct STL slicing

## Upstream boundary

Pinned OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):

- `PrintConfig.cpp:12799–12802,13078–13082` declares the false-by-default boolean and the filament list. `OrcaSlicer.cpp:1585–1587,2373–2469` selects the first nonempty filename when the flag is on, uses that loaded preset for every empty entry in the list (even before the first filename), and retains the positions of the other files. The flag without a list has no effect; a nonempty list with no filename is a configuration-file error.
- With the flag off, an empty entry does not load a preset; the position still counts. For fresh STL, `OrcaSlicer.cpp:2528–2529` takes the logical filament count from the list, not from the nozzle count. `:3566–3622,3815–3825` composes the selected slots with their own identity and options. An explicit replacement is not a project refresh.
- `PresetBundle::construct_full_config` (`PresetBundle.cpp:276–324`) likewise composes selected filament presets per position. `Preset::normalize` (`Preset.cpp:554–589`) normalizes option vectors for the configured filaments or extruders; it does not make the CLI list longer. `PresetBundle::min_filament_count` (`PresetBundle.cpp:5419–5436`) belongs to the preset selection UI, not this CLI list parser.

## Rust destination

The seam is `ares --slice 1 --outputdir …` with one STL. `args.rs` declares `--load-defaultfila`; `direct_stl.rs` resolves explicit empty list entries before passing the file list into the existing `profiles::load_settings` and `Catalog::compose` boundary. Keep the same slot indices and use the same preset validation and composition as an explicit repeated filename. Without the flag, do not substitute a filename. Do not infer entries beyond the end of the list.

Authored IDs for admitted inherited user files use the adjacent rule in `docs/architecture/cli-user-filament-authored-id.md` (`OrcaSlicer.cpp:2449–2453,3616–3622`). That rule is about explicitly loaded metadata, not how empty slots select a file. User-file admission itself (`OrcaSlicer.cpp:2113–2161`, `PresetBundle.cpp:556–650`) is also separate. Neither admission nor ID extraction changes here.

Defer saved 3MF, including its trailing project slots and system refresh (`OrcaSlicer.cpp:2781–2817`), the `slice` subcommand, other model formats, STL batches, and GUI tool-count normalization. A short list is not treated as though it contained explicit trailing blanks.

## Verification

Use a synthetic STL and standalone typed filament files. Compare a sparse list to its explicit expansion with the flag on, and verify that the flag does not affect a dense list. Compare exit status, stderr, the result report (excluding wall-clock fields), and complete G-code (excluding its generator banner) against live Native. Include a nonempty list with only empty entries to check the configuration-file failure and its report without G-code.
