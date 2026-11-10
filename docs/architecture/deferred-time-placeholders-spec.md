# Deferred print time placeholders

## Source boundary

This slice rewrites OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`. All paths are
relative to `src/libslic3r/`.

- `GCode::_do_export` (`GCode.cpp:4086-4091`) calls `PlaceholderParser::set_deferred` for
  `print_time_total_sec`, `print_time_day`, `print_time_hour`, `print_time_minute`,
  `print_time_sec` and `used_filament_length`. Each key holds its reserved tag
  (`GCode/GCodeProcessor.cpp:120-125`, `@PRINT_TIME_TOTAL_SEC@` through
  `@USED_FILAMENT_LENGTH@`). The parser is shared, so every template rendered after this
  point can print them: the file start (`GCode.cpp:4205-4206`), the machine start and end,
  layer change, tool change and other custom G-code.
- `GCodeProcessor` post-processing (`GCode/GCodeProcessor.cpp:1368-1424`,
  `process_inline_placeholders`, called for every exported line at `:1841`) replaces the
  tags. The values come from the normal mode time estimate, clamped at 0 (`:1385-1387`):
  - `print_time_total_sec` is the total in seconds, with two decimals (`:1401`);
  - the day, hour, minute and second split the truncated whole seconds (`:1389-1394`).
    `print_time_sec` is the seconds left after the minutes, not the total (`:1394`, `:1405`);
  - `used_filament_length` is the summed filament length in metres, with two decimals
    (`:1397-1400`, `:1406-1407`).
- `float_to_string_decimal_point(value, 2)` (`LocalesUtils.cpp:108`) is fixed notation with
  two decimals.

## Rust destination

- `crates/ares-core/src/project_slice/gcode_emit/placeholders.rs` (`base_config`): every
  Ares G-code template starts from this config, so the six keys are inserted here with the
  upstream tags.
- `crates/ares-core/src/project_slice/gcode_emit/processor/stats.rs`: the tags and their
  replacement, `:1385-1407`.
- `crates/ares-core/src/project_slice/gcode_emit/processor.rs`: passes the normal time and
  the filament total to the replacement for every line.
- `crates/ares-core/src/project_slice/gcode_emit/file_start.rs`: drops its local inserts.

This replaces a scaffold kept from an earlier baseline. It held two private sentinels, only
in the file start template, and its `print_time_sec` was the two-decimal total.

## Included

- The six deferred time and length placeholders, in every template, with the upstream tags.
- Their replacement, including the remainder meaning of `print_time_sec`.

## Deferred

- The statistics placeholders of `GCode.cpp:4094-4101`: `used_filament`, `print_time`,
  `normal_print_time`, `silent_print_time`, `total_weight`, `total_cost`,
  `total_wipe_tower_cost` and `total_wipe_tower_filament` (`:1408-1420`). They read the
  weight, cost and volume statistics, which another slice owns. Templates that use them
  still fail with "Not a variable name".
- The parser side of `set_deferred` (`PlaceholderParser.cpp:246-351`, `:488-512`, `:620-632`,
  `:759`, `:790`, `:815`, `:850`, `:862-895`, `:944-1034`, `:1499`, `:1877-1885`, `:2695`).
  Upstream marks a value that holds a deferred tag. If a template then uses that value in
  arithmetic, a comparison, a regular expression or a filament change, upstream fails with
  "This value is known only after the G-code is generated. It can be printed, but not used in
  a calculation or a comparison." Ares does not have this check yet. It evaluates such an
  expression on the tag string instead:
  - `+` concatenates the tag text with the other operand, even when that operand is a
    number, where upstream fails;
  - comparisons compare the tag text;
  - `-`, `*`, `/` and `%` fail with "numeric expression expected" instead of the upstream
    message.
  The next slice ports this with a Native error-path test.

## Premises

1. `compatible_regions::same_xy::thin_square_travel_preserves_complete_native_output` and
   `thin_square_scarf_travel_preserves_complete_native_output` fail in both modes. Ares stops
   before slicing with "invalid project file-start G-code template: Parsing error at line 2:
   Not a variable name". The stock Creality Ender-3 V3 SE profile's file start template
   prints `;TIME:{print_time_total_sec}`.
2. A full run of the parity tests at the base shows no other test with this error.
3. Adding `print_time_total_sec` as a temporary experiment, since reverted, makes both pass.
4. Native prints the six values in the file start, machine start and machine end templates
   of the new test, with nonzero minutes (`m=6 s=37 total=397.57` in one run).

## Observable behavior

Templates can print the six placeholders anywhere. The values are the same as Native's, and
`print_time_sec` is the seconds remainder.

## Failure modes

- Placeholders only in the file start template: other templates keep failing.
- `print_time_sec` kept as the total: the value differs whenever the print takes a minute or
  more.
- Rounding the whole seconds instead of truncating them: the split differs in the upper half
  of a second.
- Private sentinels kept instead of the upstream tags: the rendered text differs before
  replacement, and the parser-side check of the next slice could not match it.

## Test plan

| Test | Before the port | After the port |
|---|---|---|
| `runner::compatible_regions::same_xy::thin_square_travel_preserves_complete_native_output` | F | P |
| `runner::compatible_regions::same_xy::thin_square_scarf_travel_preserves_complete_native_output` | F | P |
| `runner::processor_time::placeholder::placeholder_deferred_print_time::print_time_sec_is_the_seconds_remainder_in_every_template` | F | P |

The new test uses the stock Creality Ender-3 V3 SE profile and the cube fixture. It prints
the six placeholders in the file start, machine start and machine end templates, and compares
the whole output with Native. At run time it also checks that Native's minutes are nonzero, so
the remainder differs from the total.

The template, G-code emission and statistics tests must show no PASS turning into FAIL in
either mode.
