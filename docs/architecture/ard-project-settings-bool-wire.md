# ARD — project_settings.config boolean wire forms (loader parity)

Status: SPEC (docs-only). No production code changes in this slice.
Authorization: coordinator, 2026-10-26, following the
`staggered_inner_seams` seam-scan pinpoint.

## Non-negotiable decision

The 3MF `Metadata/project_settings.config` loader must classify boolean wire
forms exactly like upstream OrcaSlicer 2.4.2. In particular, a JSON boolean
scalar on a bool-kind option is **not a value**: upstream drops the entry and
keeps the option default. Ares today accepts it as the boolean value.

## Upstream boundary (source-cited)

1. `Format/bbs_3mf.cpp` `_BBS_3MF_Importer::_extract_project_config_from_archive`
   reads the archive entry and hands it to
   `DynamicPrintConfig::load_from_json(dest_file, …)` (`Config.cpp`).
2. `Config.cpp:DynamicPrintConfig::load_from_json` (function spanning
   `:905-990`; the JSON-string branch is `:927-929` per the anchors already
   used in `project/raw_settings.rs`):
   - `it.value().is_string()` → `set_deserialize(opt_key, it.value(), …)`;
   - `it.value().is_array()` → element join, then `set_deserialize`;
   - **every other scalar JSON type (boolean, number, null) → the
     `else { BOOST_LOG_TRIVIAL(error) … "invalid json type for " << it.key(); break; }`
     branch: the key is never set**, so the option keeps its default. The
     load continues (no error is raised to the caller).
3. `Config.hpp:1828-1848` `ConfigOptionBool::deserialize`: trim the first
   comma item; exact `"1"` → true, exact `"0"` → false, anything else fails.
4. On failure, the EnableSilent 3MF load rule (`Model.cpp:259`,
   `Config.cpp:654-676`) retries with `ConfigHelpers::enum_looks_like_true_value`
   (`Config.hpp:96-99`): case-insensitive `enabled`/`on` → true; every other
   string → false. (Ares already ports this faithfully in
   `options/json_deserialize.rs:normalize_unknown_wire_substitutions` +
   `looks_like_true`.)

Consequence for the witness option `staggered_inner_seams`
(`PrintConfig`, default `false`, consumed at
`GCode/SeamPlacer.cpp:1601-1620`):

- `"staggered_inner_seams": "1"` → true (staggered inner seams);
- `"0"` / `""` / `"true"` (string) → false;
- `"on"` / `"enabled"` (string) → true (EnableSilent substitution);
- `"staggered_inner_seams": true` (JSON boolean) → key dropped → default
  **false**.

## Ares boundary (current state)

- `crates/ares-core/src/project/load.rs:119-160`: `ProjectSettingsRaw`
  (serde_json) → `normalize_embedded_wire_substitutions` →
  `normalize_wire_vector_shapes` → `handle_legacy` → typed builder.
- `options/json_deserialize.rs:normalize_unknown_wire_substitutions`: the
  bool substitution only matches `RawConfigValue::Text` — a JSON boolean is
  `RawConfigValue::Flag` and skips it (text forms behave exactly like
  upstream, including `enabled`/`on`).
- `options/config_types/scalar_bool_int.rs` `OrcaBool::Deserialize`:
  `visit_bool` accepts a JSON boolean as `OrcaBool(value)` — the deviation;
  `visit_str` is already upstream-faithful (`"0"`/`"1"` only).
- `project/raw_settings.rs:token()`: `Flag` renders `"1"`/`"0"`, so the
  flat config view also treats the JSON boolean as a real value.

## Itemized wire-form comparison (bool-kind options)

| `project_settings.config` wire | Upstream 2.4.2 | Ares (main 5948c9347) | Verdict |
|---|---|---|---|
| `"1"` | true | true | match |
| `"0"` | false | false | match |
| `""` | EnableSilent substitution → false | substitution → `"0"` | match |
| `"on"` / `"enabled"` (any case) | substitution → true | substitution → `"1"` | match |
| other strings (e.g. `"true"`) | substitution → false | substitution → `"0"` | match |
| JSON `true` | key dropped → option default | `Flag(true)` → `OrcaBool(true)` | **DIVERGES** |
| JSON `false` | key dropped → option default | `Flag(false)` → `OrcaBool(false)` | diverges whenever the default is `true` (latent) |
| JSON number (e.g. `1`) | key dropped (same else-branch) | `Number` wire — deferred, same upstream class | deferred |

## Included / deferred

Included: the boolean wire-form classification for
`Metadata/project_settings.config` bool-kind options — SPEC + tests-only RED.
Deferred: JSON number scalars (same upstream else-branch family; no witness),
preset documents (`print_setting_*`/`machine_settings_*` embedded presets use
`load_from_json` too — same mechanism, follow-up if a stock fixture ever
carries non-string wires), CLI override grammar, vector-bool options, and any
GREEN implementation (separate authorization).

## RED witness

Fixture `tests/parity/project-settings-bool-wire/k1-staggered-json-true.3mf`
(SHA256 `803a4bf91e7259068f7c1704c34454dde301281708a5d04b92fe741869a4eb86`):
the Native-runtime-exported K1 stock bridge project (seam scan
`--export-3mf`, arrange enabled) with `staggered_inner_seams` rewired from
stock `"1"` to an unquoted JSON `true` — the exact encoding the seam-scan
patcher produced. Native drops the key and slices UNSTAGGERED (first inner
wall starts `X130.857`, `SeamPlacer.cpp:1601-1620` skipped); Ares resolves
true and staggers (`X131.357`). Strict comparator must FAIL until the loader
classifies JSON booleans as upstream does. Note the encoding is
Orca-writer-never-produced (upstream `save_to_json` emits strings), so the
fixture is a wire-form conformance witness, not a Native-authored artifact.

Controls already on record (same project, official mode, native double
replay stable): `"1"` stock and `""`/`"0"` probes are full-output PASS — the
divergence is isolated to the JSON-boolean wire form.

## Exit criteria for this assignment

docs SPEC + tests-only RED (dual mode, CPU13, threads1), registered in the
parity suite group file; REVIEW/HOLD. GREEN (loader classification change)
and FULL runs require separate authorization.
