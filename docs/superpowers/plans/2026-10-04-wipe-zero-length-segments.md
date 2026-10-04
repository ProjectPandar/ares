# Plan: Prusa terminal wipe

[Spec](../specs/2026-10-04-wipe-zero-length-segments.md),
[ARD](../../architecture/ard-wipe-zero-length-segments.md).
Branch pi/prusa-terminal-wipe, base31996f48; isolated target, jobs2.

1. Public replay RED1/1 failure completed at `/tmp/ares-prusa-wipe-red` before
   edits. Single cube fixture retained; no scene simplification needed.
2. Ranked hypotheses: tiny positive residual length, point dedup, writer skip.
   Source inspection: Rust filters zero lengths; native lines includes them and
   native writer emits XY without E. External debugger at wipe_path.rs:102
   confirms exactly one duplicate adjacent pair after clipping. Capture:
   `/tmp/ares-prusa-wipe-points.jsonl`; script `/tmp/ares-prusa-wipe-rust.gdb`.
   Initial breakpoint91 had no compiled code; that attempt is not evidence.
   Successful capture has A→B→B and configured distance1000000, proving this is
   a real zero segment, not merely formatting of two nearby points.
3. Port pair retention plus upstream single junction dedup and denominator
   floor. Do not append a synthetic G1 or change clipping/writer behavior.
4. Full target/fresh-oracle and control replays; four gates. Retain all failures.
5. Push REVIEW-REQUEST, wait approval; latest-main rebase/full revalidation and
   ff integration. Then Creality SermoonV1 cooling. Random templates stay deferred.

## Final-code validation on31996f48

Target1/1 and newly sliced official CPU0 reference1/1 PASS. Controls68/68,
823/823, re3D3/3, extra-overhang stock2/2 and step2/2, official8/8 PASS.
Every inventory fully compared with `ordered_bytes_generator_only`, no drops or
skips. Reports `/tmp/ares-prusa-wipe-final-*/replay-summary.json`.

`/tmp/ares-prusa-wipe-final-exits.txt`: all0. Workspace7595/7595 PASS (9 existing
skips), fmt/defaultClippy/actual core+adapter WASM build PASS. Clippy retains219
pre-existing warnings. Logs share the final prefix. One production file changed;
no tests/assertions removed, no probes, suppressions or writer special case.
Await coordinator review; approval still requires latest-main revalidation.
