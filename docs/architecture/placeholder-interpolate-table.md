# PlaceholderParser `interpolate_table` evaluation

Rewrite slice of `InterpolateTableContext::evaluate` (`PlaceholderParser.cpp:1971-2009`, OrcaSlicer v2.5.0-beta `345efec64`): out-of-range values abort the custom G-code instead of being clamped, unsorted tables are rejected, and interpolation uses upstream's `lerp`.

## Problem
`mk4-layer-change/mk4base.3mf` with `enable_prime_tower` = 0 and `layer_change_gcode` = `;AFTER_LAYER_CHANGE\n;[layer_z]\nM201 X{interpolate_table(extruded_weight_total, (0,4000), (1400,2500), (10000,2500))} Y{…}`:
- At the first layer change `extruded_weight_total` is -0.00238604. The project has `single_extruder_multi_material` = 1, so `Extruder::used_filament` leaves out the retracted length (`Extruder.cpp:171-180`), and `layer_change_gcode` (`GCode.cpp:6718`) runs after the first retract. Ares computes the same value.
- **Native** rejects the template and exports nothing (CLI `return_code` -100):
  ```
  Failed to generate G-code for invalid custom G-code.

  layer_change_gcode Parsing error at line 3: Interpolation value is outside the table range
  M201 X{interpolate_table(extruded_weight_total, (0,4000), (1400,2500), (10000,2500))} Y{…}
                           ^

  Please check the custom G-code or use the default custom G-code.
  ```
- **Ares** clamps to the first y (`expression/functions.rs:133-148`) and exports G-code.

## Upstream boundary
`InterpolateTableContext::evaluate` (`PlaceholderParser.cpp:1971-2009`), for value `x` and table points `(x_i, y_i)`:
- **Sortedness.** For every adjacent pair, `x_{i-1} > x_i` throws "X coordinates of the table must be increasing" (`:1980-1984`).
  - The check runs over the whole table, even after a segment has matched.
  - The error range is `table[i-1].it_range_x.begin()` to `table[i].it_range_x.end()`, so the caret is at the start of the earlier point's x expression.
- **Segment.** The first segment with `x0 <= x <= x1` sets the result (`:1985-1996`):
  - `y0` when `x == x0`;
  - `y1` when `x == x1`;
  - `0.5 * (y0 + y1)` when `is_approx(x0, x1)` (`libslic3r.h:320-323`, `EPSILON` = 1e-4, `:57`);
  - otherwise `lerp(y0, y1, (x - x0) / (x1 - x0))` = `(1 - t) * y0 + t * y1` (`libslic3r.h:313-317`).
- **Outside.** When no segment matches (`:1999-2008`):
  - `x0 - EPSILON < x < x0` gives the first y;
  - `x1 < x < x1 + EPSILON` gives the last y;
  - otherwise `expr_x.throw_exception("Interpolation value is outside the table range")`.

  A one-point table only takes this branch.
- **Error report.** `throw_exception(msg, it_range)` (`:1888-1893`) raises an `expectation_failure` with tag `*msg`. `process_error_message` (`:1895-1940`) formats it as:
  - `Parsing error at line N: msg`;
  - the source line;
  - spaces up to the error column, then `^`.

  For the outside case the column is the start of the x expression (`expr_x.it_range`). The exception propagates out of `placeholder_parser_process` (`GCode.cpp:5319`), and the export ends with `PlaceholderParserError` (`GCode.cpp:3637-3645`).

## Ares destination and error shape
- **`expression/functions.rs` `interpolate_table`** ports `:1971-2009` literally: sortedness over all pairs, first matching segment, the `x == x0` / `x == x1` / `is_approx` / `lerp` cases, and the `EPSILON` window. It returns a typed error instead of a `String`:
  ```rust
  enum InterpolateTableError {
      OutsideRange,
      NotIncreasing { point: usize },
  }
  ```
- **`expression/parser.rs` function call.** The call records the byte offset of each argument's first token, plus each point's x token (the token after its `(`).
  - It maps `OutsideRange` to `scalar_failure = (x-argument offset, "Interpolation value is outside the table range")`.
  - It maps `NotIncreasing { point }` to `(offset of point - 1's x, "X coordinates of the table must be increasing")`.
  - `scalar_failure` already feeds `ParseFailure` (`expression.rs:68`) and `template/error.rs::report`, the existing port of `process_error_message`. The new failures therefore get the upstream line and caret with no new formatting code.
  - A suppressed (inactive) branch keeps no-op'ing, as for the other functions (`parser.rs:225-231`).
- **API error shape: unchanged.** The failure goes through the existing per-template path:
  - `state.record_template_failure` collects it;
  - `template_failure::from_failed_templates` builds `SliceError::CustomGCode { message, plate_index, layer_height, wall_loops, sparse_infill_density }`;
  - the CLI exits 156 with the message on stderr and the `result.json` metadata;
  - the WASM adapter rejects with the same `SliceError` text (`ares-wasm/src/lib.rs` `slice_error_js`).

  No new `SliceError` variant or field is added. `message` is byte-equal to Native's stderr.

## Included and deferred
- **Included:** `PlaceholderParser.cpp:1971-2009` (evaluate), with the reports through `:1888-1893` / `:1895-1940`, and `libslic3r.h:313-323`.
- **Deferred:**
  - the type checks of `InterpolateTableContext::init` / `add_pair` (`:1955-1969`, "Interpolation value must be a number." and the x/y messages). Ares keeps its current messages for non-numeric arguments; that is not witnessed here.
  - the `Extruder::used_filament` share-extruder value itself, which Ares already matches.
- **Not affected:** the custom G-code motion-state cases. They use a literal `M201` in `layer_change_gcode`, and the value is in range in `before_layer_change_gcode`.

## Witness and RED
An admission case in `smoke/custom_gcode_unknown_function.rs`'s harness style, as a new `#[path]` sibling `smoke/custom_gcode_interpolate_table.rs` (`orca_parity.rs` stays byte-identical).

- **Fixture:** a new `tests/parity/validation/cli-exit/custom-gcode/mk4-interpolate-table-range.3mf`, built from `mk4base.3mf` with the two edits above. It is checked in, like the other rejection fixtures.
  - Fixture note: its `inherits_group` is padded to four empty entries (process, two filaments, printer). With the three entries `mk4base.3mf` carries, Native prints `Warning: inherits_group has 3 entries, expected 4 for 2 filaments` before the rejection header (`OrcaSlicer.cpp:1852-1855`). That CLI warning is not ported and is outside this slice.
- **Assertions, as in that harness:**
  - the Native reference is captured twice and is byte-stable;
  - Native stderr starts with the custom-G-code rejection header and contains "Interpolation value is outside the table range";
  - the Ares CLI exits 156;
  - stdout and stderr are byte-equal to Native's;
  - `result.json` is comparable;
  - no G-code is retained.
- **RED:** Ares exits 0 and exports G-code.
- **Control:** in the same file, `mk4base.3mf` with `enable_prime_tower` = 0 and the stock template in `before_layer_change_gcode`, where every value is in range (0 at the first layer, then growing). Its ordered `M201` lines must equal Native. It passes now, and covers the `lerp` form on real output.
- **GREEN gate:** a paired FULL in the exact environment of the integration batch it is compared with, with no PASS→FAIL. Any case whose output changes through the `lerp` form or the new rejections is listed.
