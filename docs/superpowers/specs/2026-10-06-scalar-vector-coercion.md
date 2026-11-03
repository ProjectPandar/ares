# Scalar→sequence coercion for vector-kind options in project_settings.config

Matrix audit (main @ `028c758d3`, `${TMP_ROOT}/ares-matrix-clusters.md`): 12 TEXT_DIFF
rows where the oracle coerces a bare JSON string scalar on a vector-kind
option and continues; ares rejects during `ProjectSettings` typed deserialize
with `invalid type: string "…", expected a sequence`.

## Upstream boundary

- `Config.cpp:927-929` (`ConfigBase::load_from_json`): for every key whose
  JSON value `is_string()`, upstream calls `set_deserialize(opt_key,
  it.value(), substitution_context)` — the target option type is never
  consulted for JSON shape first. A bare string is therefore valid input to
  every `co*` vector option.
- `Config.hpp:868-887` (`ConfigOptionVector<T>::deserialize`,
  non-append): `values.clear()`, then `std::getline(is, item, ',')` splits
  the string on `,` — a single token `"0"` produces a one-element vector.
  `"nil"` elements push `nil_value()` for nullable kinds.
- `Config.hpp:1153-1159` (`ConfigOptionStrings::deserialize`): clears, then
  `unescape_strings_cstyle` on `;`-separated tokens; `""` returns without
  appending → empty vector (ares already mirrors this via
  `normalize_empty_vector_strings`).
- `Config.cpp:1001-1005`: non-string non-array JSON values (`number`,
  `boolean`) for option keys only log `invalid json type` and are skipped —
  the option keeps its default. **Out of scope**: no matrix row exercises
  it; noted so a later slice can add the same tolerance.
- `Config.cpp:959-1006` (`is_array()` branch → `parse_str_arr` →
  `set_deserialize`): already mirrored by
  `json_deserialize.rs::encode_array` on the preset path.

Exact native behavior on the audited rows: orca accepts the scalar, then
fails later in `FullPrintConfig::validate` (`PrintConfig.cpp:10519-10559`)
with e.g. `retraction_distances_when_cut: 28 not in range [10,18]` — i.e.
the coercion never reaches the error surface; ares just must not reject at
load.

## Ares destination

`crates/ares-core/src/options/json_deserialize.rs`:

- Add `pub(crate) fn normalize_scalar_vector_strings(&mut
  BTreeMap<String, RawConfigValue>)`: for each key where
  `option_definition(key).kind` is a vector kind (`is_vector`) and the
  value is `RawConfigValue::Text`, replace with `decode(kind, &text)`
  (existing function — splits on the kind's separator, validates lexical
  elements, emits `List`). Empty strings keep the existing
  `normalize_empty_vector_strings` empty-list result (decode of `""` would
  produce a 1-element `[""]` list for most kinds, not the upstream empty
  vec — so run the empty normalization first, then scalars).
- Merge the two normalizations behind one entry point (e.g.
  `normalize_option_scalars`) called from both `ProjectSettingsRaw` call
  sites; keep `normalize_empty_vector_strings` as its first stage.
- Decode errors surface as the existing `invalid Orca option {key}: {…}`
  text wrapped in `invalid project settings JSON: …` — matches today's
  preset-path wording.

`crates/ares-core/src/project/load.rs`:

- Route the no-override path through the normalized raw: parse
  `ProjectSettingsRaw` first (already done at :115), apply
  `normalize_option_scalars`, then `ProjectSettings::deserialize` via
  `project_settings_raw.clone().into_object()` exactly like the override
  path at :128-131. Drop the direct `deserialize_json::<ProjectSettings>`
  call at :114 — the builder visitor (`project_deserialize.rs`) already
  provides unknown-key tolerance and duplicate last-wins
  (`set_deserialize_raw`, `Config.cpp:573-587`), so behavior is
  unchanged for the strict path.
- Error wrapping keeps the `invalid project settings JSON: ` prefix;
  messages lose the `at line X column Y` suffix for option-shape errors
  (upstream has no line/col in its config-error output — closer parity).
  Existing test assertions use prefix + key + reason and stay valid.

## Rows covered

- `activate_chamber_temp_control` scalar `"0"` — SPARKX i7 ×4
- `idle_temperature` scalar `"180"` — E3NG ×5
- `default_filament_colour` scalar `""` — Cubicon ×3 (already handled by
  `normalize_empty_vector_strings`; verify the unified path keeps it)

## Test plan (RED first)

`crates/ares-cli/tests/orca_parity/scalar_vector_coercion.rs`:

- Synthesize 3MF fixtures whose `project_settings.config` contains
  `"activate_chamber_temp_control":"0"`, `"idle_temperature":"180"`, and
  `"default_filament_colour":""`; assert `slice_project_plate_processed`
  reaches a later gate (not the `expected a sequence` error) and that the
  resolved option is a 1-element vector.
- E2E artifact: exported case re-run of the 12 matrix rows post-fix —
  expected outcomes: SPARKX ×4 + E3NG ×5 now fail at O3-range/compat gates
  (or pass once O3 merges); Cubicon ×3 still rejected cleanly.
  Artifact: `${TMP_ROOT}/ares-audit-errors3/results.tsv` rerun table.

## Non-goals

- Non-string JSON scalars (number/bool) for vector keys (upstream skips
  silently) — no matrix row; deferred.
- Printer-option range checks (`validate_loaded_config` extension) —
  separate spec, waits for O3 merge.
- fw_retraction+wipe compat — last cluster.
