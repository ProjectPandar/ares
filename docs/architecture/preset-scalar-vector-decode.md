# Decode preset string-vector scalars before composing profiles

## OrcaSlicer v2.5.0 contract

`OrcaSlicer.cpp:2205–2217` loads preset JSON. A JSON string goes straight to
`set_deserialize` (`libslic3r/Config.cpp:984–987`), where a `coStrings` option
uses `ConfigOptionStrings::deserialize` (`Config.hpp:1225–1229`) and
`unescape_strings_cstyle` (`Config.cpp:173–242`). Semicolons delimit members
outside quotes. A double quote at the start of a member opens a quoted member;
inside it `\n` and `\r` produce newline and carriage return, and any other
escaped character is taken literally (including `\"` and `\\`). Unquoted
backslashes are literal. The parser skips spaces and tabs before a member and
after its closing quote; unquoted trailing spaces remain data. Empty wire text
is an empty vector, `""` is one empty member, and a trailing semicolon adds an
empty member. An unclosed quote, dangling quoted escape, or non-separator
after a closing quote is invalid.

`OrcaSlicer.cpp:3747–3772` copies from already-deserialized filament vectors
into project slots, taking member zero for an ordinary filament option.
`libslic3r/GCode.cpp:8370–8378` writes `opt_serialize`; the vector serializer
(`Config.hpp:1207–1210`, `Config.cpp:99–148`) escapes the logical contents once.

## Ares boundary and affected inputs

| Input | Decoder at the existing boundary | Effect of this change |
| --- | --- | --- |
| Preset JSON | `options/json_deserialize.rs` routes typed `ProfileFragment` values through `model_config_deserialize::wire::decode`. The catalog's authored raw view handles empty wire text but otherwise retains scalar `coStrings` as `Text`. | Decode scalar `coStrings` in the core authored-option view after `handle_legacy` and before inheritance/slot selection, for all preset kinds. Store logical members as a `List`, using the same decoder as the typed fragment. Authored JSON arrays already supply logical members and are unchanged. |
| 3MF `Metadata/project_settings.config` | `project/load.rs` calls `normalize_wire_vector_shapes`, which uses `json_deserialize` and the same `wire::decode` for string-vector scalars and arrays. Arrays are quoted/encoded from their logical members before decoding. | No change to embedded settings. A saved project that also loads replacement presets only changes on its preset side. |
| STL `--options` JSON | `SliceOptions` accepts JSON values, then `project_slice/option_stl.rs` handles legacy keys and calls `normalize_wire_vector_shapes` before the STL config is built. | No change; it already reaches the same decoder. |
| JSON overrides | `ConfigOverrides::validate_entry` calls `decode_wire` for a single input token, including a single-item array; other array lengths supply tokens directly. `apply` stores decoded vector elements as a list. | No change to override parsing, shape or precedence. |

Region metadata has a separate equivalent C-style decoder. Typed legacy
aliases also have their own parser with an additional nested-array check.
Neither is used to decode these live authored preset keys; neither changes.
The existing wire decoder is the core `ConfigOptionStrings::deserialize`
counterpart and already implements the grammar above. The defect is that the
parallel catalog view skipped it: when a scalar was wrapped as a single slot
by the project overlay, the wire quotes became data. Keep both views typed
before composition instead of adding any special case to CLI dispatch, slot
selection, or the footer. Other vector kinds and serializer rules are out of
scope. The `profile_settings` effective config also exposes the decoded list
for machine `default_filament_profile`, which is `coStrings`
(`PrintConfig.cpp:2350–2354`). Native selection reads its vector of values
(`PresetBundle.cpp:4070–4076,8835–8836,8854–8858`), not the authored JSON shape.

## Verification

Using a synthetic saved project, slice with the same argv and compare complete
G-code against Orca (normalized generator banner only). Exercise scalar forms
of `default_filament_colour`, `filament_notes`, and `filament_ramming_parameters`,
including quoted empty and quoted singleton values. Repeat with an escaped
note containing a literal semicolon within its quotes and a second member
separated by an unquoted semicolon; the selected slot must retain the whole
first member. Qualify CLI, STL, preset/profile/config, and filament-note and
colour cases in both Native modes, plus the core profile tests, on base and tip.
