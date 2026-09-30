# Plan: Thumbnail-definition parity probes

Base 17a214e0; isolated `/tmp/ares-options-current.rPJrPW`.
Use the agreed generated-domain, project application and full-output seams.
Source/destination/included/deferred boundaries: ARD-0056.

1. Reproduce application mismatch; add canonical-domain regression RED.
2. Replace shorthand authored probes with explicit-format cases.
3. Verify original CLI shorthand/default-format normalization and every
   canonical case, then compare first-reference complete outputs.
4. Run selected/full tests, fmt/Clippy and Tier-1 compiler checks.
5. Independent six-axis review, repair/re-review, code+docs commit and push.

Baseline RED: `/tmp/ares-thumbnails-red.log`, 1.722 s, exit 100;
requested `300x300`, effective `300x300/PNG` fails application proof.
Artifacts: `/tmp/ares-thumbnails-red.jxiqXu`.
Current full-option scan is a separate immutable base-17a214e0 run, not
evidence for this pending patch.

Domain RED: 0.486 s (`/tmp/ares-thumbnails-domain-red.log`). Canonical
probes pass the domain regression, but direct full output fails BTT_TFT:
official omits the header and Ares emits it. Three earlier formats compare.
`/tmp/ares-thumbnails-direct.log`, 3.022 s, artifact
`/tmp/ares-thumbnails-direct.7sS9Lx/case-IqlhSx` retains both full streams.
Extend step 3 to the source-owned header omission and explicit input-selected
raw byte comparator, with artifact-level mutation regressions, per ARD-0056.

BTT artifact regression RED: 0.125 s, the project API still emits HEADER_BLOCK
(`/tmp/ares-btt-artifacts-red.log`). After the source branch port, all three
targeted tests pass in 7.407 s (`/tmp/ares-thumbnails-bound.log`): twelve
original CLI full-output cases plus domain and synthetic mutation checks.
Independent pre-review requests removing a misleading fixed comparator name
from artifact logging; repaired without changing comparison semantics.
BBL state side effects and raw unnormalized 3MF mode selection remain deferred.
Full-check results follow below.

Final targeted run after review/lint repairs: 3 PASS, 8.196 s
(`/tmp/ares-thumbnails-final.log`); twelve first-reference complete outputs
in `/tmp/ares-thumbnails-final.vLoKKt`. Selected combined sweep: 1 PASS,
23.979 s, seven definitions plus five format-enum values, 12 strict
comparisons and zero rejected (`/tmp/ares-thumbnails-sweep.log`,
`/tmp/ares-thumbnails-sweep.7GpgKM`). All 24 paired manifests have verified
file sizes and SHA-256 hashes. Two direct and one sweep BTT output pairs
are raw byte-identical without normalization.

Official binary SHA-256:
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Invocation/binary-hash evidence is separate from the manifests' unknown
cache producer provenance; no independent identity claim is added.
WASM adapter, Windows GNU workspace and macOS aarch64 workspace compiler
checks pass (`/tmp/ares-thumbnails-{wasm,windows,macos}.log`); these do
not establish platform runtime parity. Clippy is back to existing 248 core
and 28 parity-test warnings after removing two new nested-block warnings.
First workspace run: 7422 PASS, 9 skipped, 263.820 s
(`/tmp/ares-thumbnails-workspace.log`). Final exact-code workspace chain:
7422 PASS, 9 skipped, 267.843 s
(`/tmp/ares-thumbnails-workspace-final.log`), default/parity Clippy
(248/28 existing warnings), fmt and diff checks all pass (exit 0).
Final independent six-axis review approves the eleven-file slice after the
logging/scope/Clippy changes and terminal evidence, with no remaining code
repair items. Approval excludes the deferred boundaries in ARD-0056 and
does not close the global parity gate.
