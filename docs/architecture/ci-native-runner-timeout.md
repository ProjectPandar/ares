# CI runner timeout investigation and repair gate

## Scope

This is Native oracle **adapter
infrastructure**, not a new slicer feature or a libslic3r rewrite milestone.
Do not invent an upstream timeout implementation: the owning source is
Ares's existing Native command lifecycle, listed below. Native slicing
behavior and the upstream CLI contract remain unchanged.

## Source boundary

- `crates/ares-cli/tests/orca_parity/runner/stages.rs:159-176`,
  `run_with_timeout`: delegates to `command::run` with the supplied timeout.
- `runner/command.rs:18-88`, `run`: creates command evidence and live
  capture files, then synchronously calls `Command::spawn`.
- `runner/command.rs:89-104`: constructs the deadline **after spawn**;
  polls direct-child status, sleeps 10ms, then calls `shutdown`.
- `runner/command.rs:135-146`, `shutdown`: kills only the direct child,
  polls for reap with an existing 2s shutdown grace.
- `runner/command.rs:106-132,149-163`: writes lifecycle evidence, reads
  regular-file snapshots bounded by their observed lengths, and finishes.
- `runner/stages.rs:207-266`, `finish`: persists immutable output and status.
- `runner/lifecycle_tests.rs:21-35,51-55`: elapsed starts **before** the
  adapter call; timeout remains 500ms and return must remain below 2s while
  the descendant still runs. `:92-123` is the bounded real subprocess
  fixture: descendant holds inherited handles for 3s.

Rust destination stays in `runner/command.rs` and its narrowly scoped
command/lifecycle test boundary; split any file that exceeds 400 lines.
No core/WASM process management is introduced.

## Observed failure, not yet proven root cause

A self-hosted CI workspace run reported:
`elapsed=4.888604235s`, `Timeout`, `kill=Ok(())`,
`reap=Ok(Some(ExitStatus(unix_wait_status(9))))`.
The capture contains both pre-timeout binary sentinels and no descendant
late-output marker. A later official parity leg passed the lifecycle test.

The current implementation has **no pipe-reader thread and no reader
join**: stdout/stderr are regular files inherited by descendants. Snapshot
reads use `take(metadata.len())`, so inherited-handle EOF is not a wait
condition. The assertion's diagnostic mentions EOF but is not phase evidence.
Existing logs do not locate the 4.89s inside spawn, polling/reap, or file I/O.
A high-load scheduling explanation alone is not a demonstrated root cause.

## Ranked falsifiable hypotheses

1. Preparation/spawn takes most elapsed time. Prediction: pre-spawn and
   spawn intervals dominate; post-spawn timeout/reap remains near 500ms.
   This exposes an existing mismatch between externally measured elapsed
   and the deadline starting after spawn.
2. Parent scheduling or reap takes most time. Prediction: spawn is short,
   but deadline observation or reap overshoot dominates.
3. Snapshot/persistence is slow. Prediction: direct child is killed/reaped
   promptly, but capture/final evidence completion accounts for the delay.

## Bounded diagnostic-first RED phase

Before any fix, use the existing real-process lifecycle command as the
red-capable symptom loop (same 500ms and <2s assertion), capturing retained
artifacts and original failures. Any stress reproduction must be bounded,
not added to canonical full/workspace invocations.

If existing evidence cannot distinguish hypotheses, add phase durations to
structured retained command/lifecycle evidence in the RED change: adapter entry, prepared, spawn returned, timeout detected, kill
returned, reap completed, snapshots completed, finish completed. Preserve
failure precedence and byte capture; no debug printing or debug probes.
Do not report a proven root cause until a failing run locates the phase.

For the public Native official+seeded E2E RED, note that Native success-output parity cannot itself expose delayed
spawn/return of the bounded helper. Proposed coverage is the existing
public oracle adapter invoking real bounded subprocesses for the timing
regression, plus real Native official+seeded adapter parity for unchanged
normal execution and capture. No new post-hoc unit test or weakened check.
Native-dependent legs wait for RESTORED where their oracle dependencies
are missing. A replacement reproduction must be test-first, not inserted
after GREEN.

## Repair decision gate

Choose only the repair demonstrated by phase evidence. A deadline moved
before spawn may prevent an extra 500ms of runtime after a slow launch,
but it cannot bound synchronous spawn itself or guarantee scheduling;
**that change alone must not be presented as a fix for this failure**.
Similarly, killing descendants is not a substitute for bounded direct-child
return and would invalidate the existing descendant-still-running contract.
If a repair requires platform-specific spawn/capture changes, revise this
spec before implementing them.

## Included / deferred and non-negotiable decisions

Included: precise timeout root-cause attribution, minimal repair at the
Native adapter boundary, unchanged binary capture/evidence behavior,
unchanged error categories, 500ms timeout and <2s assertion.
Deferred: slicer logic, CLI exit semantics, process-tree termination,
changing the 2s shutdown grace, unrelated tests/refactors, unbounded stress.
No UnsupportedProjectFeature guard, silent skip, ignore/allow attribute,
assertion weakening, Native binary/artifact commit, or core filesystem I/O.

Verification: reproducing RED before GREEN; lifecycle module and adapter
E2E official+seeded; cargo fmt, strict clippy, wasm32; fresh-target
workspace. Full requires restored oracle stores, canonical script unchanged.
Report residual timing/platform limitations explicitly.
