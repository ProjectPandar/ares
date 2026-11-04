# C2-T PLAN correction: validate canonical legacy pattern tokens

Status: proposed PLAN after STOP `320fb0757`; source inspection only.
No Native, cargo or nextest runs. Existing tests/production are untouched.

## Source comparison

Orca source commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

| Boundary | Evidence | Result |
| --- | --- | --- |
| Global top domain | PrintConfig.cpp:2074-2086 | Eight values: monotonic, monotonicline, rectilinear, alignedrectilinear, concentric, hilbertcurve, archimedeanchords, octagramspiral |
| Global bottom domain | PrintConfig.cpp:2097-2103 | Copies top enum_values |
| Global internal-solid domain | PrintConfig.cpp:2106-2112 | Copies top enum_values, no zig-zag/zigzag entry |
| Membership check | Config.hpp:2498-2503 | Exact enum_values comparison |
| Four global checks | PrintConfig.cpp:10401-10419 | sparse/top/bottom/solid checks are unconditional; no under_cli, density or role condition |
| JSON scalar load | Config.cpp:927-929 | Calls set_deserialize |
| Before deserialize | Config.cpp:573-579 | handle_legacy modifies key/value before option decode |
| Exact legacy rewrite | PrintConfig.cpp:8201-8207 | For sparse/top/bottom/solid/ironing/support-ironing, zig-zag becomes rectilinear |
| Real ZigZag enum | PrintConfig.cpp:232,3024 | zigzag is distinct; global sparse admits it, global surface subsets do not |

Therefore Ares' existing sparse and surface predicate domains are correct.
Native's Anycubic result is explained by loading normalization, not a larger
internal-solid enum domain or a conditional validate check. It validates
serialized rectilinear after rewriting the input zig-zag.

## Ares path and existing contracts

`options/validation/helpers.rs:10-19,55-60` reads the retained string and
checks it directly. `project/raw_settings.rs::handle_legacy` and
`SliceOptions` deserialization call `options/legacy/token.rs::handle_legacy`,
but that string-token port omits PrintConfig.cpp:8201-8207.
`options/tests/legacy_pattern_migrations.rs` explicitly asserts that
SliceOptions retains zig-zag for all four patterns; these assertions stay.
The typed legacy rule inventory independently already records the exact
zig-zag to rectilinear transform (`typed_legacy/actions/rules.rs:13-16,
113-118`). `ProcessInfillPattern` currently aliases zig-zag to ZigZag
(`process_options/object_source/enums.rs:167-171`), a separate typed/geometry
normalization discrepancy. This slice will not change that enum or stored
configuration behavior under cover of a validation repair.

The STOP matrix has three Anycubic presets with the extra invalid value
zig-zag message on both modes. Raw-string validation sees a pre-normalized
compatibility-shell representation where upstream sees a canonical enum.

## Proposed correction

Rust destination: `options/validation/helpers.rs::validate_pattern_option`.
This helper is used only for the four global infill-pattern checks.
Before membership validation, view the exact retained zig-zag token as
rectilinear, citing PrintConfig.cpp:8201-8207 and Config.cpp:577-579.
Keep both predicate domains unchanged. No new whitelist entries, ignored
errors, case folding, trimming, numeric policy or fallback. zigzag stays
zigzag; Zig-Zag and zig-zag-grid stay invalid. Other tokens retain their
existing error text. The exact source rewrite yields a valid canonical token
for this alias, so it produces no invalid-value diagnostic.

The helper remains a temporary compatibility shell over upstream's
post-handle_legacy serialized enum validation. It does not normalize the
stored raw map, local metadata, Fill selection, config echo or typed enum.
Unifying those storage/typed paths is deferred and requires its own source
slice and public RED; it cannot delete/change the existing preservation
assertions in this repair.

The concrete proposed diff is retained externally (unapplied).
Only the helper changes, in addition to the accepted top guard
removal and loaded global validation call. No new tests after production;
the unchanged public O3 matrix is the existing RED for this normalization
boundary. Keep both existing C2-T public RED commits unchanged.

## Gates

Official and seeded runs with
fresh cache/artifact roots;
rerun finite7, option_coverage220, bridge/anchor11 and actual
runner::profile_entry loaded-config17. Strict/fmt/wasm must pass.
Any other existing result change triggers STOP again, no assertion edits.
Then full parity before integration.
