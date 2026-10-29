# Lightning initial-overhang Polygons witness

`a1-lightning15.3mf` is the unchanged input-only archive from the approved
stable A1 lightning15 discovery. SHA256:
`fbba9c87450ad465300b3d522912068e9625395faf3feed937eaa4267f54c067`.

It uses the repository A1 cube_with_hole scene, with spiral disabled and
`sparse_infill_pattern=lightning`, density15%, five top/three bottom shells,
two walls and by-layer printing. No geometry or machine/filament setting
was tuned to repair output. This is a patched project-settings discovery
input, not a claimed new Native CLI export.

Upstream owner: `Fill/Lightning/Generator.cpp:166` and
`ClipperUtils.cpp:407-408` at pin
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.
The full architectural boundary is
`docs/architecture/ard-lightning-initial-overhang-polygons.md`.

The test is registered inside the existing Lightning module. Run its
`test(orca_parity_lightning_initial_overhang_polygons_a1)` nextest selector
under each official/seeded control set with CPU18, one build/test/Native
thread, `ARES_REQUIRE_ORCA=1` and the shared persistent
`ARES_ORCA_CACHE_DIR`. It qualifies two Native replays and compares complete
Native/API/CLI streams using the generator-only comparator. All artifacts
go to the external artifact root; no Native G-code snapshot is checked in.
