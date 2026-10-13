# C2-T global-role validation RED

PLAN `a5f445512c4e7537dcd47f6de64eaa7ca8e480e0` approved.
This test-only commit precedes both production changes.

Upstream `PrintConfig.cpp:10400-10421` validates the four global pattern
subsets using `Config.hpp:2498-2503::has_enum_value`. Local metadata remains
full-keymap input (`Config.hpp:2084-2092`, `PrintObject.cpp:3664-3709`).
The Rust target remains the existing global validation call site and local
top guard described in `bridge-anchor-top-pattern-repair-plan.md`.

The public fixtures change only `Metadata/project_settings.config` in a
Native-exported ceiling 3MF. Every other ZIP member is SHA-checked unchanged.
A local rectilinear top override intentionally remains present: invalid raw
global configuration must be rejected even when local metadata overrides it.

Four independent public API/CLI tests cover global top/bottom/solid Grid
and sparse Monotonic. Native reference uses the existing bounded cached
InvalidValues CLI runner, which checks the -18 exit family. Each test asserts
the entire exact stderr and absence of Native G-code. It saves the public
input, Native stdout/stderr/result, receipt directory, Ares output/error and
CLI stdout/stderr/status before the public API error assertion.

Official CPU13 and seeded CPU14 both completed before 18:20, jobs2/threads2,
outer cargo pinned by taskset and fresh cache/artifact root per leg:

- official `/tmp/worker2-c2t-global-red-official.YmDmbH`: 0 PASS, 4 FAIL.
- seeded `/tmp/worker2-c2t-global-red-seeded.eMobHX`: 0 PASS, 4 FAIL.

All eight exact Native diagnostics pass. Public Ares accepts global top,
bottom and sparse invalid values (API/CLI success); global solid instead
returns UnsupportedProjectFeature(internal_solid_infill_pattern), rather
than InvalidValuesInProject. Every failure reaches the public API config
error assertion; no setup, timeout or oracle failures.

External joined evidence: `/tmp/worker2-c2t-global-red-evidence.json`.
Native artifacts remain external and are not committed. Existing local RED
assertions are unchanged. C2-S and numeric D behavior remain deferred.
