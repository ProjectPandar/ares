# coInts comma-list regression addendum

Bounded correction: docs first, then public RED before GREEN. Builds on the
coInts integer identity port and overall-chamber integer writeback; keep their
other behavior unchanged.

## Source-owned failure

Config.hpp995-1006 defines ConfigOptionInts as vector<int>.
Config.hpp1023-1031 serializes vector elements separated by commas;
1045-1068 deserializes comma-delimited elements before streaming each int.
PlaceholderParser.cpp932-945 expands through scalar/vector serialization and
1212 preserves coInts element identity. Upstream pin8500fcdccaa10b5099ac20d252af3a7c560046f1.

Ares value.rs::Config::from_block→parse_value→parse_scalar now preserves
registered non-nullable Ints as Integer, but parse_value first parses each whole
semicolon segment BEFORE splitting commas. A legitimate comma-list therefore
reaches raw.parse::<i64>().unwrap() and panics before API generation.
The internally generated config block is trusted, so do not add validation,
fallback, default values, error swallowing or a non-Integer escape path.

## Exact minimum destination and invariants

Only value.rs::parse_value splitting order may change: for registered
non-nullable coInts, split semicolon/comma-delimited text into leaves BEFORE
calling the existing parse_scalar integer conversion. Preserve list/scalar shape
and existing Config::insert aliases. Registered Ints integer identity remains
mandatory. Non-Ints parsing, formatter, lexer/tokenizer, evaluator, other option
kinds, runtime bindings, temperature selection and writer/processor stay frozen.

Existing parse_value is an incomplete temporary config-text adapter for the
upstream vector operation, not a new Ares deserialization policy. No new helper
engine, name branch, legacy fallback, feature flag or new pipeline.

## Frozen public regression

The exact legal Native-exported discovery archive is frozen byte-for-byte:
seam-variant.3mf SHA3e392f7a34ea2b869722aa768ff729b51d1bf0c99d9f1be708671e618f06eebf.
It has BBL X1C machine, one object with two normal parts, Classic walls, aligned
seam, and valid multi-element registered Ints options. Its historical discovery
label is seam variant; do not claim requested rear was its effective enum.
The input is not regenerated or simplified to evade a later residual.

Use unchanged native_stream_case::check→NativeCase with dynamic Native streams,
frozen SHA and upstream citations. Native2 must be stable first in official and
seeded modes; then the existing API panic at value.rs integer parsing is the
actual RED, not expected-rejection success. Do not catch the panic inside the
public test or change helper/comparator semantics. Because API panic precedes
CLI launch, no RED CLI or raw-equality assertion execution is claimed.

## Sequence / strict stop

1. Commit this addendum without production edits.
2. Freeze the original input plus one public case; run actual official+seeded
   RED with fresh stage caches; no inferred mode result.
3. Commit minimal Ints-only leaf splitting correction separately. Run the same
   full ordered generator comparisons in both modes. Retain M73/time/motion/
   comments/metadata/footer: no masks/tolerances/expected changes.
4. If the fixed variant has ANY different strict residual, STOP immediately,
   preserve it and report; do not repair its seam/travel/other behavior here.
5. Static fmt/clippy alltargets/portable compile on final verified candidate.

All changed text<400LOC, no tests after production. Previous frozen sources
remain untouched except the private value.rs splitter. Discovery scripts'
unrelated preparation failures are not evidence against Native or substitutes
for RED.
