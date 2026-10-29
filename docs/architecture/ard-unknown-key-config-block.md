# ARD — unknown-key CONFIG_BLOCK echo gate

Non-negotiable decisions for the unknown-key CONFIG_BLOCK milestone
(spec: `unknown-key-config-block-spec.md`).

1. **Definition-table literal port.** The known-key judgment is exclusively
   the existing Ares option-registry tables (`option_definition` +
   `OptionMetadata::aliases`), ported from upstream `Config.cpp:608–629`.
   Hand-written white/black key lists in `writer.rs` are forbidden.
2. **Single-file production boundary.** The GREEN edit touches only the
   `options/config_export/writer.rs` raw-echo loop. `config_export/value.rs`
   is owned by the queued W6 window and must remain untouched.
3. **Legacy rules frozen.** The `handle_legacy` cleared-key behavior
   (`Config.cpp:576–590`) keeps the existing `EXPLICIT_RULES` semantics
   byte-for-byte; no rule is added, removed, or reordered in this milestone.
4. **Native-first RED.** Witnesses use `native_stream_case::check` with
   group registration; Native output is generated at runtime, never
   committed, and a missing Native configuration fails the test — no silent
   pass. The complete comparator is never weakened.
5. **E2E-only verification.** Per repo policy, verification is the two public
   parity witnesses (target single-key and control); no isolated unit tests
   are added.
