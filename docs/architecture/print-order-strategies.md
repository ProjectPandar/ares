# `PrintOrder::Snake` and `PrintOrder::BestOfStrategies`

Source-cited Rust rewrite slice of OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).

## 1. Upstream Boundary

All behavior in this slice is owned by `OrcaSlicer/src/libslic3r`:

1. `GCode/OrderingStrategies.hpp:26-148` and `GCode/OrderingStrategies.cpp:30-104,115-228,282-550`:
   - `tsp_2opt_improve` (`:30-104`): returns `false` immediately for `path.size() <= 2`; runs up to `max_passes = 10` best-gain reversals on the closed cycle.
   - `tsp_remove_crossings` (`:115-228`): returns `false` immediately for `path.size() <= 3`; removes the first geometric crossing `(i, j)` iteratively with a 500-edge uniform grid threshold and FNV-1a cycle memoization up to `int(pn * pn)`.
   - `tsp_cycle_path_length` (`OrderingStrategies.hpp:39-48`) and `tsp_max_edge_length` (`:51-61`): closed-cycle Euclidean total length and maximum edge length (`0.0` when `path.size() < 2`), casting coordinates to `double` before subtraction.
   - `compute_row_threshold` (`:287-342`): extracts unique `Y` values with `MIN_GAP_FILTER = 1.0` comparing consecutive sorted `Y` values (`sorted_ys[i] - sorted_ys[i - 1] > 1.0`), computes consecutive gaps and median/min gap, sets `threshold = (median_gap < min_gap * 1.5) ? min_gap * 0.5 : median_gap * 0.5`, detects row structure when `unique_ys.size() * 2 <= n` or `max_gap < min_gap * 2.0`, and otherwise returns `max((y_max - y_min) * 0.02, 1e4)`.
   - `group_into_rows` (`:346-368`): bins each point by `int64_t(floor(double(y) / row_threshold))`, averages `Y` per bin in input index order, and sorts rows ascending by `avg_y`.
   - `build_serpentine_path` (`:372-398`): sorts each row ascending by `X`, emits row `0` left-to-right, and for each subsequent row chooses left-to-right when `dist_to_left <= dist_to_right` (squared Euclidean distance from previous row endpoint) and right-to-left otherwise.
   - `row_serpentine_path` (`:401-423`) and `snake_core` (`:425-438`): builds the serpentine path then runs up to 3 rounds of `tsp_2opt_improve` followed by `tsp_remove_crossings` (without `or-opt`), breaking early when neither improves.
   - `chain_instances_with_core` (`OrderingStrategies.hpp:71-122`), `chain_print_object_instances_snake` (`:440-448`), and `chain_print_object_instances_best_of` (`:454-495`): instance-level chaining over `PrintInstance::shift`. When `start_near != nullptr`, `chain_instances_with_core` pre-rotates centers/instances so the closest point is index `0`, runs `snake_core`, and rotates index `0` to the front; when `start_near == nullptr`, it calls `tsp_rotate_minimize_closing`. `chain_print_object_instances_best_of` evaluates `[Default, Snake]` and chooses the shorter total cycle length, tie-breaking on smaller maximum edge length and keeping `Default` on an exact tie.
   - `order_points_with_strategy` (`:501-548`): island-level entry point. Dispatches non-`Snake`/non-`BestOfStrategies` orders to `chain_points_with_postprocessing`. For `Snake`, runs `snake_core` directly on `points` (without pre-rotation), rotates the point closest to `start_near` to the front when `start_near != nullptr` (without nearer-neighbor reversal), or minimizes the closing edge when `start_near == nullptr`. For `BestOfStrategies`, evaluates `[chain_points_with_postprocessing, run_snake]` and selects by `(tsp_cycle_path_length, tsp_max_edge_length)` keeping `Default` on tie.
2. `GCode.cpp:3680-3689`: in non-sequential (`PrintSequence::ByLayer`) export, assigns `print_object_instances_ordering` from `chain_print_object_instances`, `chain_print_object_instances_snake`, `chain_print_object_instances_best_of`, or `sort_object_instances_by_model_order`. Sequential (`PrintSequence::ByObject`) export at `GCode.cpp:3614-3616` always uses `sort_object_instances_by_model_order` regardless of `print_order`.
3. `GCode.cpp:7270-7392`: in `process_layer`, island-level ordering is active whenever `print_sequence != ByObject`, `single_object_instance_idx == size_t(-1)`, and `print_order != AsObjectList`. For each filament, quantized island centroids (or instance shifts when no chainable islands exist) are passed to `order_points_with_strategy(node_points, print.config().print_order, &wt_pos)` (`:7358`), reversed (`:7360`), and grouped into consecutive `InstanceVisit` entries.
4. `PrintConfig.hpp:318-325` and `PrintConfig.cpp:413-419,2275-2302`: enum wire tokens `"default"`, `"as_obj_list"`, `"best_of"`, and `"snake"` (already parsed and serialized in Ares).

