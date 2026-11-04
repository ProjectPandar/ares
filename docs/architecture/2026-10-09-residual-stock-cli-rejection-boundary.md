# ARD: residual13 stock rejection channels

Status: proposed source-only.

Non-negotiable ownership: libslic3r GCode.cpp:3636-3645 and
PlaceholderParser.cpp:931/999 own template failures; Model/BuildVolume and
OrcaSlicer.cpp:5710-5724 own eligibility; OrcaSlicer.cpp:1682-1690 owns embedded
postprocess guard. CLI record_exit_reson/exception/error-family transport remains
an adapter to those boundaries, not an independent Ares pipeline.

Scope is five -100, four -50 and four -19 cases named in
[the spec](../superpowers/specs/2026-10-09-residual-stock-cli-rejection-parity.md).
No production mismatch is presumed. Preserve original/current/reconstructed
input identities separately. Public Core APIs compare typed messages; the public
CLI E2E compares process code and complete stdout/stderr/result bytes.

Reuse existing runner/cache recipe and #97 tests. No duplicate recipe or
wall-clock normalizer.
Only shared native_wall_clock removes prepare_time/export_time values; all
other metadata bytes, stdout and stderr remain exact.

Fresh unchanged runtime RED must precede any production port. All equal means
docs/test-only closure; Native crash/abort/read/vendor failures are non-PASS and
never emulated or converted into new guards. No frozen-reference fallback,
fixture repair, new API registry, changed expected channels or raw stock-status
promotion. Default Native absence is visible SKIPPED and required absence
fail-closed, before artifact access.

Native artifacts are not committed; conclusions retain actual run-to-commit
bindings.
