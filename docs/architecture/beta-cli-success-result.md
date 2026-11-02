# Beta CLI success result.json

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64`. The CLI writes result.json on success as well as on failure; Ares writes it only on failure. This slice ports the success report for the 3MF slice the Ares CLI runs, one plate.

## Upstream boundary
- **The report.** At the end of a run (`OrcaSlicer.cpp:8126-8136`), `record_exit_reson(outfile_dir, 0, plate_to_slice, cli_errors[0], sliced_info)` writes `CLI_SUCCESS`, "Success." (`:194`). With `duplicate_count > 0` it adds the key value `sliced_count`. Like every report, it is Linux-only (`:537`), in `outfile_dir`, with sorted keys and a tab-indented dump.
- **The fields it adds beyond a failure's:**
  - `plate_index` is `plate_to_slice`: the `--slice` argument, 0 for all plates.
  - `layer_height`, `wall_loops` and `sparse_infill_density` come from `m_print_config` at the plate's setup (`:6926-6931`), the same values the failure families after that point already carry.
  - `sliced_plates` holds one entry per sliced plate, pushed after its export (`:7272`, `sliced_plate_info_t` at `:245-251`):
    - `id`, the plate number (`:6529`);
    - `sliced_time` and `sliced_time_with_cache`, wall-clock seconds (`:7259-7260`);
    - `triangle_count`, the facet count of every model-part volume of each instance whose print-volume state is inside, skipped instances excluded (`:6576-6640`, stored `:6915`, set `:7179`);
    - `warning_message`, the text of the plate's last slicing warning (`:7148`), otherwise empty.
  - `upward_compatible_machine` / `downward_compatible_machine` when present, and `warnings` / `strict_mode` as on failure.
- **Witnesses (beta, `--slice 1`).** Each exits 0 with empty stdout and stderr.
  - `tests/parity/orca_cli_ender3.3mf`: `triangle_count` 12, `layer_height` 0.20000000298023224, `wall_loops` 2, `sparse_infill_density` 15.0, `warnings` [].
  - `byobject-perobject-multi-copy-brim/combined-brim.3mf`: `triangle_count` 36, three cube instances of two objects, so the count is per instance.

## Ares destination
- **One writer.** `record_exit_reason` stops being a `FamilyFailure` method and becomes a result.json value that both paths fill: return code, error string, plate index, the three settings, upward machines, sliced plates and warnings.
  - The writer files are split first, in a move-only commit, if `failure.rs` (392 lines) would pass 400.
- **The warnings element** becomes an exhaustive enum, `CliWarning`, with one variant today: `ConfigValueSubstituted { file, substitution }`, dumped as `{class, file, key, new_value, old_value}`.
  - The deferred non-critical slicing warning becomes a second variant, `SlicingWarningNonCritical { plate_id, text }`, dumped as `{class, plate_id, text}`. The serializer's `match` then fails to compile until that variant is written.
  - This slice adds no variant without a producer.
- **Success data from the core.** The CLI's 3MF slice returns, beside the G-code, the plate's `triangle_count` and the three settings values.
  - `triangle_count` is computed over the instances the existing print-volume gate counts inside (`project_slice/state/print_volume.rs`), summing each model-part volume's facets.
  - The settings values use the same resolved-settings derivation as the failure families (`print/validate.rs:35-38`).
- **The CLI on success, 3MF input** writes result.json next to the output:
  - "Success.", return code 0, `plate_index` = the selected plate;
  - one `sliced_plates` entry `{id, sliced_time: 0, sliced_time_with_cache: 0, triangle_count, warning_message: ""}`;
  - upward machines from the existing report, `"strict_mode": false`, and the load's substitution warnings.
- **Test helper.** `result_comparison_bytes` also zeroes every `sliced_time` and `sliced_time_with_cache`, which are wall-clock like `prepare_time` and `export_time`.

## Included and deferred
- **Included:** the success report for the Ares CLI's one-plate 3MF slice, on both the plain and the `--load-settings` path.
- **Deferred:**
  - `sliced_count` (the Ares CLI has no duplicate/repetition option, so `duplicate_count` is 0 and upstream takes the `else` branch);
  - `--slice 0` (all plates; the Ares CLI slices one);
  - `warning_message` and the `slicing_warning_non_critical` entries (`PrintObject::is_support_necessary`, the support-necessity slice that lands on top of this one);
  - the `--inspect_mesh` / `--inspect_paint` reports (`:6361`, `:6385`), since Ares has no such actions;
  - the STL `--options` adapter path, which has no Native invocation to compare with;
  - `downward_compatible_machine` (no Ares downward check).
- **Nonzero timing values** are wall clock and stay normalized.

## RED
1. A new success-channel test, `cli_success_result`. For the Ender-3 cube and `combined-brim.3mf`, it runs Native `--slice 1` and `ares slice --plate 1` and compares exit, stdout, stderr and the complete result.json through `result_comparison_bytes`. Both fail today: Ares writes no result.json.
2. The helper's sliced-time normalization gets its own unit case, in the existing `native_wall_clock.rs` tests.

## GREEN
- **Core.**
  - `ares_core::SlicedPlate { triangle_count, layer_height, wall_loops, sparse_infill_density }` is filled through a new `sliced_plate: &mut SlicedPlate` field on `CliProjectSliceRequest` and `LoadSettingsSliceRequest`, following `CliUpwardMachine.report`. `LoadSettingsSliceRequest` loses its unused `Clone, Copy` derive.
  - `prepare_post_closing`, where both the plain and the profile path first hold the selected project and resolved settings, writes it.
  - The triangle count is `inside_triangle_count` (`state/print_volume.rs`): the same `calc_print_volume_state` test as the -50 gate, summing the model-part volumes' facets for each inside instance. The settings are read as the validate families read them.
- **CLI.**
  - `result_json.rs` holds the one writer, `ResultJson`, filled by the failure families (`FamilyFailure::record_exit_reason`) and by `Success`.
  - `CliWarning` is the exhaustive warnings element, with its one producer-backed variant.
  - `sliced_plates` is written only when the report has a plate, as upstream adds the key only on the first push.
  - After writing the G-code, a 3MF success prints the substitution lines and, on Linux, writes "Success.", the selected plate, one plate entry and the upward machines (none on the `--load-settings` path, which has no upward report).
- **Test helper.** `result_comparison_bytes` zeroes every `sliced_time` and `sliced_time_with_cache`.
- **Result** (beta oracle):
  - `cli_success_result_matches_native_for_one_cube` and `cli_success_result_counts_triangles_per_instance_like_native` pass, as does the helper case.
  - The warnings slice's set is unchanged.
