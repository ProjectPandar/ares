# BBL M205 Z/E tail precision inputs

Source: `GCodeWriter.cpp:298-299`, defaultfloat with precision two.
Specification: `docs/architecture/2026-10-24-bbl-jerk-tail-precision-spec.md`.

The existing GUI-generated BBL one-object project has its tower disabled and
`default_jerk=9` in all cases. The target changes only the saved machine Z jerk
to 0.45; the controls retain stock Z=3 or use Z=2.5. E remains 2.5. No geometry,
template, XY, temperature, PA or other process change is made. These are machine
motion settings, not a claim about GUI decimal display or recommended tuning.

| Input | SHA256 |
|---|---|
| stock-control.3mf | 8218407a4f814505bf6df4f4c82b0c67a39c73acfaf938da72ba5144f50fe32f |
| fractional-z-exact.3mf | 5eb57eb85313786b8b3f19cf8591771c2d79aeda963e571c619a20ba2fb18a52 |
| positive-z-control.3mf | 70db0f1afd38380c28f93b2d435b01120d49cb74a46cf3925c10ed6ddcf03afe |

Three direct calls use the existing `native_stream_case` seam. After fresh Native
replay qualification, callbacks inspect body M205 suffix words against the real
saved input values. All admitted values have at most two significant digits,
so their ordinary input-derived text is an independent expectation. This checks
leading-zero preservation without calling the Ares formatter. It is not a test
of every rounding or exponent corner. No hardcoded Native coordinates, output
counts, reference values or Native output files are committed.

Then complete Native/API/CLI output is compared with the unchanged generator-only
comparator, including M73, headers, configuration and footers. Observations are
not filters. The clean target changes only M205 text; M73 remains byte-equal.
The Z=0.4567 discovery archive and its M73 movement are explicitly excluded.

Failure modes were documented before implementation in the SPEC: fixed versus
significant precision, missing zero, incorrect rounding/exponent selection,
regressions in existing six-digit callers, duplicate algorithms, and unrelated
header/cache/XY or time-estimator edits. GREEN must parameterize the existing
single ostream implementation, not duplicate it or patch only the prefix.

Expected RED: target fails, both controls pass in official and seeded modes;
Native replay, callbacks and raw API/CLI equality qualify. Tests and inputs freeze
before GREEN. Group registration preserves normal separators and root registries.
