# Process variant column Standard control

`ender3-standard-nozzle.3mf` changes only `nozzle_volume_type` from `High Flow` to
`Standard` in `Metadata/project_settings.config` of `nozzle-footer/variant-column.3mf`.
Every other ZIP entry keeps its bytes and the entry order is unchanged. It is an input
only; no Native output is stored.

SHA256: `036316b8d1fd464e9209a6efb85b71396aa197eb8cb0798ead31eec7a18133ca`.

The Creality Ender-3 project keeps length-1 process columns
(`print_extruder_id = 1`, `print_extruder_variant = "Direct Drive Standard"`) and
`extruder_variant_list = "Direct Drive Standard,Direct Drive High Flow"`. Upstream
synthesizes the two process columns from that list (`PrintConfig.cpp:11623-11660`) and
selects the Standard column for the Standard nozzle, so the process label stays
`"Direct Drive Standard"`, while the High Flow project selects `"Direct Drive High Flow"`.
