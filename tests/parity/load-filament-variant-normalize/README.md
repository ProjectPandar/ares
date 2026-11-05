# Load-time filament variant layout

Source: beta `345efec64c7b95a05ce5cf96df0dde66a158835c`, `PrintConfig.cpp:11431-11457`,
called by `handle_legacy_composite` (`PrintConfig.cpp:10094-10096`) and by
`CLI::run` after the defaults merge (`OrcaSlicer.cpp:4250-4259`).

`absent-variant-keys.3mf` is `../auto-brim-object-adhesion/auto_brim-pla.3mf`
(SHA256 `ec2304a186a04bc1dcd0a7ad2497003f0c209fafd78eb5a66c90f044fc9ddb31`) with
the `activate_air_filtration` and `during_print_exhaust_fan_speed` blocks removed
from `Metadata/project_settings.config`. Every other ZIP entry is byte-identical.
SHA256 `a8e7658679fafb80e11a73ca7b5025519e2621ad97362e34a85282f5e6f203c5`.

The project lists `filament_self_index = [1,1,1,1]`. The two removed keys reach
Native only as their single defaults, which the second call lays out to
`0,0,0,0` and `60,60,60,60`.
