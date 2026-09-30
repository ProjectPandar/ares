# Locked Zag width activation plan

Source-cited boundary and deferred behavior: ARD-0044 and matching spec.

1. Add separate CLI E2E tests through the existing injected-project boundary.
   Verify live RED while the default pattern remains crosshatch.
2. Add only the two options' activating pattern/density dependency.
   Verify live GREEN, exported values, actual-width sparse extrusion and
   strict complete-output equality.
3. Run the selected option sweep and retain its expected INCOMPLETE
   completion result; no range-completion claims.
4. Run workspace Nextest, fmt/Clippy and Tier 1 compilation, then request
   independent six-axis review; fix/review before code-plus-doc commit/push.

## Evidence in progress

Isolated checkout: `/tmp/ares-locked-width-isolated.hmo5aI`, base cf4cc39e.
Preliminary direct CLI probes at `/tmp/ares-locked-width-probe.r4raHE`
matched complete outputs for 0.6 mm under both keys and showed changed
extrusion between skin and skeleton cases. These exploratory probes are
not the regression acceptance evidence.

The live regression RED uses fresh root
`/tmp/ares-locked-width-red.uOiqem` and log
`/tmp/ares-locked-width-red.log`. Reference:
`scripts/orca-parity.sh`, `ORCA_APPDIR=/tmp/orca-official-242/squashfs-root`;
no modified reference or preset substitution.

RED: both E2E tests failed (0.772 s, Nextest exit 100) because the exported
pattern was crosshatch instead of lockedzag. GREEN: after adding only the
two activation dependencies, both passed (1.907 s, exit 0), exercising
four complete comparisons with positive sparse-extrusion witnesses.
GREEN log/root: `/tmp/ares-locked-width-green.log` and
`/tmp/ares-locked-width-green.QIdh4F`.

The actual selected sweep also compared all four authored cases strictly:
`/tmp/ares-locked-width-sweep.log`, fresh artifact root
`/tmp/ares-locked-width-sweep.MQj0mr`. It correctly returned exit 100
(4.000 s), because both range domains remain INCOMPLETE. The external
summary records 4 generated/4 compared/0 rejected, not complete domains.
The tracked historical all-option summary is not overwritten by this
two-key run. Fmt check passes.

The original binary SHA-256 was rechecked:
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.

Static six-axis review found no blocking issue and suggested making the
positive-E witness's relative-mode assumption explicit. The tests now
verify exported `use_relative_e_distances=1`. A new fresh-root run at
`/tmp/ares-locked-width-reviewed.BvLzhp` passed both tests (5.364 s), log
`/tmp/ares-locked-width-reviewed.log`. Fmt passes after this assertion.

Follow-up domain discovery (not this slice's completion evidence): direct
original-CLI export/slice and Ares comparisons at 0 mm, 2 mm and 500%
match for each width. Original Orca rejects 0% during slicing with exit
156 and negative Flow spacing. Retained inputs/outputs and
`endpoints.log` are under `/tmp/ares-locked-width-probe.r4raHE`.
This supports a later context-bound domain slice; it neither changes the
current authored domain nor proves every endpoint/dependency is covered.

Workspace Nextest completed with 7380 passed, 9 skipped in 257.097 s
(`/tmp/ares-locked-width-workspace.log`). WASM, Windows and macOS compiler
checks completed successfully; logs are `/tmp/ares-locked-width-` plus
`wasm/windows/macos.log`. These checks do not claim cross-platform runtime
parity. `cargo clippy --workspace --all-targets` also completed successfully
with warnings (1m 28s, `/tmp/ares-locked-width-clippy.log`); no warning
references the new regression file. Independent final six-axis review
approved this limited seven-file slice without required fixes, including
the explicit relative-E assertion and terminal validation results. Both
range domains remain INCOMPLETE; this is activation repair, not global
parity completion.
