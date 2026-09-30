# Plan: Cut-retraction value coverage

Follow ARD-0047 and its spec. Source boundaries:
`PrintConfig.cpp` cut-retraction/nullable definitions and
`compute_filament_override_value`; `GCode.cpp::_do_export` placeholders.
Test seams remain the agreed CLI domain contract and project-to-full-G-code
boundary, not internal source structure.

1. Require the two Boolean domains, inherited distance endpoints and
   interiors, and nullable nil states; capture the missing-distance-nil RED.
2. Add only the selected missing numeric nil probe; preserve inherited
   source bounds and existing seeded sampling.
3. Add active runtime witnesses for the four keys, capture activation RED,
   then co-inject the proper gate and distinct machine/filament settings.
4. Execute all twelve generated cases plus a reverse Boolean nil witness
   and the real selected sweep; run full
   workspace, lint and platform checks.
5. Independent six-axis review, repair/re-review, code/doc commit and push.

## Evidence

Base 18d228b3. Reuse `/tmp/ares-cut-gate-isolated.rnuJct` only after
verifying its index/worktree exactly matched that already-pushed commit
and moving its detached HEAD to that commit. Main's unrelated work is
preserved. At plan creation verification was pending; no overall
completion is claimed.

Domain RED failed on the missing filament-distance nil case (0.476 s,
exit 100; `/tmp/ares-cut-values-red.log`). The selected nil addition
made it GREEN (0.492 s; `/tmp/ares-cut-values-green.log`).
Live activation RED failed all four keys because their exported gate
remained 0 instead of 1/2 (0.773 s, exit 100;
`/tmp/ares-cut-values-live-red.log`, root
`/tmp/ares-cut-values-live-red.ILzKy0`).

Extending only the cut activation branch fixed those dependency failures.
The new four-key suite and previous mode suite passed together: 8 tests
in 3.100 s, comprising 12 new and 6 prior complete-output comparisons plus
two domain tests (`/tmp/ares-cut-values-live.log`, root
`/tmp/ares-cut-values-live.KVVZ3c`). All comparisons directly use the
first reference output, with unchanged ordered-byte comparison.
No core production code was changed in this slice.

WASM/Windows/macOS compiler checks completed successfully; logs are
`/tmp/ares-cut-values-` plus `wasm/windows/macos.log`.
These are compilation checks, not platform runtime certification.
Full workspace/lint and the selected sweep were then started.

The actual selected sweep passed 4/4 domains: 12 generated, 12 strictly
compared, no rejections, exit 0 (`/tmp/ares-cut-values-sweep.log`,
root `/tmp/ares-cut-values-sweep.B9rSYm`). This is scoped fixture-domain
coverage, not all-printer/default evidence; the historical all-option
summary is not merged with this partial report.

Independent review requested a stronger Boolean nil witness. Retained
the machine-true case (which differs from upstream's false schema default)
and added a machine-false case (which differs from the activation's
concrete true filament setting). Both must render their machine value;
neither a constant true nor constant false implementation can pass.

The initial workspace run exposed the old range-shape whitelist missing
the additional nil: 1126 passed, 1 failed, 9 skipped in 20.979 s, with
6263 not run (`/tmp/ares-cut-values-workspace.log`). The whitelist now
allows min/max/seeded/nil only for the selected nullable distance key;
existing acceptance shapes are unchanged. Clippy did not run in that
failed chain. That repair required the fresh full verification below.

Focused repaired verification passed 9 tests in 3.115 s, including
13 new value/fallback comparisons, 6 earlier mode comparisons and the
3 domain-contract tests (`/tmp/ares-cut-values-reviewed.log`, root
`/tmp/ares-cut-values-reviewed.mjxseR`). No output comparison was relaxed.

Artifact audit confirmed 19 PASS manifests. Reference witnesses show
seeded machine distance 11.712, seeded filament distance 16.155, nil
distance falling back to 12, and Boolean nil falling back to both true
and false. The reference executable SHA-256 remains
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Independent repair review accepted the reverse-nil and exact range-shape
changes with no new required repairs. Scope is nine files, all five
changed Rust files under 400 lines (maximum 360).

Final full workspace Nextest passed 7390 tests with 9 skipped in 257.676 s
(`/tmp/ares-cut-values-workspace-final.log`). Clippy completed successfully
with warnings in 1.60 s (`/tmp/ares-cut-values-clippy.log`); this is not a
zero-warning claim. The final chained process exited 0. Final fmt and
diff checks passed. Independent six-axis final review approved the
nine-file code/documentation slice for commit/push with no required
repairs. Approval covers this fixed context, not the still-open overall
all-option/all-printer goal.
