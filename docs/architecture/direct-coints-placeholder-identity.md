# Direct coInts placeholder identity — source-cited SPEC

Upstream pin is `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Upstream owner and exact Rust destination

OrcaSlicer `PrintConfig.cpp:2917-2923` registers
`temperature_vitrification` as `coInts` with default `ConfigOptionInts{100}`.
`Config.hpp:995-1006` defines `ConfigOptionIntsTempl` as a vector of `int`,
with the non-nullable alias at1084. Its per-element serialization at
`Config.hpp:1070-1077` writes the signed integer directly to an ostream; the
existing nil sentinel/error behavior is not changed by this slice.

`PlaceholderParser.cpp:932-945` expands a scalar/vector placeholder through
`serialize()`/`vserialize()[idx]`; therefore a raw coInts element is emitted as
an integer, not as a double. Its expression lookup at1212 uses `coInts` and
`set_i`. The cited operation is configuration-to-placeholder type preservation,
not the PlaceholderParser lexer/tokenizer, arithmetic evaluator, formatter, or
runtime writer.

Rust destination is narrowly
`crates/ares-core/src/project_slice/gcode_emit/value.rs::Config::from_block`,
`parse_value`, and `parse_scalar`. Current `from_block` consults the option
registry only for Bool/Bools; other numeric config text is parsed as f64 into
`Value::Number`, so registered non-nullable coInts lose their integer identity
before placeholder lookup. The registry already declares
`temperature_vitrification` as `Ints`.

## Included behavior

Port only registered, non-nullable `coInts` identity at config-block ingress.
The existing semicolon/comma vector parsing and indexed aliases must retain the
integer variant for each coInts element. A signed value such as `-123456789`
must remain an Integer and render through the existing Integer path.

This slice does not change the lexer, tokenizer, expression evaluation,
formatter, runtime placeholder bindings, vector selection, nil handling,
coInt scalar options, nullable Ints, floats, percentages, enums, strings,
assignment lifecycle, or any global numeric retyping. Integral doubles remain
Numbers. No name-specific formatter branch, validation, fallback, tolerance,
mask, changed expectation, or new pipeline is allowed.

## Frozen public RED witness

The existing exported 3MF inputs from the read-only discovery are copied
byte-for-byte; no regeneration is allowed:

- control: `temperature_vitrification=["110"]`
- target: `temperature_vitrification=["-123456789"]`

Both append the same existing machine-start comment:
`;DIRECT_COINTS [temperature_vitrification]`. The sole decoded configuration
delta is the temperature_vitrification value. All other ZIP member payloads
are identical. Native export accepts both; neither archive contains G-code.

The public test must use unchanged `runner::native_stream_case::check` through
`NativeCase`, dynamic Native expected streams, complete ordered generator-only
comparison, and no hardcoded expected output. It must run Native2 stability
before API and CLI. Both official and seeded modes must be executed with fresh
stage caches: control PASS and target actual strict FAIL. The target seeded
comparison must not be inferred from official; it must actually run.

## Deferred sequence

1. Commit this SPEC only, retaining the existing RED inputs unchanged.
2. Add a formal public RED case and register it without changing shared helpers,
   production code, or expected values. Run official and seeded; each must show
   Native2 stable, control PASS, target FAIL, API/CLI raw equality, and the
   strict first difference. Stop if either mode differs from this matrix.
3. GREEN must not widen the destination boundary.

All changed text must remain below400LOC. No browser or multicore runtime claim
is part of this SPEC.
