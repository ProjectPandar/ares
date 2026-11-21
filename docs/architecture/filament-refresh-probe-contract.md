# Native filament refresh probe contract

## Source and destination

Use OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).
`CLI::run` in `src/OrcaSlicer.cpp:2539–2817` sets up the project refresh
routes, including the decision to refresh unlisted values and the per-slot
filament lookup. `PresetBundle::resolve_system_preset`
(`src/libslic3r/PresetBundle.cpp:724–759`) calls
`PresetBundle::find_preset_vendor` (`:403–482`) to locate the system preset
in the supplied catalog.

The boundary is `crates/ares-cli/tests/project_filament_refresh.rs`: compare
Native `--export-settings` and its refresh diagnostics with the existing
expectations, and compare the public Ares `profile_settings` result for the
same project and catalog. The Ares refresh implementation is unchanged.

## Oracle and scope

Set `ARES_ORCA_FILAMENT_REFRESH_BIN` to the same CLI executable (ELF) used by
`ARES_ORCA_PRESET_SELECT_BIN`. Invoke its ordinary CLI route, not the
preset-selection request hook. The existing `ARES_ORCA_RESOURCES_DIR` hook
points Native at the test's synthetic profile tree.

CPU affinity is a runner isolation setting, not part of filament refresh
behavior. The runner controls affinity and supplies `ORCA_REPLAY_CPU` for
seeded replay. The test neither requires nor reads a separate affinity file.

Run all existing refresh cases with the official and seeded reference
configurations. Keep the same fixtures, exit-status, exported-value,
diagnostic, and Ares API assertions. Changes to refresh behavior, comparison
semantics, and runner configuration are outside this slice.
