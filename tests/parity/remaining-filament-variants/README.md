# Nullable float filament variant inputs

Source boundary: `PrintConfig.cpp:8869–8937,8971–8977,10153–10161`,
`PrintConfig.hpp:1925,1927–1931,2129`, and
`PrintConfig.cpp:11862–11994::gather_option_values` / static dispatch.

Each archive derives from unchanged `../variant-retraction/l59.3mf`
(SHA256 `908612c123f73537d6cd07b3baaf93e5c48e97f84503030944c330ac3327f23d`).
Only six added keys in `Metadata/project_settings.config` differ. Every other
archive entry, its contents, the variant map, tool activity and source-index
layout remain unchanged. High Flow selects source columns1/3 into logical
slots0/1 on the same physical nozzle.

Keys in table order are `filament_preheat_temperature_delta`,
`filament_ramming_travel_time`, `filament_ramming_travel_time_nc`,
`filament_ramming_volumetric_speed`, `filament_ramming_volumetric_speed_nc`,
and `filament_retract_length_nc`. They are nullable floats, not integers.

| Archive | Input shape | Native logical values, in key order |
| --- | --- | --- |
| `distinct.3mf` | Independent four-entry fractional vectors | `-2.5,-4.5`; `2.5,4.5`; `6.5,8.5`; `22.5,24.5`; `32.5,34.5`; `12.5,14.5` |
| `mixed.3mf` | Three-entry vectors with missing source column3 | `0,nil`; `nil,0`; `0,nil`; `nil,-1`; `-1,nil`; `nil,0` |
| `all-nil.3mf` | Three explicit zeros for preheat; three nils for each other key | `0,0`; the other five keys are omitted |

The ragged three-entry shape bypasses singleton/per-filament load layout
(`PrintConfig.cpp:11431–11457`), isolating original-front fallback at column3.
Nil remains distinct from zero and the speed sentinel `-1`; ordinary nullable
serialization determines line spelling and omission.

Existing default inputs are reused without copies: the four-slot Ender PLA
archive at `../auto-brim-object-adhesion/auto_brim-pla.3mf` and the two-slot MK4
archive at `../mk4-layer-change/mk4base.3mf`. Their expected six-key defaults
are respectively four or two repetitions of `0`, `0`, `0`, `-1`, `-1`, `10`.
