# Orca float spelling in sliced 3MF plate metadata

## Upstream and Rust destination

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) writes
`Metadata/plate_<n>.json` with `nlohmann::json::dump()` in
`src/libslic3r/Format/bbs_3mf.cpp:7048–7053`. `BBoxData::to_json` and
`PlateBBoxData::to_json` (`src/libslic3r/GCode/ThumbnailData.cpp:8–43`) populate
that object. nlohmann's default `number_float_t` is `double`
(`deps_src/nlohmann/json_fwd.hpp:33`), including when a field originated as a
`float`. The shortest round-tripping spelling alone is not the contract:
`645.1599731445313` and `645.1599731445312` can denote the same binary64 value.
The archive must contain the spelling selected by Orca.

The destination is `crates/ares-core/src/project/export`: a private number
formatter used by `xml.rs::plate_json` for both single and arranged sliced
exports. The default float spelling in `serde_json::to_vec` is the temporary
substitute being replaced. Keep the typed `Serialize` structs, field order,
string escaping, and compact layout. Serialize directly using
`serde_json::Serializer::with_formatter`: its `Formatter` trait already exposes
`write_f64` and `write_f32`; the latter promotes to `f64` before applying the
same formatter. Do not round through a `serde_json::Value`, hand-write the JSON
structure, or export a general-purpose serializer. The pinned `serde_json`
1.0.149 uses `zmij` for its default float spelling.

## Writer inventory

| Upstream output and source | Rust counterpart and scope |
| --- | --- |
| Sliced plate JSON, `Format/bbs_3mf.cpp:7048–7053`; fields in `GCode/ThumbnailData.cpp:8–43` | `project/export/xml.rs::plate_json`; use the formatter for bounds, area, layer height, first-layer time and nozzle diameter. |
| `Metadata/filament_sequence.json`, `Format/bbs_3mf.cpp:8664–8690` | `project/export/xml.rs::filament_sequence`; its sequences and assignments contain integers, so no float hook or output change is needed. |
| Preset/settings JSON, `Config.cpp:1564–1604`, used by the archive writer at `Format/bbs_3mf.cpp:8203,8224` | `project/settings_export.rs`; config scalars are serialized strings and vectors contain serialized elements. Keep this representation unchanged. |
| Statistics JSON, `GCode/GCodeStatistics.cpp:182–194`; CLI `result.json`, `src/OrcaSlicer.cpp:527–579` | `gcode_processor/statistics.rs::GCodeStatistics::to_json_bytes` and `ares-cli/src/result_json.rs`; separate CLI outputs, not 3MF members. Float spelling in those formats is deferred. |
| Preset bundle manifest and legacy preset migration, `PresetBundle.cpp:9286`, `Preset.cpp:2911`; app settings, `AppConfig.cpp:1181` | No matching archive writer. Profile bundle import, migration and app state are outside this slice. |
| Texture displacement settings, `TextureDisplacement.cpp:5771–5851`; intermediate slice-data dumps, `Print.cpp:6867–6891` | No matching Ares export writer. Do not add these formats. `Format/AssembleList.cpp` reads JSON but does not write it. |

The `libslic3r` paths in the table are relative to `src/libslic3r/`.

## Included number behavior

- Port binary64 boundaries, cached powers, significand multiplication and
  rounding, digit generation and the final digit adjustment from
  `deps_src/nlohmann/detail/conversions/to_chars.hpp:48–912` (`diyfp`,
  `compute_boundaries`, `get_cached_power_for_binary_exponent`, `grisu2`). Retain
  the upstream license and attribution. Use portable Rust integer arithmetic
  and `f64::to_bits`; there is no platform-specific formatting or fallback.
- Port `append_exponent`, `format_buffer` and `to_chars` (same header,
  `:920–1106`). Preserve signed zero, the `.0` on a float written as a fixed
  integer, the binary64 fixed/scientific cutovers (`min_exp = -4`,
  `max_exp = 15`), and lowercase `e` with a signed, at least two-digit exponent.
- Match `dump_float` (`deps_src/nlohmann/detail/output/serializer.hpp:768–795`):
  all non-finite values produce `null`, which the Serde JSON serializer already
  handles before its float hook. JSON integers keep unadorned decimal output
  (the separate `dump_integer` path at `:332–338,682`). Do not port the fallback
  for non-IEEE formats (`:797` onward).

The change is only numeric serialization. Do not change numeric inputs,
geometry, metadata fields, ZIP writing, CLI routing or configuration parsing.

## Public verification

Use the existing sliced STL export route (`ares slice … --slice 1 --export-3mf`)
with explicit presets and a synthetic chamfered STL. Its binary-exact mesh
coordinates produce a fractional first-layer area while preserving integral
bounds and center. Confirm that both CLIs accept the model and that the area
exercises a differing float spelling before committing the regression test.
Use the converted inch STL through the top-level `--convert-unit --slice …
--export-3mf` route once it supports sliced STL export. Neither route may patch
a JSON member after writing it.

Run the CLI case against the pinned Orca release in official and seeded modes.
Require byte equality of `Metadata/plate_<n>.json` without parsing or rewriting
its numbers. Compare the complete set of archive members with the existing
comparator and only its established generator-banner and member-order rules.
Do not compare the raw ZIP envelopes or add exceptions. Keep both original
archives.
