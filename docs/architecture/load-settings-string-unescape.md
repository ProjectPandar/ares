# Decode scalar string values from preset JSON

## Upstream contract

OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) loads flattened CLI preset files via `src/OrcaSlicer.cpp:2217`. After JSON parsing, the string branch in `src/libslic3r/Config.cpp:985–987` calls `set_deserialize`. It dispatches to `opt->deserialize` at `Config.cpp:686`. For `coString` (including `file_start_gcode`, `PrintConfig.cpp:6985`), `ConfigOptionString::deserialize` (`Config.hpp:1177–1181`) calls `unescape_string_cstyle` (`Config.cpp:150–170`). It maps backslash-`n` and backslash-`r` to LF and CR, copies any other escaped character without the backslash, and rejects a dangling backslash. Actual newlines from the outer JSON parser pass through unchanged.

## Rust boundary and scope

Keep the existing preset-only text decode in `crates/ares-core/src/options/json_deserialize.rs::decode_preset_values`. Admit `OptionValueKind::String` to the same dispatcher as other preset wire values. `model_config_deserialize/wire.rs::decode_scalar_string` already implements the upstream transform; no new decoder, G-code template logic, or CLI transport is needed. Leave `coStrings`, already-typed config application, and project/model-settings read and export paths unchanged. In particular, apply the transform to preset wire text, not to the typed string again at template emission.

## Verification

Generate the stock Ender-3 V3 SE 0.4 mm preset selection and grounded cube at runtime. Compare the public byte/API and CLI outputs against the complete live Native stream (guarded generator line only). Use a second direct STL case that changes only `file_start_gcode` to static comments with escaped line breaks and an escaped backslash; it does not depend on expansion of a placeholder. Use a Native-saved stock 3MF with the same preset selection to guard the existing project entry. Keep Native artifacts out of Git. Run paired load-settings, preset and config regressions with both the official and seed-0 Native environments; require zero pass-to-fail transitions.
