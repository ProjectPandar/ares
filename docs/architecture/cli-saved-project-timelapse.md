# Smooth timelapse in the saved-project plate loop

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) reads the
`--enable-timelapse` action option once (`src/OrcaSlicer.cpp:1593–1595`). After
filament mapping, if that option is on and the shared `timelapse_type` is
`tlSmooth`, the CLI cancels the pending single-filament prime-tower disable
(`4302–4320`). It does not turn a disabled tower on or change the timelapse
mode. Each selected plate starts with that same shared configuration, then
applies its plate and extra overrides before calling `Print::apply`
(`6924–6926,6986`). The `src/libslic3r` boundary is the resolved print config
consumed by `Print::apply` and `Print::enable_timelapse_print`
(`Print.cpp:5235–5238`); the latter checks the mode, not the CLI flag.

## Rust destination

Pass the direct saved-project `--slice 0` flag from `ares-cli/src/direct_project.rs`
through the existing `SlicerContext` to
`ares-core/src/project_slice/cli_plates.rs::slice_cli_project_plates`. Apply
the action value to the existing profile input before shared profile loading
and before `fit_slice_wipe_tower` or `apply_plate` on any plate. Reuse
`project/domain/profile_input/mapped_prime_tower.rs` for the decision. The
separate per-plate context used by `PlateInput::Applied` is too late to
change preparation; do not run the decision again there or change G-code
emission to consult the flag. Preserve the single-plate path's existing
behavior and plate override precedence.

## Verification

Use the public CLI with a saved two-plate project generated from synthetic
STLs and stock-derived presets. Run Native and Ares with identical argv:
`--slice 0 --arrange 0` with smooth-mode, tower-enabled replacement presets
and one mapped filament. With `--enable-timelapse`, Native succeeds and the
config block in **both** plate G-codes retains `enable_prime_tower = 1`;
without the flag the same project and presets produce both plates with
`enable_prime_tower = 0`. Assert mode, tower state, exit status and output
inventory in both lanes. Compare the flag-on and flag-off cases in both
Native modes and keep the existing single-plate remap cases passing.

## Deferred

No changes to filament mapping, the mixed-filament or wrapping-detection
exceptions, plate placement, tower generation or G-code templates. Direct
STL and non-slicing CLI actions are out of scope.