`Print.cpp:3817` (brim owner order) unconditionally calls `chain_print_object_instances(*this)` without reading `print_order`, so brim generation order is unaffected by `Snake` or `BestOfStrategies`.

## 2. Rust Destination Boundary

| Rust boundary | Upstream owner |
| --- | --- |
| `crates/ares-core/src/geometry/point_ordering/two_opt.rs` | `OrderingStrategies.cpp:30-104::tsp_2opt_improve` (`pn <= 2` early return) |
| `crates/ares-core/src/geometry/point_ordering/crossings.rs` | `OrderingStrategies.cpp:115-228::tsp_remove_crossings` (`pn <= 3` early return before grid allocation) |
| `crates/ares-core/src/geometry/point_ordering/snake.rs` | `OrderingStrategies.cpp:282-438` (`compute_row_threshold`, `group_into_rows`, `build_serpentine_path`, `row_serpentine_path`, `snake_core`) |
| `crates/ares-core/src/geometry/point_ordering.rs` and `geometry.rs` | `OrderingStrategies.hpp:39-61,71-122` and `OrderingStrategies.cpp:440-548` (`cycle_path_length`, `max_edge_length`, `chain_instances_snake`, `chain_instances_best_of`, `order_points_with_strategy`) |
| `crates/ares-core/src/project_slice/gcode_emit/layers/object_order.rs` and `instances.rs` | `GCode.cpp:3680-3689` (`snake_order`, `best_of_order` dispatch) |
| `crates/ares-core/src/project_slice/island_print_order/visits.rs` and `gcode_emit/layers/island_visits.rs` | `GCode.cpp:7270-7361` (`order_points_with_strategy` with `print_order` and `&wt_pos`) |
| `crates/ares-core/src/project_slice/capabilities.rs` | Lift `UnsupportedProjectFeature` rejections for `print_order: best_of` and `print_order: snake` |

Existing layer/instance adapters (`gcode_emit/layers/{instances.rs,object_order.rs,island_visits.rs}`) remain temporary compatibility shells around `GCode::process_layer` and `GCode::do_export`, not an independently designed pipeline.

## 3. Determinism and Numeric Alignment

- `group_into_rows`: every `y_key = floor(y / row_threshold)` bucket spans `[k * t, (k + 1) * t)` with `t > 0`, so `avg_y` is strictly increasing with `y_key` and bin insertion preserves ascending input index order. Storing buckets in `BTreeMap<i64, Vec<usize>>` followed by `avg_y` ordering is deterministic and independent of hash randomization.
- `build_serpentine_path`: sorts row indices by `x` using `crate::geometry::fixed_gcc_sort_by`, matching libstdc++ `std::sort` tie order across all row lengths.
- `compute_row_threshold`: preserves upstream raw coordinate literals (`MIN_GAP_FILTER = 1.0`, `fraction_of_y_range = 0.02`, `min_threshold_um = 1e4`) without rescaling, matching `OrderingStrategies.cpp:293,402-403`.
- Live runtime verification against OrcaSlicer v2.5.0 confirmed identical normalized G-code between official and seeded builds across multi-instance (`tiny_3`, `small_6_grid`, `grid_4x4`, `grid_5x3_wide_rows`, `grid_3x5_tall_cols`, `clustered_14`, `staggered_18`, `subgrid_18`), multi-island (`multi_island_16`, `stepped_best_of_switch`), interleaved two-object (`interleaved_2obj_18isl`), two-filament (`two_filaments_mixed_best_of`), and `by_object_3` scenes for `default`, `snake`, and `best_of`. Notably, `subgrid_18` exercises `snake != default` with `best_of == snake`, while `small_6_grid` and `clustered_14` exercise `snake != default` with `best_of == default`.

## 4. Included and Deferred Behavior

**Included**:
- Full `Snake` and `BestOfStrategies` algorithms and dispatch at both `GCode.cpp:3683-3687` and `GCode.cpp:7272,7358`.
- `ByObject` sequential prints accepting `print_order = snake` and `print_order = best_of` while retaining upstream model-order traversal (`GCode.cpp:3614-3616,7270`).
- Cross-instance interleaved `InstanceVisit` consumption and multi-filament per-extruder strategy ordering.
- Lifting `capabilities.rs` rejections for `print_order: best_of` and `print_order: snake`.

**Deferred**:
- `GCode::m_ordering_cache` (`GCode.cpp:7350-7391`): pure memoization cache; Ares continues to recompute the pure tour per layer identically to `docs/architecture/gcode-island-visit-order.md`.
- Invalid-visit-index `BOOST_LOG_TRIVIAL(error)` log line (`GCode.cpp:7720`): unreachable for internally produced visits.
