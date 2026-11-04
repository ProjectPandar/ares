# Machine-start overall chamber placeholder integer identity

Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Owning upstream constructor and exact Rust boundary

This is a libslic3r GCode placeholder-constructor rewrite slice, not an Ares
expression policy or independently designed slicing feature.

`GCode.cpp:3009-3012` initializes integer max_chamber_temp to zero and computes
its maximum over the writer's used extruders' chamber temperatures.
`GCode.cpp:3037` registers overall_chamber_temperature as
`new ConfigOptionInt(max_chamber_temp)`.
`Config.hpp:954-972` stores a signed integer and serializes it as an integer.
`PlaceholderParser.cpp:1040-1041` resolves coInt via set_i, not set_d;
`PlaceholderParser.cpp:494-509` divides two integers as integers, promoting
only when an operand is double. Integer/Number rendering remains unchanged.

Rust `placeholders.rs:285-299` ALREADY inserts Value::Integer for this alias.
`machine/start_config.rs::self_start_config` then calls that shared base_config
and overwrites the same alias with Value::number(f64::from(...)) at28-32.
The temperature helper already returns i32. That last writeback loses the
upstream integer identity at machine-start expression evaluation.

Exact future GREEN destination is ONLY that Value constructor and its widening
conversion in `project_slice/gcode_emit/machine/start_config.rs`.
Construct Value::Integer(i64::from(the existing temperature helper result)).
Keep the helper, used-extruder IDs, maximum calculation, all other bindings,
writer state and invocation order unchanged. No extra validation/fallback.
The current base-config implementation is not a second fix site.

## Included / deferred behavior

Include integer identity of the machine-start overall_chamber_temperature
placeholder; this replaces one incorrect Number compatibility writeback around
upstream ConfigOptionInt. Preserve current temperature selection and commands.

Defer other runtime placeholders, layer-change rebinding, config option typing,
nullable options, multi-tool lifecycle/temperature selection, thermal commands,
lexer/tokenizer, evaluator, formatter, motion, processor, timing and acceleration.
No name branch inside a generic formatter or evaluator. No global integer
retyping or changed numeric literal semantics.

This boundary avoids orchestration total_layer_count/auto_lift,
lexer/tokenizer/fan/zero-E, and writer jerk/format/acceleration. It does not
extend the separately frozen coInts ingress slice.

## Frozen Native-first public witness

Copy the existing Native-exported archives unchanged; do not regenerate them:

| role | chamber_temperature | SHA256 |
|---|---|---|
| control | ["4"] | `f14985d07827445123e4e42ed68582e94c622bba651f12c472ab9ec7d87cfcc7` |
| target | ["5"] | `ddb1daa8c94b29eeaf41e5f92f33768a69465da99dad9abacf926f9f1b436e31` |

The common machine_start_gcode is exactly
`; CHAMBER {overall_chamber_temperature/2}`.
Only chamber_temperature differs in decoded project_settings.config; every
other ZIP payload is byte-identical, with no archived G-code. A value4 control
has the same result for integer/double division; target5 distinguishes them.

Use unchanged native_stream_case::check -> NativeCase with (false, noop),
fixture SHA and upstream citations. No copied comparator or embedded Native
expected streams, coordinate or numeric expected-value assertions.
Native2 must be stable FIRST, followed by real validated API and actual CLI.
Official and seeded formal RED must each actually show control PASS/target
strict FAIL and actual raw API/CLI equality. Preserve the complete ordered
stream, including comments/M73/motion/time/metadata/footer; no masks/tolerances.
Canonical RED panic precedes final raw assertion and helper adapter update:
audit raw artifacts independently instead of claiming unreachable assertions.

## Sequence

1. Commit this SPEC before tests or production changes.
2. Commit frozen inputs, README/hash manifest and public two-case RED separately
   from production. Register in the existing case group, not the full root.
3. Actual official and seeded formal RED with independent initially empty
   caches. Static fmt and strict workspace/all-target clippy; verify frozen
   inputs, source boundary and all changed text<400LOC.
4. Subsequent GREEN remains restricted to the single start_config.rs writeback
   above, never formatter/eval/temperature computation.

Formal RED does not borrow discovery verdicts or claim runtime type captures. The source-only max_print_z idea was excluded
because its existing binding is already Integer. No browser/multicore claim.
