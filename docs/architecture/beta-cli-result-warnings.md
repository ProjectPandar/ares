# Beta CLI result.json warnings channel

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. The beta CLI's result.json gains a structured `warnings` array and a `strict_mode` flag, and the CLI records a warning for every config value it substitutes while loading. Ares's result.json has neither field, so every whole-file result.json comparison against the beta fails on them.

## Upstream boundary
- **`record_exit_reson`** (`OrcaSlicer.cpp:535-586`) always writes `j["warnings"] = sliced_info.warnings` and `j["strict_mode"] = sliced_info.strict_mode` (`:573-574`). nlohmann sorts keys, so `strict_mode` follows `sparse_infill_density` and `warnings` comes last. An empty list dumps as `[]`; entries dump as tab-indented objects with sorted keys.
- **`cli_record_warning`** (`:505-518`) appends `details` with `"class"` set.
- **`cli_report_config_substitutions`** (`:520-534`), class `config_value_substituted`. For each `ConfigSubstitution` it prints `warning: <file>: value "<old>" for <key> is not valid, using "<new>" instead` to stderr and records `{class, file, key, old_value, new_value}`, where `new_value` is `subst.new_value->serialize()`.
  - It is called after a 3MF load (`:1992`), whose `ConfigSubstitutionContext` uses the CLI's `ForwardCompatibilitySubstitutionRule::Enable` (`:1380`, `:1737`).
  - It is also called after a `--load-settings` JSON load (`:2262-2263`).
- **`slicing_warning_non_critical`** (`:7150-7160`) records each `NON_CRITICAL` slicing status. In the beta runs on disk, every one comes from `PrintObject::is_support_necessary` (`PrintObject.cpp:5289-5303`, reported at `:975-986`): "It seems object … has a floating cantilever / floating regions …".
- **`--strict`** (`:1388-1398`, `:7156-7160`) sets `strict_mode`, refuses `--no-check`, and turns a `NON_CRITICAL` warning into `CLI_SLICING_ERROR`.

## Ares destination and scope
- **Included, the result.json keys.** `ares-cli/src/failure/report.rs::record_exit_reason` writes `strict_mode` (always `false`, since `--strict` is deferred) and `warnings`, in upstream's key order and dump format. `FamilyFailure` carries the recorded warnings.
- **Included, `config_value_substituted`.**
  - `json_deserialize::normalize_unknown_wire_substitutions` is Ares's one substitution site, reached for the 3MF project settings (`project/load.rs:127`, `raw_settings.rs:67`) and the settings normalization (`raw_settings.rs:56`).
  - It returns each substitution as key, old wire text and the new value serialized.
  - Project loading exposes the list through the smallest public read the CLI needs.
  - The CLI prints the stderr line and records the warning in upstream's order, the 3MF before the `--load-settings` files.
  - The existing comments citing an `EnableSilent` rule (`Model.cpp:259`) are corrected: at the beta the CLI passes `Enable`, so substitutions are reported.
- **Deferred, `slicing_warning_non_critical`.** It needs `PrintObject::is_support_necessary` and the support-overhang detection behind it, which Ares does not compute. That is a libslic3r port of its own. Until then, a failure after slicing an object Native flags reports `"warnings": []`. None of the current comparisons hits this case (below).
- **Deferred, `--strict`.** Its only action is on `NON_CRITICAL` warnings, and its `--no-check` refusal has no counterpart: the Ares CLI has no `--no-check`.
- **Deferred, the success result.json.** Upstream also calls `record_exit_reson` on success.
  - At the end of a run it writes `CLI_SUCCESS` ("Success.", `:194`) with `sliced_plates` (`:557-566`), plus `sliced_count` when objects were duplicated (`:8128-8136`).
  - After the beta's `--inspect_mesh` / `--inspect_paint` actions it also writes it (`:6361`, `:6385`); Ares has no such actions.
  - Ares writes result.json only on failure. Two ARDs already list the success report as deferred, without an owning slice (`cli-compatible-machine-result-metadata.md`, `ard-cli-retained-output-exit102.md`), so it is an unported channel.
  - Proposed as the follow-up slice: the success-side writer with `sliced_plates` and `sliced_count`. The success-path `config_value_substituted` entries (the 106 successful runs on disk) would be witnessed there.

## Comparisons this moves
Measured against the beta oracle over the 21 test files that call `result_comparison_bytes` (72 tests; 15 pass on this branch's base). An uncommitted probe that only added `"strict_mode": false` and `"warnings": []` was then reverted. It moved 34 tests to PASS:
- `spiral_vase_multi_copy_by_layer_rejected_like_native`;
- `cli_custom_gcode_exit` ×5 (flsun_qqs, ginger 1.2/3.0/5.0/8.0);
- `cli_exit_channels` ×6 (empty_plate, notfound, postprocess, process_compatible, validate, values);
- `compat17_branch1_reject`, `compat17_branch1_switch_reject`;
- `cli_sequential_clearance` ×3 (`cli_s4_rejects_*`);
- `cli_custom_gcode_failure_preserves_nonempty_upward_machine_header`;
- `load_settings_3mf_missing_file_post_process_order`, `load_settings_3mf_post_process_reject`;
- `post_process_cli` ×2;
- `custom_gcode_unknown_function` ×2, `custom_gcode_vector_scalar` ×2;
- `stock_early_cli` ×8 (kobra3 02/04/06/08, mk3s 025/04/06/08).

The other 23 fail at assertions that are not result.json comparisons (stderr, G-code, banner, cache reruns; panic lines recorded). No failure-path Native result.json among them records a warning. The substitution class moves none of the current comparisons. The beta runs on disk that record it are successful slices, whose result.json Ares does not write yet (the deferred success report above), or another worktree's evidence.

## RED and the retarget
1. **RED** for the key schema: the 34 tests above, which fail at their result.json comparison on the base.
2. **RED** for substitutions: a new CLI case. A project with an unknown `support_ironing_pattern` value whose plate holds no object inside the print volume must match Native's stderr warning line and its result.json `warnings` entry. It is rejected with `-50` (`:6562-6573`), after the load reports the substitution (`:1992`). A `post_process` rejection cannot serve: it comes earlier (`:1806`), before the report.
3. **The held `plate_origin_cli` retarget** lands on top. Its Native result.json has `"warnings": []`, so with the keys it compares whole.
