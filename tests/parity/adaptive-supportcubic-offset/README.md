# Adaptive SupportCubic anchor offset witness

`mk4-supportcubic-d40.3mf` is the unchanged input-only archive from the
stable MK4 supportcubic40 discovery. SHA256:
`a6ee6f899f0f640ff4aaec35b12175cc59f29aab6ef7e26fed22a1a7e803095e`.

It uses the repository MK4 cube_with_hole scene, with spiral disabled and
`sparse_infill_pattern=supportcubic`, density40%, five top shells, two walls
and by-layer printing. No geometry or machine/filament setting was tuned to
repair output. This is a patched project-settings discovery input, not a
claimed new Native CLI export.

Upstream owner: `Fill/FillAdaptive.cpp:641-647` (`create_offset_line`
translate using `intersection.closest_line`) and `Line.cpp:121-126`
(`Line::extend` on the offset line's own direction) at pin
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. The stage2 coordinate
PINPOINT placed the first MK4 cursor-unit divergence at that anchor
translation and the subsequent `Line::intersection` cast.

The test is registered inside the existing Adaptive module. Run its
nextest selector under each official/seeded control set with
`ARES_REQUIRE_ORCA=1` and the shared persistent
`ARES_ORCA_CACHE_DIR`. It qualifies the Native replay and compares complete
Native/API/CLI streams using the generator-only strict comparator, plus
API/CLI raw equality. All artifacts go to the external artifact root; no
Native G-code snapshot is checked in.
