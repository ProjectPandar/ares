# Plan: Filament-map wire-domain parity

Base 403bc2bb, clean reused `/tmp/ares-enum-artifacts.XZ9HkF`.
Reuse agreed generated-domain, original-export application proof and complete
project-output comparison seams. Boundary/inclusions/deferred: ARD-0055.

1. Reproduce selected sweep; add valid-wire domain regression RED.
2. Prefer source key-map for this option without changing other domains.
3. Direct all-token full outputs plus invalid UI-token negative proof.
4. Selected sweep, workspace/lint/format/platform checks and independent review.
5. Repair/re-review, then commit/push code+docs.

Baseline selected RED: exit 100, 3.550 s. Three legal tokens compare; Default
requests export as Auto For Flush and fail strict application proof
(`/tmp/ares-filament-map-red.log`, `/tmp/ares-filament-map-red.POwUmS`).
The source map excludes Default; the UI list includes it. The existing typed
Rust enum already has the correct serialized domain; no production fix needed.
Domain RED: 0.482 s, four UI values versus three wire tokens
(`/tmp/ares-filament-map-domain-red.log`). Initial map extraction still failed
because the old parser omitted the `= {` initializer form; the parser repair
then passes the typed-domain regression. A later direct run deliberately
checks final G-code mode and exposes stock plate precedence
(`/tmp/ares-filament-map-verified.log`, 2.325 s), rather than accepting three
project-level values that all slice under Auto For Flush.

The native test adapter now binds the concrete mode into the generated plate,
verifies both owners and uses new complete-project SHA-256 cache paths.
Three active direct complete outputs pass (3.196 s,
`/tmp/ares-filament-map-bound.log`, `/tmp/ares-filament-map-bound.UVnMHO`).
Selected sweep plus dual-owner proof test: 2 PASS, 3.385 s; 1/1 wire domain,
3 generated/strict comparisons, 0 rejected, with plate-aware evidence JSON
(`/tmp/ares-filament-map-sweep.log`, `/tmp/ares-filament-map-sweep.5w1kCJ`).

Independent pre-review requests I/O classification repair for filesystem
storage errors. Regression RED: 0.042 s, ApplicationMismatch instead of Io
(`/tmp/ares-filament-map-io-red.log`). The repair uses FailureKind::Io for
directory/file writes, keeping parse/value errors as application mismatches.
Final direct/domain/dual-owner/I/O tests: 4 PASS, 8.308 s
(`/tmp/ares-filament-map-final.log`, artifacts
`/tmp/ares-filament-map-final.leOAxR`), including three first-reference
complete-output comparisons. Final exact-code selected sweep: 1 PASS,
3.449 s, three generated/strict comparisons, no rejected cases
(`/tmp/ares-filament-map-final-sweep.log`,
`/tmp/ares-filament-map-final-sweep.bxkJuO`).

Final workspace: 7419 PASS, 9 skipped, 251.653 s
(`/tmp/ares-filament-map-workspace-final.log`). Format/diff checks and
default/parity-target Clippy pass; existing warnings remain 248/28.
WASM adapter, Windows GNU workspace and macOS aarch64 workspace compiler
checks pass (`/tmp/ares-filament-map-{wasm,windows,macos}.log`);
these are compiler checks, not runtime parity.

All six final-direct/initial-sweep manifests have verified input/output sizes
and SHA-256 hashes. Decompressed archive payloads match the original export
except the single plate metadata replacement; official output headers name
the requested modes. Official binary SHA-256:
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Manifests retain unknown producer identity; invocation and binary hash are
separate evidence, not independent cache provenance.
Final independent six-axis re-review approves this eleven-file slice after
the I/O repair: requirements, correctness, boundaries, quality, coverage and
actual runs pass, with no remaining code repair items. Multi-material mapping,
ARD-0054 and the global parity gate remain outside this acceptance.

ARD-0054 resolution diagnostics remain in a
different worktree and are excluded from this patch.
