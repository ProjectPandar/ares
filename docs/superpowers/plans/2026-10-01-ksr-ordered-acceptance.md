# KSR ordered acceptance plan

Source/destination and included/deferred behavior are in the companion spec
and ARD-0081. Existing tolerant tests are temporary acceptance scaffolding,
not the target pipeline architecture.

1. Revalidate committed initialized capture identity and009d full-output
   evidence; preserve original official fixture. Verify: hashes unchanged.
2. Add a sensitivity assertion through actual CLI-produced complete bytes.
   Verify: changed object IDs are incorrectly accepted by old semantic gate,
   causing the new assertion to fail before changing the gate.
3. Replace CLI/browser gate with shared exact ordered comparator, retaining
   full input/reference/actual/identity first. Delete orphaned ID rewrite and
   redundant 90% structural golden; rename affected browser caller/selector.
   Verify: sensitive GREEN and native full equality.
4. Build actual WASM bindings and run Chromium project route. Verify: full
   browser output versus independent capture; retain any genuine RED.
5. Run workspace Nextest, fmt/clippy and relevant Tier-1 gates. Submit fixed
   code/docs patch to independent six-axis review; fix/re-review, commit/push.

No code repair has been selected beyond the acceptance-shell replacement.
The existing immutable009d full printer/option processes remain live and
must be polled, not restarted due to missing summary files mid-run.

Execution checkpoint: object-ID mutation RED/GREEN witnessed; native ordered
gate passes. Final workspace7459PASS/9skip, all-target Clippy and fmt pass.
Release WASM build and Windows/macOS/Linux CLI test compilation pass.
Two actual Chromium runs produce identical non-generator output, but both
remain RED against Orca at line8134 (`E-.02097` versus `E-.02098`). Complete
paired outputs are retained. Preliminary review's retention-order finding is
fixed; six-axis re-review approves the finite gate promotion, with final
immutable patch confirmation still required. Browser producer repair and
global printer/option acceptance remain unfinished; do not restore tolerances.
