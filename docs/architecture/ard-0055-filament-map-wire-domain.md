# ARD-0055: Filament-map serialized enum domain

`PrintConfig.cpp:579-584::s_keys_map_FilamentMapMode` owns the serialized
values Auto For Flush, Auto For Match and Manual. The option's UI list at
2502-2516 additionally advertises Default, but ConfigOptionEnum/EnumGeneric
deserialization (`Config.hpp:1977-2092`) uses the key map, not that UI list.
The Rust destination is the existing `ProjectFilamentMapMode` and typed
project G-code options; it correctly rejects the UI-only Default spelling.
The CLI source-domain generator is a temporary validation shell around these
libslic3r serialization boundaries, not an independently defined option API.

For this option derive generated values from the actual upstream key map.
Do not add Default to the Rust enum, drop a valid wire value, or accept its
replacement by Auto For Flush as application proof. Live tests must retain
the raw requested value in the supplied process, prove each valid export,
and compare all three complete ordered G-code outputs. An explicit invalid
Default export is a negative application-proof case, not a fourth passed
enum value. Existing strict comparison and other option domains stay unchanged.

Final-config evidence also exposes plate precedence: the stock exporter
writes Auto For Flush into plate metadata, overriding the global request.
`bbs_3mf.cpp:4442-4449::_extract_model_config_from_mb_plate` owns concrete
plate tokens; the existing Rust destination is `project::plate_config`.
Bind the same validated concrete token in the generated single-plate fixture
before either slicer reads it. Retain strict proof for global and plate owners,
reject stale/missing plate values, content-address the amended project/cache,
and keep other archive entry payloads intact. Require the final G-code header
to carry the requested mode. ZIP representation may change; no G-code changes
or comparison normalization is permitted.

This slice includes the complete global serialized token set on the single-
filament fixture, not multimaterial mapping behavior. Defer GUI plate-inherit
sentinel behavior (outside the rewrite boundary), other plate metadata variants
and activation of multi-extruder assignment algorithms to their owning source
slices. `PartPlate`'s sentinel must not be invented as a libslic3r wire token.
