# inherits_group coStrings import/export parity boundary

Status: proposed prerequisite to the approved range-region port; no production
change authorized before this spec review. Stage2 edits are paused outside the
working tree; original range fixture and strict comparison remain unchanged.

## Source-owned conclusion

PrintConfig.cpp:4193 declares inherits_group as coStrings. Config.hpp
ConfigOptionStrings::serialize delegates to Config.cpp:72-120
escape_strings_cstyle: semicolon-separated slots; quote any slot containing
space/tab/backslash/quote/CR/LF, and quote a single empty slot; escape backslash,
quote, CR and LF inside quotes. This is configuration serialization, not a
G-code numerical difference or an optional normalization rule.

Ares registry is already correct (`registry/definitions/table/middle_independent.rs`
Strings). Existing `config_export/value/strings.rs::escape_string_vector_element`
and OrcaStrings serializer already own the source rule. The defect is the
embedded raw export path: ProjectSettingsRaw::normalize_option_scalars decodes
scalar vectors but leaves raw JSON array strings as unescaped text; raw_settings
iter/token joins them without Strings semantics; config_export/writer.rs echoes
unmodeled inherits_group from that generic token. Profile-generated bookkeeping
already uses insert_string_slots and stored export-escaped tokens. Do not change
registry kind or manually patch quoted G-code lines.

## Minimal Rust destination

Canonicalize embedded inherits_group at the existing project raw normalization
entry (`project/load.rs` → `ProjectSettingsRaw::normalize_option_scalars`), after
upstream scalar-vector decoding, using the existing coStrings slot serializer.
Both scalar-wire and JSON array input must arrive at the same canonical raw token
representation already used by profile bookkeeping. Preserve slot order/count
and multi-empty `;;`; do not re-escape tokens later in writer or on profile merges.
Check selected-profile metadata/overlay paths for double escaping. No new wrapper,
option abstraction, export exception or broad string-option refactor.

Scope is inherits_group only. Other vector families, numerical slicing,
range routing, filament usage and unrelated raw placeholder behavior are deferred.
No Stage2 region production diff in this prerequisite. External malformed option
handling stays at the existing parse/validation boundary; no internal defensive
checks or invented default slot values.

## Verification conclusions required before acceptance

Use the existing unmodified project-array.3mf (10mm cube) and runtime OrcaRunner
Native. Independent baseline Native succeeds; actual inherits_group vector
contains a profile name with spaces and two empty slots. The public
slice_project_plate_validated output must fail complete generator-only equality
before production and pass afterward, including motion/M73 and every config line.
The earlier diagnostic used an existing CLI binary and is not promoted to the
new same-build runtime RED; retain all receipts outside the repository.

Before the applicable production change, runtime Native E2E must also establish
legal coStrings escaping cases (tab/backslash/quote/CR/LF, single empty slot and
multi-empty slots) at this same public seam, without weakening comparison or
modifying the original baseline fixture. Profile-entry controls must rule out
double quoting. Test source/input only enter the repository; Native bytes,
receipts, diagnostics and process records stay runtime-cache/artifact storage.

Acceptance requires fresh exact-main base/candidate seeded plus workspace
nextest, targeted E2E, clippy -D warnings/fmt and platform checks as requested.
Bind common unchanged streams and legitimate inherits_group-only differences;
no retry-to-PASS or fixture exceptions. Coordinator review/approval before
ordinary ff-only push. Then rebase the paused region-range port and resume its
unchanged whole-output comparator; fixing this prerequisite is not Stage2 GREEN.
