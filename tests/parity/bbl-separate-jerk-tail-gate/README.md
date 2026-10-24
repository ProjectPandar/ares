# BBL separate-flavor jerk tail inputs

Source: `GCodeWriter.cpp:298-299::set_jerk_xy`. The BBL Z/E suffix follows all
flavor branches, before the optional comment and newline. Specification:
`docs/architecture/2026-10-24-bbl-separate-jerk-tail-gate-spec.md`.
Base: `17ec2e082`. This tests-only RED does not change production code.

Native admits the BBL/Repetier saved configuration. The coordinator explicitly
accepted that cross-family boundary as in scope. This is not a claim that stock
Bambu hardware runs Repetier or a recommendation to use this printer setup.

BBL cases derive from the existing GUI-generated BBL one-object archive; the
non-BBL control derives from the genuine `orca_cli_ender3.3mf`. Every case has
its tower disabled, `default_jerk=9` and `travel_acceleration=6000`. Only the
BBL target's saved flavor differs from the BBL control. No geometry or template
changes are made. The Ender3/Repetier control verifies that ordinary non-BBL
commands retain no Z/E suffix.

| Input | SHA256 |
|---|---|
| stock-bbl-control.3mf | 6cb522350420360807f04e3e920a71c71286a92583f2d513a8def2eec5b2c975 |
| bbl-repetier-tail.3mf | 92dbe0333f559c08a88f125018bf719dfcf2d2588b1cff98e7b059a634f58204 |
| ender3-repetier-control.3mf | bf34d2e7ccfc2e33b1311cab464d1f57f7883da5165a4f643da309d98df38254 |

Three direct calls use the existing `native_stream_case` seam. Fresh Native
replay qualifies first. Callbacks read saved printer identity, flavor and Z/E
values, select the actual Native body prefix, and qualify the tail presence and
text. The selected values have at most two significant digits, so ordinary
input-derived text suffices independently of the Ares formatter. For the
non-BBL control the callback verifies absence of both tail words. No hardcoded
Native coordinates, counts or output reference files are committed.

Complete Native/API/CLI comparison then uses the unchanged generator-only
normalization. M73, headers, configuration and footers remain included; raw
API==CLI is required. Callbacks are observations, not masks. Expected RED:
target fails at byte 67234 (`M207 X9 Z3 E2.5` versus `M207 X9`), both controls
pass. Discovery's complete target diff consists of 299 missing suffixes with
all M73 records identical; public RED must qualify this independently.

Failure modes were written in the SPEC before implementation: gate trapped
inside M205; tails on non-BBL commands; formatter/cache/clamp regressions;
comments placed before the tail; or hiding unrelated travel/time differences.
The travel=10000 M202 discrepancy is explicitly deferred and that archive is
not included. Do not rename a BBL model to manufacture a non-BBL control.

Freeze these tests and archives before GREEN. The later port only moves the
existing suffix after flavor selection, reuses precision two and preserves
upstream tail/comment/newline order. CPU17/PIN17, jobs=1 and threads=1; no FULL.
Send one combined SPEC/RED review request, then hold before production changes.
