# Configuration floating extraction: committed-exponent failure

Status: proposal only. No runtime evidence or implementation is claimed.

## Owning upstream boundary

This is a Rust rewrite slice of pinned OrcaSlicer
`345efec64c7b95a05ce5cf96df0dde66a158835c`, not a new numeric policy or pipeline.
`src/libslic3r/Config.hpp` delegates numeric extraction to fresh
`std::istringstream >> double`:

- `ConfigOptionFloat::deserialize:844-858` returns false on `iss.fail()`.
- `ConfigOptionPercent::deserialize:1262-1273` does the same; a trailing percent
  sign is optional and does not turn failed extraction into success.
- `ConfigOptionFloatOrPercent::deserialize:1381-1393` sets value/percent only
  after successful extraction.
- `ConfigOptionFloatsTempl::deserialize:932-958` stores the extracted value
  without testing failbit; `ConfigOptionPercentsTempl` inherits that loop.
- `ConfigOptionPoints::deserialize:1619-1642` streams each coordinate
  independently into a zero-initialized point.

The delegated stream contract matters: after a mantissa and `e`/`E` are
accumulated, optional sign without exponent digits is extraction failure,
not permission to fall back to the earlier mantissa. C++ `[facet.num.get.virtuals]`
Stage2 accumulates the numeric input field; Stage3 stores zero and sets
failbit when conversion does not consume the entire accumulated field:
https://eel.is/c++draft/facet.num.get.virtuals#3.3 . This source interpretation
must still be witnessed through the actual pinned Native public CLI.
Overflow has different rules and is explicitly outside this slice.

`Config.cpp:614-618,626-710::set_deserialize[_raw]` forwards deserialize failure;
numeric options are not eligible for enum/bool substitution fallback.
`Config.cpp:982-1054` feeds project JSON strings/arrays through that boundary;
`Config.cpp:1564-1570` exports actual serialized scalar/vector values.
`PrintConfig.cpp:5385-5404` generates `machine_max_speed_x` as coFloats with
minimum0, providing a public vector witness without inventing an option.

## Exact Rust destination and current mismatch

`crates/ares-core/src/options/decode.rs::cxx_float_prefix:30-66` is the existing
compatibility shell around delegated C++ floating extraction. It consumes a
mantissa, scans `e`/`E` and optional sign, but advances the accepted endpoint
only if exponent digits exist. Otherwise it parses the old mantissa and
returns `extracted=true`. Thus source inspection predicts `1e+` becomes1
rather than failed extraction. This is a prediction, not a recorded RED.

The proposed production change is confined to that exponent branch and its
in-function contract comment: preserve failure as `(zero, not extracted)`
instead of resuming at the earlier mantissa. Keep the existing mantissa,
complete-exponent, whitespace, sign, suffix and numeric conversion paths.
Do not introduce a parser framework, dependency, FFI, feature flag or fallback.
The remaining lexer stays a temporary compatibility shell; this does not
certify complete `num_get` grammar, locale or floating conversion parity.

Public callers are `decode_config_value` / `decode_config_value_by_key`:
scalar Float/Percent/FloatOrPercent use extraction success; ordinary floating
vectors/percent vectors and Points coordinates use its numeric value.
`model_config_deserialize/wire.rs` preserves these numeric tokens while
splitting vectors; it is not a new parsing owner and should remain unchanged.
Core logic stays platform-neutral Rust for browser WASM, Windows, macOS and
Linux, with no file/process/terminal/UI behavior or new API.

## Included and preserved behavior

- A mantissa followed by committed `e`/`E`, optional sign, but no exponent
  digit fails; include end-of-input and a non-digit suffix after the sign.
- Scalars report failed extraction; ordinary floating/percent vector elements
  and independent Points coordinates retain the source failure value0.
- Complete positive/negative exponents and ordinary trailing junk after a
  complete numeric field keep the existing upstream prefix behavior.
- Plain finite values, delimiters, percent flags on successful scalar values,
  existing nullable-vector interception, serialization and ordering stay put.

This is not strict whole-token validation: rejecting all trailing junk would
contradict the cited upstream stream use.

## Adjacent source mismatch requiring explicit review disposition

`ConfigOptionFloatsOrPercentsTempl::deserialize:1457-1479` also ignores failbit
and stores `{value, percent-presence}`. The Rust FloatsOrPercents public branch
instead recursively invokes scalar FloatOrPercent decoding and propagates
its failure. That pre-existing wrapper mismatch is not silently repaired by
the proposed primitive change. On malformed exponents it may move from a
wrong mantissa value to an error while Native still stores zero. No parity
claim is made for that wrapper; nullable FloatOrPercent representation is
also outside this proposal.

Recommended scope is the extraction primitive only, with that wrapper
explicitly deferred to a separate source-cited slice. Before implementation,
review must accept this boundary or request an expanded ARD. Do not add a
kind/key exception, suppress failbit, or silently fix vector/nullable schema
to make a selected test green. Valid FloatsOrPercents controls must not regress.

## Proposed Native-first public proof after approval

1. Before production code, add one bounded public E2E in the existing config
   round-trip test subtree, without touching the top-level parity ROOT.
   Use the actual pinned Native `--export-settings` and existing project
   mutation/producer adapters, then public decode-by-key plus encode-by-key.
2. First candidate: `machine_max_speed_x` with `1e+`. Native should export the
   source vector failure value while current public Rust decoding should
   retain the mantissa. Capture actual Native before asserting a real RED;
   if the witness is rejected or normalized earlier, stop and review rather
   than substitute a private parser/kernel, canned diagnostic or mock.
3. Request a bounded set of malformed forms (`1e`, `1e+`, `1E-`, `1e%`) and
   complete/plain controls (`1e2`, `1e-2tail`, `10`, `0.5`). Obtain expected
   exported strings only from real Native runs, not a copied algorithm.
4. Record a real scalar admission failure, plus affected ordinary-vector,
   percent-vector and independent-coordinate public controls before claiming
   those caller boundaries. Record exact executable/input/source/command/
   exit/stdout/stderr/export data and public results. No missing-Native pass.
5. After approved RED, port only the source-derived exponent failure clause;
   repeat strict public comparisons, compare control statuses to their
   recorded baseline, and retain verifiable artifacts. Do not claim every
   caller or platform if its witness was not executed.
6. Run fmt and workspace/all-target/all-feature clippy with warnings denied;
   keep changed files below400 lines, preserve original tests/comparators/
   missing-Native policy, and submit GREEN review before publishing.

## Deferred behavior and stop conditions

Overflow/underflow, signed zero, hex floats, NaN/inf, grouping/locale, integer
prefix extraction, new nullable kinds, FloatOrPercent-vector failure dispatch,
Point/PointsGroups redesign, JSON shape/legacy normalization, typed numeric
serde consolidation, loaded-range predicates/metadata/EPS, diagnostic rendering,
geometry/G-code/timing, browser UI, CI/cache/provisioning and new pipeline work
are deferred. If qualification needs one of these prerequisites, stop and
route it before code rather than broadening the primitive implicitly.

Failure inventory before tests: Native option/witness not reached; config
normalization changes the token; scalar/vector failure policy confused;
complete exponent or valid suffix regresses; percent/nil handling changes;
unwitnessed shared caller assumed green; zero selection/missing Native; lost
producer/input identity. No new tests, production edit, Native/Rust build,
qualification runtime or push is authorized by this document itself.
