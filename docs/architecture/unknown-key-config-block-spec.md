# Unknown-key CONFIG_BLOCK echo — source-cited spec

Status: SPEC approved (coordinator ACK 2026-10-25); this document freezes the
boundary for the tests-only RED, then a bounded GREEN.

## Witness (discovery on main `5e8be001f`, CPU18 serial)

Stock Bambu Lab P1S 0.4 ByLayer plate (`overhang.obj` + `cube_with_hole.obj`)
with exactly one added project-settings key
`first_layer_size_compensation = "0.2"` (single-key delta against the control
proven in discovery; the key has zero matches in upstream `OrcaSlicer/src`).

- Official Native exits 0 across fresh runs; generator-only replay is stable,
  including cross-run replay.
- Ares CLI output is byte-identical across runs.
- The unchanged complete comparator fails: first difference at byte 12675
  (line 192, column 9) — Ares emits `; first_layer_size_compensation = 0.2`
  (38 bytes; 463202 → 463240); no other stream difference exists besides the
  normalized generator line.
- Class proof: an arbitrary key `zz_bogus_unknown_key = "3"` is omitted by
  Native and echoed by Ares.

## Upstream boundary (pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`)

- `Config.cpp:576–590` `ConfigBase::set_deserialize_nothrow`:
  `handle_legacy` clears keys invalid in this version; cleared keys are pushed
  to `substitutions_ctxt.unrecogized_keys` and ignored — never stored.
- `Config.cpp:608–629` `set_deserialize_raw`: a key with no `ConfigDef` entry
  and no alias throws `UnknownOptionException`; it cannot enter the config.
- `Config.cpp:946–962` (`load_from_json` array branch): unrecognized keys are
  recorded and skipped.
- `GCode.cpp:5591–5646` `GCode::append_full_config`: the `; CONFIG_BLOCK`
  serializes only keys of `print.full_print_config()` (defined keys), minus
  the banned set (5613–5627) and nil options (5636–5638). Keys unknown to the
  definition table therefore never reach the CONFIG_BLOCK.

## Ares destination boundary

`crates/ares-core/src/options/config_export/writer.rs::
write_canonical_entries`, raw-echo loop starting at the
`for entry in raw_settings.iter()` statement. After the existing
`from|name|version`, `EXPLICIT_RULES`, nil-shadow, and already-emitted filters,
every remaining raw project-settings key is emitted verbatim — including keys
that neither the Ares option registry nor upstream defines. The canonical
loop already gates on `orca_block_keys::contains || raw_settings.contains_key`;
the raw loop has no definition-table gate.

## GREEN contract (single bounded edit)

In the raw-echo loop only, drop raw keys that the upstream definition table
does not store under that name. The judgment must be a literal port of
upstream's def lookup:

- a raw key is echoed iff `options::registry::option_definition(key)`
  returns a definition; keys that are only aliases of a definition are
  dropped too, because `Config.cpp:608-629` stores an alias under its
  canonical key (`opt_key = opt.first`) — the alias name itself is never
  serialized, and the canonical key echoes via its own raw entry;
- no new hand-written white/black list; the registry tables are the only key
  authority;
- the `handle_legacy` cleared-key path keeps the existing `EXPLICIT_RULES`
  behavior byte-for-byte (Config.cpp:576-590 maps onto those rules today).

Alias-value attribution observed on the official Native probe (input sha
`e345c077…`, `perimeter_feed_rate=123` appended after canonical
`inner_wall_speed=300`): Native omits the alias name and keeps
`; inner_wall_speed = 300`, i.e. the canonical typed value wins; Ares
behaves the same once the alias name is dropped. Deeper alias ordering
semantics (alias before/after canonical, multiple aliases) remain deferred.

## Included / deferred

Included: the writer.rs raw-loop definition-table gate; RED witnesses below.

Deferred: value serialization/spelling (`config_export/value.rs` is owned by
the queued W6 window and must not be touched), vector Floats verbatim-echo
semantics, legacy rename/bookkeeping rules (unchanged), upstream's
unrecognized-key reporting side channel (Ares has no substitution context),
machine-start/custom G-code content, and all geometry-generation behavior.

## RED contract (this phase)

Two `native_stream_case::check` cases, group-registered in
`tests/orca_parity/case_support.rs`:

- target `p1s-bylayer-unknown-key`: the single-key witness archive
  (`026a736275d93c7a88ce979b67afeeab088625de2f1c5e05fc08a7c840546565`);
- control `p1s-bylayer-control`: the same plate without the key
  (`2ff89639d4f6c6898393690e3236b6a1fff7edc116c3ba77c3eb8f98de5ff6ca`).

Native witness assertions for both: `; printer_model = Bambu Lab P1S`,
`; print_sequence = by layer`, `; total layer number: 50`,
`; model label id: 45,56`, and Native must not contain
`; first_layer_size_compensation` (the target archive sets it; Native drops
it). Native output is generated at runtime and never committed; a missing
Native configuration fails the test rather than passing silently.

Exit RED: control PASS and target FAIL (Ares emits the extra CONFIG_BLOCK
line) in official and seeded modes, API/CLI success with byte-equal outputs,
unchanged complete comparator, `cargo fmt` and strict Clippy clean. GREEN
later flips the target to PASS with the control and all frozen parity tests
unchanged.

## Operating constraints

CPU18 `taskset`, all threads 1, no FULL, no rebase/merge/push; branch
`w1/unknown-key-config-block` off main `5e8be001f`; docs commit then tests-only
RED commit; one REVIEW-REQUEST then HOLD.
