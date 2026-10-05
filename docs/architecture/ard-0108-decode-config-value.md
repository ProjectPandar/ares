# ARD-0108: decodeConfigValue — serialize()-form to typed JSON

## Boundary

Upstream: `OrcaSlicer/src/libslic3r/Config.cpp`
`ConfigOption*::deserialize` element parsing (string `unescape_*_cstyle`
rules, `,`-/`;`-joined vectors, `nil` elements on nullable vectors,
`ConfigOptionBool` reading only the first comma element, `ConfigOptionFloats`
empty input → `[0]`), plus `Config.hpp` `ConfigOptionType` → our
`OptionValueKind`.

Rust destination:

- `ares_core::decode_config_value(kind, serialized) -> Result<String, String>`
  — emits JSON text directly. Value shapes are fixed per kind, so no dynamic
  `serde_json::Value` intermediate is used in production code (keeps the
  `no_unapproved_dynamic_values` baseline clean).
- `ares_core::decode_config_value_by_key(key, serialized)` — resolves `key`
  through `option_definition` (unknown keys error).
- wasm binding `decodeConfigValue(key, serialized) -> String` (JSON text;
  throws on unknown key or malformed value).

## Included

- All 21 `OptionValueKind` variants exported by `printConfigDef`.
- `bool` accepts exactly `1`/`0` (after trim; scalar reads only the first
  comma element) — `ConfigOptionBool::deserialize` / `ConfigOptionBoolsTempl::
  deserialize_with_substitutions` parity. `true`/`false`/empty error, same as
  upstream. The exported metadata `default_value` bool encoding
  ("true"/"false") therefore does not round-trip through this decoder; that
  encoding predates this slice and is tracked as a separate follow-up item
  rather than tolerated here.
- Non-nullable vectors reject `nil` with an error, matching upstream
  `ConfigurationError` on deserializing nil into a non-nullable object.
- Numeric extraction mirrors `std::istringstream >> double/int` prefix
  semantics (`"0.4;0.6"` → `0.4`, `"0,0"` scalar → `0`): upstream parses the
  longest valid numeric prefix and ignores the rest. `Points` elements split
  on `x` with a missing coordinate decoding to `0` (upstream `Vec2d::Zero`
  default).
- Elements with no numeric prefix at all (`"V"`, `""`) error instead of
  emulating upstream's uninitialized-garbage `>>` success — stricter than
  upstream only on inputs upstream would corrupt anyway.

## Repeatable verification

`cargo run -p ares-core --release --example decode_config_value >
target/decode-config-value-report.json` decodes every registry-known
key/value in `tests/**/*.3mf` `project_settings.config` and
`OrcaSlicer/resources/profiles/**/*.json` (≈12027 files / 427731 entries,
≈342203 decoded, ≈85523 unknown keys). Each failure records whether upstream
`deserialize` would also fail; entries upstream accepts only by storing
garbage are classified `upstream_lenient_accepts_garbage`.
- Nullable vector elements `nil` → JSON `null`.
- `Floats` empty input → `[0]` (upstream `ConfigOptionFloats` behavior).
- `String`/`Strings` use the C-style unescape rules already ported in
  `model_config_deserialize::wire` (`unescape_strings_cstyle` parity);
  `wire` was promoted to `pub(crate)` instead of duplicating the tokenizer.

## Value shapes

| kind | JSON |
| ---- | ---- |
| bool | `true`/`false` |
| int, float, percent | number |
| float_or_percent | `{ "value": N, "percent": B }` |
| enum, string | string |
| point | `[x, y]` |
| points | `[[x, y], …]` |
| points_groups | `[[[x, y], …], …]` |
| vectors | array of element shape; `nil` → `null` |

`percent` decodes to the bare number (`"50%"` → `50`), while
`float_or_percent` keeps the unit flag — matching how `Percent` and
`FloatOrPercent` differ upstream.

## Deferred

- Encode direction (`serialize()` from JSON) — not requested; 3MF/preset
  writing stays in the existing serialize paths.
- `deserialize_with_substitutions` placeholder substitution semantics —
  decode is pure lexical; the CLI parity path handles substitutions where
  upstream does.
- Empty elements inside `Ints`/`Floats` vectors error instead of
  `strtol`/`strtod` yielding `0`: serialize() output never contains empty
  elements, and erroring on malformed input is stricter than silently
  coercing.
