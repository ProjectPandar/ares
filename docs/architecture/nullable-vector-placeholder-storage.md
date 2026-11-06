# Nullable vector storage in placeholder config

## Upstream boundary

Pinned beta: `345efec64c7b95a05ce5cf96df0dde66a158835c`.
`Config.hpp:1083–1086,1090–1135` stores nullable integer nil as INT_MAX and
serializes `nil`. It is legal config storage, not a malformed numeric token.
`PlaceholderParser.cpp:1065–1075,1118–1119,1352–1353` throws only when a nil
element is referenced, using "Trying to reference an undefined (nil) element
of vector of optional values" for a vector element. Nil can remain unreferenced
through a complete slice. Referencing nil is a template error, not a panic.

## Rust destination

`project_slice/gcode_emit/value.rs::Config::from_block` currently eagerly
parses both Ints and IntsNullable through an integer unwrap. Preserve a distinct
nil value for nullable numeric/bool vector tokens at this config boundary.
Nullable floats, percents and bools share this vector-token boundary and must
also preserve nil identity rather than treating it as ordinary string text.
Do not change nonnullable parsing or introduce generic nil language behavior.

The existing expression/template reference paths must reject a selected nil
value with the cited upstream message and propagate their existing returned
error into SliceError. Inactive expressions must continue to skip evaluation.
Native vector index front fallback remains unchanged. No rendered-text patch,
catch_unwind or nil-to-zero conversion is permitted.

`PlaceholderParser.cpp:1504–1518` owns is_nil variable-reference semantics.
Ares lists is_nil as a keyword but has no implemented evaluator. That function
and its dedicated variable-reference grammar remain deferred; this slice does
not add a generic expression-level nil test.

## Public witnesses and failure modes

The unchanged mixed 3MF SHA256
`561ca738f30133e99af9b162ceb2e83586d1068edfa09b2e5ddf2d914d6ca387`
is the pre-cooling input-only fixture retained at
`tests/parity/pre-cooling-variant/mixed.3mf`. Native successfully stores mixed
nil/zero vectors and emits `0,nil` and `nil,0` for the two pre-cooling fields.
Public Ares API and CLI must successfully slice that same unreferenced input
and produce equal raw output bytes. This prerequisite does not yet claim
pre-cooling footer equality: its separate typed-member/gather port is on hold.

Derived inputs retain the same archive except authored settings for the first
nullable integer vector and machine_start_gcode. Its selected nil element is
referenced with brace and legacy syntax. Native must reject with the exact nil
message; Ares must return SliceError and CLI failure with that message rather
than panic. Keep actual Native logs, input identities, API results and CLI
stderr as repeatable artifacts. No private parser tests are added.

Failure modes: eager parsing panic even without a reference; nil coerced to
zero/string; selected nil emitted as text instead of rejected; nonnullable
parsing altered; legacy/brace inconsistency; errors swallowed in active code
or raised in skipped branches. Existing nullable gather and expression controls
remain required. Keep strict whole-output comparisons separate from these
public success/error witnesses.

## Included and deferred

Included: nullable vector config storage and existing reference error paths
only, platform-neutral in ares-core on WASM, Windows, macOS and Linux.
Deferred: is_nil implementation, generic nil language design, changes to
nonnullable input validation, pre-cooling typed members/defaults/key list/gather,
single-variant printer filament variant selection and all other variant cohorts. The latter
remain independently owned slices, not an Ares pipeline redesign.
