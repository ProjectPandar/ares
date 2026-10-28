# Overlong plate filament map

`x1c-cube-too-long.3mf` retains the existing one-consumable X1C cube with
prime tower disabled. Its plate map is `1 1`; the project has one nozzle
and one filament diameter. SHA256:
`3de8de84dee7bbbdb45530a0c247e79896f43ed253f69bb6a5020cc46375ca13`.

`OrcaSlicer.cpp:6027-6034` grows a short map with `1` but does not truncate
an overlong map. Single-extruder reset affects only `0..filament_count`,
not the tail. The Rust normalization owner is
`options/project_variants.rs::expanded_filament_map`.

The existing `runner::native_stream_case::check` qualifies two Native
replays before strict complete generator-only API/CLI comparisons and raw
API/CLI equality. This registration adds no opt-in switch, skip, mismatch
pin, or comparator exception. Official stability has been qualified for
this cube independently of the richer KSR's uninitialized-object-ID issue.
