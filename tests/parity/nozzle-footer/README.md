# Initial nozzle footer variant-column witness

`variant-column.3mf` changes only `Metadata/project_settings.config` of the
existing authored `m140-initial-bed/nonzero-initial-tool.3mf`. It is not derived
from Native exported G-code. All other ZIP entries and their bytes are retained.

SHA256: `908612c123f73537d6cd07b3baaf93e5c48e97f84503030944c330ac3327f23d`.

The source settings describe four filament variant columns:

| Source column | Filament self index (1-based) | Variant | Initial nozzle temperature |
|---|---:|---|---:|
| 0 | 1 | Direct Drive Standard | 211 |
| 1 | 1 | Direct Drive High Flow | 217 |
| 2 | 2 | Direct Drive Standard | 223 |
| 3 | 2 | Direct Drive High Flow | 237 |

`extruder_variant_list` contains Standard and High Flow, and the active
`nozzle_volume_type` is High Flow. Actual model activity is logical tool1.
Its source variant column is3, not logical ID1 or physical nozzle0. Existing
variant materialization selects source columns1 and3 to produce logical
`nozzle_temperature_initial_layer = 217,237` and `filament_self_index = 1,2`.
The expected compatibility footer is `first_layer_temperature = 237`.
Taking the reduced vector front would instead give217; indexing the original
vector directly with logical ID1 would also give217.

The source owners are `GCode.cpp:3681–3687,4894,5039–5053`,
`Print.cpp:5068–5071,5114–5147`, and `Config.hpp:676–680` at upstream revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`. Reuse the existing variant resolver
and shared nozzle reader; do not add another index formula. Public Native/API/
CLI checks compare the unique footer line and retain raw outputs and complete
first differences separately, without a whole-output parity claim.
