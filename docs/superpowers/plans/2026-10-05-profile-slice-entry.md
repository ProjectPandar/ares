# Profile entry implementation plan

Source boundaries, interface and failure inventory:
[spec](../specs/2026-10-05-profile-slice-entry.md),
[ARD](../../architecture/ard-profile-slice-entry.md).

1. **Native RED before production.** Retain original system JSON plus a named STL;
   independently flatten for native CLI `--load-settings`/`--load-filaments`, slice
   twice, prove native complete-stream repeatability, and record the existing
   public STL entry's complete-stream failure. Native command/setup failure is not
   a valid feature RED. Then exercise the new public entry against the same oracle.
2. **CLI FFF composition to existing lifecycle.** Add the stateless typed catalog
   and selection ingress in core; reuse profile fragments/composition around the
   CLI's `update_full_config` and filament-copy rules, not GUI `full_fff_config`.
   Keep static default membership separate from explicitly loaded dynamic keys;
   verify omitted versus explicit values and compatibility scalar/group placement. Establish the
   exact libslic3r Model import/default-instance and config normalization behavior
   from source/native traces before connecting it. Keep modules below 400 lines;
   do not create a new pipeline or regenerate intermediate 3MF archives.
3. **3MF selected-profile path.** First add a native override E2E and embedded-path
   regression. Preserve model and scoped settings; explicitly verify selected
   profile precedence and plate selection against native CLI. Existing no-profile
   exports remain unchanged.
4. **Compatibility.** First add public filtering fixtures for the spec's failure
   inventory; port Preset.cpp boolean semantics with diagnostics for its explicit
   error=true behavior. Reuse the upstream-placeholder expression implementation
   without duplicating it in TS. Include a real BBL catalog E2E timing receipt.
5. **Import and adapter.** First add JSON/archive and filename E2E cases, then port
   byte-only import rules and expose thin WASM bindings. Update all loadScene web
   callers in the same candidate; do not add web-side composition or caching.
6. **Acceptance and handoff.** Run all four gates and full controls, record exact
   source revision and artifacts, commit ARD with the implementation, push checked
   candidate and notify coordinator for review; stop until approval.

Steps 2–5 each use tests-first vertical slices, not a speculative implementation
of the entire PresetBundle. Adjacent upstream behavior outside the spec stays
deferred; current Ares structures are compatibility shells around cited concepts.
