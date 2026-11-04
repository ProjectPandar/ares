# ARD: bounded stock config-validation ownership

Status: proposed source-only. No runtime probe, test or production change
under this slice.

## Non-negotiable decisions

1. #7 is a libslic3r rewrite/confirmation slice, not an independently designed
   Ares validation pipeline. Owners are PrintConfig.cpp:10337–10568::validate
   and Print.cpp:1524–1556,1652–1692::Print::validate width checks.
2. Scope is28 historical ORCA_ERROR candidates:20 loaded-config errors and8
   minimum-width errors. Each exact payload and line-number attribution is in
   [the spec](../superpowers/specs/2026-10-09-stock-config-validation-parity.md).
   Signals/UB/read failures/vendor incompleteness cannot become validation
   behavior. Eligibility, postprocess, templates and INPUT_REJECTED are outside.
3. Reuse options/validation loaded, aggregate, firmware_retraction and
   extrusion_width, registry metadata and print/validate/extrusion_width.
   Existing API orchestration is a compatibility shell around those owners,
   not permission to create another registry/validator or widen API scope.
4. Loaded-config ordered aggregate errors and applied-print width errors are
   distinct upstream phases. Compare authoritative detailed stderr/metadata,
   not generic CLI error descriptions. CLI transport/exit mapping is a
   separate adapter slice.
5. Historical evidence establishes leads, not current runtime RED. The mixed-
   nozzle Snapmaker record already reports payload parity on7a; do not ignore
   that result or relabel its reconstructed selection as original bytes.
   Failed-Export inputs cannot be claimed original archived replay inputs.
6. A fresh public-API mismatch must precede any production
   edit. If existing behavior is equal, deliver conclusion-only evidence;
   **no forced implementation**. Preserve first attempts and inconclusive cases.
7. No preset repairs/default invention, expected Unsupported assertions,
   fixture-specific exceptions, retry-to-PASS, crash imitation or new fallback.
   Success controls use complete output with generated-by whole-line-only
   normalization. Native outputs are not committed; inputs only may be committed.
8. Source/binary/input/commit/run bindings remain explicit. Raw stock ORCA_ERROR
   status is not rewritten as valid-stream PASS because a rejection matches.

## Source identities and failure-sensitive seams

Checked source PrintConfig.cpp SHA256
`f635101adb6da3ee657656c6fcaa9449871b7c17446b5e2eb59b560a0eb836aa`;
Print.cpp SHA256
`c9f66cd1750d12842f80249c99d70a43e976ee035b73dd8c0b1269de07ecc586`.

Nullable integer nil is not nullable-float NaN; bounds for
extruder_printable_height are not printable_height bounds. Bridge-width
percent resolution uses maximum nozzle while rejection uses minimum nozzle.
Width auto-zero, equality with layer height, bridge enablement and key-major
region iteration retain upstream semantics. No inferred per-stock first failing
width key is admitted solely from the shared `Too small line width` message.

The selector/flush prevalidation read-only audit found no portable missing
pre-crash detector. This ARD does not revive those exclusions or move validation
stages to reproduce unsafe upstream reads. Existing reports show no new missing
core detector; admission may legitimately end with a docs-only closure.
