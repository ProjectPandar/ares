# Placeholder regex whole-match INPUT pair

Source PlaceholderParser.cpp699–704 uses whole-subject regex_match then
negates for !~. Rust comparison slice ONLY expression.rs::parse_compare
RegexMatch/RegexNotMatch. See
`docs/architecture/2026-10-21-placeholder-regex-full-match.md`.

Actual official-exported INPUT3MFs copied unchanged. Sole GUI field
machine_start_gcode changes the regex pattern on ASCII subject abc from
/abc/ to /b/. Probe is in a harmless custom G-code comment. Same ordinary
8-sided six-layer cylinder/Classic1wall/no support/brim/skirt/spiral.
Manifest records the exact INPUT strings, whole/per-uncompressed-entry
SHA256/export provenance, identical inventory/sole-payload/sole-key proof.
No expected output or Native result constants/assertions/snapshots.

Two direct native_stream_case::check tests reuse main double Native
replay then native_api_cli validated API/bare CLI whole-byte comparison.
No copied replay/normalization/JSON/orchestration or private output seam.
ARES_REQUIRE_ORCA=1,explicit official/seeded Native
environments with retained artifact roots. Target:

```
cargo nextest run -p ares-cli --test orca_parity --no-fail-fast \
  --test-threads 1 -E 'test(placeholder_regex_full_match_)'
```

Require whole controlPASS/substring positiveactualFAIL in both modes,
stable repeats before production change. Only existing
canonical generated-by identity/date normalization; no comment/M73/time/
header/E/XY/config/trailing-line mask/tolerance. No ignored or expected-
failure flags. Environment-disabled tests are NOT evidence. Shared checker
retains full streams/launches/both public comparison errors.

GREEN covers ONLY whole-subject regex comparisons
through anchored compile ^(?:pattern)$ using existing crate, with existing
negation and alternation semantics. No first-unanchored-span shortcut.
Dialect/Unicode/escaping/errors/other users/rendering deferred; value.rs is
unchanged. Any residual requires STOP/report. Tests/inputs/helpers are frozen.
Extra !~/alternation/wildcard screens remain external, not new formal fixtures
or dual-mode regression credit. Discovery evidence is not substituted for
formal shared-checker public API/CLI proof.
