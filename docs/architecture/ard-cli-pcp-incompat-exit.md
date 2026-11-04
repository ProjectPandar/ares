# ARD: 3MF `print_compatible_printers` CLI rejection is a load-time read gate

Decision for `docs/superpowers/specs/2026-10-10-cli-pcp-incompat-exit.md`.
Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Decisions (non-negotiable)

1. **Fourth branch only.** The ported adjudication is exactly
   `OrcaSlicer.cpp:2608-2620` (plain CLI slice: empty list ⇒ compatible,
   otherwise membership of the printer system name). Branches 1–3
   (`:2562-2606`) are out of scope until a CLI affordance can reach them;
   they are not partially emulated.
2. **Read-only at load.** The gate lives in the plain-3MF load path and
   reads already-parsed embedded values (`printer_settings_id`,
   `print_settings_id`, `inherits_group`, `print_compatible_printers`).
   No serialization/write changes; 3MF `print_compatible_printers`
   serialization fidelity is a separate slice.
3. **Typed error, family exit.** Rejection is a dedicated
   `SliceError::ProcessNotCompatible` mapped in `ares-cli/src/failure.rs`
   to the `-17` family (stdout line, empty stderr, sorted-key
   `result.json`, POSIX exit `239`), mirroring the already-ported
   `flush_and_exit`/`record_exit_reson` channels — not a generic
   `InvalidInput` string.
4. **System-name semantics.** Compatibility compares against the
   inherit-resolved system name (`inherits_group` slot, falling back to
   the preset's own name), never the display name, matching
   `:1737-1753`.
5. **Single E2E RED, fixture pinned.** One RED
   (`cli_exit_channels.rs` case + `EarlyExit::ProcessNotCompatible`) with
   the admission input as a byte-pinned fixture; no auxiliary unit-test
   RED, no synthetic re-derivation of the fixture.

## Rejected alternatives

- Emulating all four branches now: unreachable inputs, speculative code.
- Reusing `profiles/compatible.rs` catalog filtering here: that is the
  GUI-side `Preset.cpp:801-840` `is_compatible_with` port for preset
  selection; the CLI load gate is `OrcaSlicer.cpp:2562-2654` and has
  different inputs (embedded values, system names).
