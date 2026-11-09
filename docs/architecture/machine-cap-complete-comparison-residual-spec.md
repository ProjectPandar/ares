# Normal-mode writer machine-limit rounding

## Upstream contract

Upstream pin: `345efec64c7b95a05ce5cf96df0dde66a158835c`.
This is a bounded Rust rewrite of `GCodeWriter::apply_print_config`, not a new
Ares pipeline. `GCodeWriter.cpp:128-149` defines `rounded` using `std::round`,
then `LIMITS_UINT = to_uint(rounded(LIMITS(...)))`, and sets the extruding
acceleration cache. Klipper additionally clamps it to positive X/Y limits.
`GCodeWriter.cpp:168-169` sets X/Y jerk with `rounded(LIMITS(...))`.
`std::round` is nearest, ties away from zero, not ties-to-even.

The normal-mode stride extraction and existing flavor gates remain intact.
Existing Rust option helpers and field assembly implement the writer-cache
compatibility boundary. Update these conversions in place without another
cache, writer, public API or fallback.

## Rust implementation scope

All paths below are under
`crates/ares-core/src/project_slice/gcode_emit/`.

| File and owner | Current expression | Replacement |
| --- | --- | --- |
| `motion/options/helpers.rs:21-22`, `machine_acceleration_limit`, extruding cap | `first_float(...machine_max_acceleration_extruding).round_ties_even()` | Same expression with `.round()` |
| `motion/options/helpers.rs:29`, Klipper X/Y conversion | `first_float(axis).round_ties_even()` | `first_float(axis).round()` |
| `motion/options/build/fields/assemble.rs:239`, `max_jerk_x` | `first_float(...machine_max_jerk_x).round_ties_even()` | Same expression with `.round()` |
| `motion/options/build/fields/assemble.rs:244`, `max_jerk_y` | `first_float(...machine_max_jerk_y).round_ties_even()` | Same expression with `.round()` |

Change only these four conversions in these two files. One Klipper map
expression applies to both axes. Retain unsigned conversion, zero/inactive
behavior, flavor gating, first normal slot, axis minimums, existing callers
and cache representations. Float `.round()` is platform neutral across browser
WASM, Windows, macOS and Linux.

Do not change generic `rounded_acceleration`, travel acceleration, Z/E jerk,
raw machine-envelope/header values, vector cardinality/configuration,
formatters, command deduplication, progress/time, geometry or fixture inputs.

## Existing failing case

`even_half_machine_cap_rounds_away_from_zero` derives requested acceleration
and cap from its unchanged input using the current source `round` contract.
In both official and seeded modes and API/CLI paths, complete comparison
fails at byte1333/L60C9, Native `M204 S401` versus Ares `M204 S400`, with both
streams107774bytes. These values describe the failure; they are not new
assertion goldens. Both stock and odd-half controls pass. Their API/CLI bytes
must remain exactly unchanged after the conversion update.

Fresh Native replay precedes the Native check and complete comparisons.
The existing case supplies the failing case and pre-change baseline.

## Failure modes and verification

Establish the pre-change baseline for the affected jerk/acceleration cases.
Distinguish:

- Wrong tie semantics versus raw header/cache confusion.
- Applying the conversion to unsupported flavors or inactive/zero limits.
- Dropping Klipper's positive-axis clamp or changing axis ordering.
- Accidentally rounding Z/E jerk, travel acceleration or the generic helper.
- Native premise drift in an adjacent historical test versus a production
  complete-output failure. Do not alter or waive such a test silently.
- A pre-existing failure versus a new failure or changed stream.
- A passing selected target versus unchanged byte behavior on other cases.

Use existing E2Es through `native_stream_case`/`NativeCase`. The regression
matrix includes the three machine-acceleration-cache cases and all existing
jerk-named cases: machine-jerk rounding, Klipper XY clamp/separate precision,
merged precision/tolerance, deduplication, Repetier and BBL flavor/precision
controls. Keep the same logical test inventory before and after. Do not add
unit tests or numerical goldens.

Run official and seeded modes with the same upstream pin. Native replay and
complete Native/API and Native/CLI comparison remain mandatory. Retain raw
API/CLI bytes and every M73/header/configuration/footer byte.

For jerk impact, report per-case before/after pass/fail, first differences
and actual API/CLI byte changes. Passing-before cases must not regress. Each
changed case must correspond to the four conversions and match Native.
Existing failures remain explicit; a passing target does not imply success
for the whole suite. If a stale Native premise blocks an adjacent case,
report that gap without extending the implementation scope.

This matrix does not add a separate Klipper half-axis witness. State any
coverage limitation without claiming unobserved values.

## Acceptance

Acceptance requires the existing even-half complete comparisons in both
modes to pass, with stock/odd-half control API/CLI bytes identical to their
pre-change baseline. The jerk matrix must have no unexplained pass-to-fail
or byte change. Preserve failures and do not weaken complete comparisons.

The implementation must pass `cargo fmt --check` and
`cargo clippy --workspace --all-targets -- -D warnings`.
