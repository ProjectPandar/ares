# Filament volume precision implementation plan

1. Preserve the existing public full-output RED from the phase slice:
   `/tmp/ares-phase-isolated-live.log` and the 22.4 mm fixture. The source
   boundary and temporary compatibility shell are specified in ARD-0040.
2. Pass configured diameter into accounting and port per-extrusion float
   volume rounding before double accumulation and final length conversion.
3. Re-run full-output fill cases and positioning-mode tests. Verify the
   isolated phase-plus-accounting slice with nextest, fmt/clippy and Tier-1
   checks, then request independent six-axis review before commit/push.

Ranked hypotheses: per-move volume rounding; E parse/positioning precision;
different extrusion classification. Read-only reconstruction of identical
motion gives 18725.533830001856 by direct E summation, but
18725.535008176823 by upstream volume rounding. These reproduce the observed
.53/.54 formatting, supporting the first hypothesis. No reference, comparison
or motion generation change is part of this accounting slice.

## Execution

The minimal volume-precision change turns the preserved full-output fixture
GREEN, including the final 18725.54 mm statistic. The existing four grid
cases also pass: four tests / five complete-output cases, 1.422 s,
`/tmp/ares-volume-green.log` (exit 0). The oracle and comparator are unchanged.
This validates the per-move volume-rounding hypothesis for the real fixture,
not a private arithmetic snapshot. Independent review and isolated workspace
verification are still pending.

Independent review caught an incorrect early narrowing in the first patch:
`GCodeProcessor.hpp::AxisCoords` is double, so the float area is promoted
before multiplying the double delta and only the product becomes float.
The patch and ARD now preserve that boundary. The first GREEN alone was
insufficient to validate the arithmetic; corrected-code verification follows.

Follow-up source review also requires the actual reader boundary:
`GCodeReader.cpp::parse_line_internal` parses double then stores float;
processor positions are double and deltas follow endpoint updates even in
relative mode. The main patch now follows that chain. The main corrected
run `/tmp/ares-volume-reviewed.log` includes a 2.85 mm diameter case with
effective-setting verification. The initial isolated workspace run uses an
earlier patch without reader/endpoint changes and therefore cannot validate
the final corrected patch; repeat final isolation after its terminal result.

The corrected reader/endpoint/product chain passes all six full-output cases
in five tests, including both 1.75 mm and 2.85 mm tall fixtures:
`/tmp/ares-volume-reviewed.log`, exit 0, 4.760 s. Effective diameter values
are verified in the official exported project. No output normalization was
added. Final isolated workspace/Tier-1 validation and explicit public
absolute-E coverage remain open before independent approval and commit.

The first public absolute-E probe was rejected by official validation: the
Creality profile's per-layer G92 E0 is incompatible with absolute addressing
(`Print.cpp::Print::validate`). This is not a slicing RED. The fixture now
keeps the layer comments but omits that reset in absolute mode only.
Its sparse-extrusion counter tracks M82/M83, G92 and E deltas, so absolute
retraction/wipe coordinates cannot masquerade as positive extrusion.

The earlier isolated patch completes its full workspace run with 7373/7373
passing and nine skipped in 270.586 s
(`/tmp/ares-volume-isolated-workspace.log`). As noted above, this predates
the final reader/endpoint changes and cannot close their isolation gate.
Main corrected-code clippy succeeds with 227 existing warnings; main fmt
also succeeds. Final public absolute-E and isolation verification continue.

The valid absolute-E fixture passes strict complete output. All six test
functions / seven cases pass in 1.387 s
(`/tmp/ares-volume-absolute-valid.log`, exit 0), retaining at least 110
sparse-extrusion layers and checking the effective addressing mode.
The final reader/endpoint patch and all seven cases are now synchronized
into `/tmp/ares-phase-isolated.Pvc01L`; final live/workspace/clippy/fmt logs
use `/tmp/ares-volume-final-{live,workspace,clippy,fmt}.log`. This new run
must finish before it is counted as verification; Tier-1 checks and final
independent approval remain pending.

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
