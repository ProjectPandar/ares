# ARD — spiral vase multi-copy by-layer admission

Non-negotiable decisions (spec: `spiral-vase-admission-spec.md`).

1. **Literal single-check port.** Exactly `Print.cpp:1334-1341`: instance
   count over 1 with a non-by-object sequence rejects with the upstream
   diagnostic on the existing `Validate` path. No extra conditions, no
   object-count heuristic, no new exit plumbing.
2. **Deferred stays deferred.** The `Print.cpp:1342-1353` regions check and
   BBL spiral emission byte parity (the guard-probe findings) are out of
   scope; a GREEN that touches spiral emission is a STOP.
3. **Native-first RED.** Runtime Native via the existing
   `early_cli_reference` (`EarlyExit::Validation`, -51) reference machinery;
   double-run determinism asserted; no Native artifacts committed; the
   witness asserts full stdout/stderr channel and `result.json` equality.
4. **Acceptance-level guards, deliberately.** The two guards assert that
   Native and Ares both accept (exit 0 plus G-code production), because
   accepted-scene spiral bytes diverge on this tip (recorded findings). This
   is an explicit, documented bound — not a weakened comparator on the
   witness.
5. **E2E-only verification.** No unit tests; the three cases above are the
   sole verification.
6. **Boundary hygiene.** Only `print/validate.rs` changes in GREEN; the file
   stays ≤400 LOC; other files are untouched.
