# Direct BBL projects with unresolved system filaments

This addendum to [direct filament refresh](direct-bbl-filament-refresh-spec.md)
supersedes its missing-bundle rejection contract. Native reference:
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

## Included

`OrcaSlicer.cpp:2793–2817` resolves system filament presets one slot at a
 time. At `:2803–2807` a lookup miss logs that the filament keeps its values
and continues. With no catalog, none of the slots can resolve: preserve the
embedded values and continue through ordinary project validation and slicing.
Do not invent a fatal error, substitute a catalog, or make the native lookup
log a slicing notification. `:2820–2825` likewise skips an unavailable upward
machine resource; leave that independent lookup unchanged.

In `ares-core/src/project_slice/source.rs`, run the existing direct refresh
only with a supplied catalog. The no-catalog path does not reapply resolved
presets or bypass project validation. Supplied catalogs still refresh
resolvable slots, preserve individual misses, and report malformed inputs as
before. Explicit replacement and project-derived selection are unchanged.

## Public verification

- Keep `cli_upward_machine_result::invalid_values::cli_invalid_values_preserves_upward_machines`
  and `cli_invalid_values_dump_moves_preserves_upward_machines` unchanged:
  neither passes `--resources-dir`. Compare Native's invalid-values result,
  including upward machine metadata, and verify that no G-code or moves leak.
- Change only the obsolete no-catalog rejection in
  `ares-core/tests/profile_filament_self_index.rs::direct_bbl_project_without_catalog_returns_missing_bundle_error`:
  the public direct API now slices and retains the embedded density, flow and
  diameter. Existing explicit-catalog tests remain unchanged.
- Change only the obsolete no-catalog assertion in
  `ares-wasm/tests/direct_project_catalog.rs::direct_bbl_project_requires_explicit_catalog_and_slices_with_supplied_bundle`:
  omission and an explicitly empty bundle produce identical G-code. Keep the
  supplied-bundle leg and its artifact. These are the two tests asserting the
  removed error.

## Deferred

Default CLI resource discovery and packaging (`OrcaSlicer.cpp:8169–8197`),
resolver diagnostic transport, and any changes to the explicitly supplied
catalog or machine-resource lookup are separate work.
