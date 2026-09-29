# Cross-zag phase precision implementation plan

Follow ARD-0039 and the matching source-cited spec. The upstream boundary
is Fill.cpp's cross-zag/locked-zag phase assignment; the Rust projection is
only a compatibility shell around that behavior.

1. Find a complete-output red case using the unmodified official oracle.
   Verify effective settings and real sparse extrusion at sufficient layers.
2. Move only the phase's f32 narrowing after the f64 multiplication. Verify
   the tracer and existing fill-grid cases without changing the comparator.
3. Run fmt/clippy, workspace nextest and Tier-1 checks, isolating this patch
   from pending unrelated work before independent six-axis review.
4. Resolve review findings and commit/push code plus docs together. The
   global printer/option parity goal remains open.

## Diagnosis

Arithmetic search suggested candidate steps, but is not regression evidence.
The first 7.6 mm slab with step 0.0655373 passes full output in 0.857 s
(`/tmp/ares-phase-red.log`), so no production change followed that run.
A 41.2 mm slab with the same step fails at layer 109, line 23868:
official E1.47736 versus Rust E1.47735. The live run exits 100 in 1.373 s
(`/tmp/ares-phase-search.log`); full input/output artifacts are in
`/tmp/ares-phase-search.8rDjXB/case-MvBWFz`.
The next probe reduces height to 22.4 mm and requires 110 real sparse
extrusion layers, retaining the failing layer without unnecessary height.

The 22.4 mm probe remains genuinely RED at line 23903 with the same E
difference (`/tmp/ares-phase-min-red.log`, exit 100, 1.092 s). After the
one-line phase narrowing change, all motion matches but the full-output
comparison remains RED at the final filament length: official 18725.54 mm
versus Rust 18725.53 mm. The complete diff contains only this statistic and
the permitted generator header difference. Artifacts:
`/tmp/ares-phase-green.0OWCkM/case-FuH3OQ`; despite the exploratory log name,
`/tmp/ares-phase-green.log` is NOT a green result (three tests pass, one fails).

Isolation from HEAD fc98a2d2 with only the one-line production patch and
fill_grid test changes reproduces the same statistics failure:
`/tmp/ares-phase-isolated.Pvc01L`, `/tmp/ares-phase-isolated-live.log`,
three tests pass and one fails in 1.372 s. The chained workspace run did
not start because live acceptance failed. This rules out dependence on
the main tree's pending bridge/affine/timelapse and user geometry changes.

Independent static six-axis review finds no blocking code issue, but
withholds final approval pending strict GREEN and verification. No commit
or push has been made for this unfinished slice. Keep the 22.4 mm case;
do not waive statistics or shorten it merely to evade the failure.
The next source-boundary diagnosis is
`GCode/GCodeProcessor.cpp::process_G1` and `UsedFilaments` volume accounting
versus `gcode_emit/finish.rs::account_used_filament`. Upstream rounds each
extruded volume to float before double accumulation; Ares currently sums
double E directly. This is a candidate cause, not yet a proven repair.

A read-only arithmetic reconstruction from the actual identical emitted
motion strengthens that hypothesis: direct E sum is 18725.533830001856;
rounding each E and cross-sectional volume to f32, accumulating volumes in
f64, then dividing by the f64 filament area yields 18725.535008176823.
These format to the observed Rust .53 and official .54 respectively.
This diagnostic only selects positive-E XY commands for this known relative
fixture; it is not a general processor implementation or acceptance test.
Any accounting repair needs its own source-cited boundary and public tests.

Isolated clippy succeeds with 249 existing warnings (27.80 s), and fmt
check succeeds; logs are `/tmp/ares-phase-isolated-{clippy,fmt}.log`.
Live full-output acceptance remains red, so workspace and cross-target
completion are still pending rather than claimed as passing.

The separate source-cited filament-volume precision dependency (ARD-0040)
now turns the preserved 22.4 mm complete-output case GREEN, without removing
statistics from comparison. `/tmp/ares-volume-green.log` passes four tests /
five cases in 1.422 s. Isolation and final review are still pending, so the
combined source slices have not yet been committed.

After independent review corrected the accounting dependency's reader and
product precision boundaries, `/tmp/ares-volume-reviewed.log` still passes
the tall phase regression and all existing cases, plus a 2.85 mm diameter
case: five tests / six cases, 4.760 s, exit 0. Final isolation and approval
remain pending; no broadened parity-completion claim is made.

## Final verification

The final isolated patch on HEAD fc98a2d2 passes all seven live complete-output
cases in 1.444 s and all 7375 workspace tests (nine skipped) in 255.684 s.
Both KSR regressions pass. Clippy finishes successfully with 249 existing
warnings in 14.27 s; fmt check also succeeds. The chained process exits 0,
so the empty fmt log is backed by terminal status, not assumed success.
WASM, Windows and macOS compilation checks all finish successfully with a
separate chained exit 0. Logs use `/tmp/ares-volume-final-*.log`.
No reference or comparator changes were made. The commit scope is the five
code/test files in the isolated diff, these two source-cited slice document
sets and the roadmap paragraph; unrelated pending work remains excluded.
This closes the precision dependency pair's validation, not the global
all-printer/all-option or full KSR byte-parity goal.
Final independent six-axis review approves the two dependency slices with
no remaining fix requests. Its approval excludes all other dirty slices.
