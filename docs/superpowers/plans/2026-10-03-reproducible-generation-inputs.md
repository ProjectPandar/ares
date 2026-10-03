# Reproducible generation inputs plan

Source rewrite boundary and included/deferred scope:
[spec](../specs/2026-10-03-reproducible-generation-inputs.md),
[ARD-0095](../../architecture/ard-0095-reproducible-generation-inputs.md).
Existing `GenerationMetadata` is the header/template input owner; existing
`SlicerContext` owns the deterministic seed. Neither is a new pipeline scaffold.

1. Finish and locally commit the frozen ExportLines slice first. Do not modify
   its production snapshot while workspace gates are running.
2. RED: public CLI project route asserts epoch header and full output equality
   across two processes. Existing KSR full Orca comparator stays intact; its
   old wall-clock range assertion changes to the user's fixed-time contract.
   Browser harness likewise asserts epoch and complete repeated output.
3. GREEN: shared core default metadata uses Unix epoch zero; CLI/WASM adapters
   consume it without clock reads. Explicit core timestamps remain supported.
4. Replace Ares test ZIP implicit defaults with `SimpleFileOptions::DEFAULT`.
   Preserve explicit timestamp metamorphic variants. Verify representative
   ZIP output bytes and entry timestamps using existing fixture boundaries.
   Do not rewrite official exports or normalize comparator output.
5. Run focused public tests, actual Chromium WASM route, existing KSR complete
   comparison, workspace nextest, fmt/clippy/WASM, then independent six-axis
   read-only review. Commit code and documents together as a local slice.

Preflight evidence: CLI `main.rs:59–67` and WASM `lib.rs:32–41` read the current
local clock. All observed ZIP writers are test fixtures. ZIP8.6
`FileOptions::DEFAULT` fixes DOS epoch; `default()` can use current time when
the dependency's time feature is unified. Make the timestamp policy explicit.

Independent reviewer accepts this finite scope, requiring actual browser
execution, exact epoch assertions, retained full KSR comparisons and timestamp
variants. These controls must not be described as universal reproducibility.

ExportLines checkpoint completed locally as be5d592f after7563PASS and final
independent approval. Fresh CLI32198 terminated/reaped exit100,
/tmp/ares-reproducible-cli-red.log:two real CLI processes slice the same
small saved3MF successfully; fixed epoch header assertion fails (0.359s).
No comparator normalization is involved. Browser RED build88101 remains
live on unchanged production timestamps; existing Chromium143 host runtime
was checked successfully using the already installed Nix library paths.

Browser RED build88101 terminated/reaped exit0 (58.32s); bindings58283 exit0.
Actual Chromium76775 terminated/reaped exit1,
/tmp/ares-reproducible-browser-red.log:expected epoch header versus actual
2026-10-03 at20:41:22 after completing repeated real KSR project slicing.
Production timestamp changes began only after this terminal receipt.

Implementation retains explicit core timestamps and source formatting;
shared default metadata fixes Unix epoch0. Delete orphan browser date helper
and direct js-sys dependency; CLI's jiff is now only a dev dependency.
All20Ares ZIP-writing fixture files use explicit DEFAULT, with existing
timestamp metamorphic overrides intact. The current dependency feature set
already used DOS epoch implicitly; this is an explicit-policy hardening,
not a claim that those archives had a reproduced timestamp failure.

Independent static six-axis review reports no must-fix. CLI/KSR28184 and
WASM14065 builds remain live; no final approval yet. Browser same-page job
isolation alone is not all-platform/configuration/worker reproducibility.

Frozen implementation receipts: fmt78903 exit0; CLI/KSR28184 terminal/reaped0,
85PASS/2skip in18.345s (`/tmp/ares-reproducible-cli-green.log`), including
public two-process full bytes/epoch and retained complete KSR comparator.
WASM14065 terminal/reaped0 (1m48 including build-lock wait),
/tmp/ares-reproducible-wasm-green-build.log; bindings75689 terminal0.
fmt97377 and diff check exit0. The initial combined format/grep44944 exit1
was no remaining clock-pattern matches, not a format error. Final standalone
format receipt avoids conflating these statuses.
Clippy71595 terminal/reaped exit0 (1m20),
/tmp/ares-reproducible-clippy.log:253lib/269test/CLI31 unchanged.
Actual Chromium39685 terminal/reaped exit0:3PASS in1.9m,
/tmp/ares-reproducible-browser-green.log. The original full KSR comparison,
binding isolation and new repeated complete byte/epoch test all pass.
Complete comparison artifact: /tmp/ares-wasm-gcode-HTXSId/browser.gcode.
Workspace48036 terminal/reaped exit0:2m45 compile,260.486s run,
/tmp/ares-reproducible-workspace.log:7565PASS,35slow,9skip. The new complete
ZIP byte/entry epoch test passes (0.134s); existing spiral-vase and spacing
behavior coverage remains intact. Source stayed frozen throughout gates.
Request independent final finite local checkpoint review. Full dual-tool
parity remains an open RED boundary; no geometry/timing gap is claimed fixed.
These controls do not establish all-platform/worker/config reproducibility
and do not authorize push/main or global goal completion.

Final fmt28902 and cached diff check terminal0. Independent final six-axis
readback approves the36file local code+docs checkpoint with no must-fix,
limited to default time/ZIP policy and the listed public controls. Preserve
the explicit metadata boundary and deferred universal coverage; no push/main,
dual-tool complete parity or global goal completion approval is inferred.

## Header date removal after filament-change timing checkpoint

ff8a2517 closes the current finite timing checkpoint locally. The dual-tool
full comparison remains RED; no push/main/global completion is inferred.
Source inventory and exact deletions are in ARD0095. Independent six-axis
preflight approves automatic-header-only scope, strict per-producer patterns,
retained template variables and raw fixture hashes, and public CLI/browser RED.

1. RED: two real CLI processes and actual Chromium assert the exact undated
   Ares header against unchanged production. Retain raw complete repeat checks.
2. GREEN: static automatic header and delete its orphan metadata argument;
   preserve timestamp template inputs. Add public saved3MF template output
   proof. No vendor/user template rewriting without a separate scope decision.
3. Match only the new Ares format in strict comparators; retain Orca dated
   identity validation and all stream/mutation assertions. Recompute only the
   derived normalized hash; adapt synthetic comparator inputs, not real fixtures.
4. Focused full KSR/arachne/isolation/retry controls, workspace nextest,
   fmt/clippy/WASM and actual Chromium, then independent six-axis local review.
   Code and documents share a finite local commit; full dual-tool parity stays
   open and is not replaced by these reproducibility controls.

User confirmed automatic-header-only scope; retain vendor/custom template
semantics. CLI RED terminal exit100, /tmp/ares-header-cli-red.log, 0.363s:
actual epoch-dated line differs from exact undated expectation. Added public
saved3MF template test passes before production changes: seven epoch variables
emit `19700101-000000 1970 1 1 0 0 0`; no private injection.
Chromium RED58365 terminal/reaped exit1, /tmp/ares-header-browser-red.log:
first full KSR comparison succeeds, then exact undated header fails; repeat
test likewise fails on the old epoch header. Bindings privacy passes. Production
edits started after its terminal receipt. Comparator synthetic inputs adapt to
the new format; all real fixtures and body assertions remain unchanged.

Static six-axis review finds no new source must-fix. Date validation scope is
unchanged: the shared golden normalizer checks the original numeric date syntax,
not calendar validity; arachne/closest comparators keep their existing calendar
validation on the Orca side. Do not claim the shared helper validates calendars.
The undated sentinel changes only the derived KSR normalized SHA256 to
`164e7d1383d6086c368fd1110ad14fb3bb6cd105696ada9b3f12523b1ad02d73`;
both raw fixture hashes and committed inputs/reference bytes remain unchanged.
Processor isolation compares raw complete bytes now, with no timestamp filter.

Frozen header implementation receipts: focused84378 terminal/reaped0,
/tmp/ares-header-focused.log, compile2m49 and96PASS18.703s. Includes complete
KSR (18.679s), generator/stream mutation contracts, synthetic retry rejection,
arachne full controls, processor isolation, CLI raw repeat and seven epoch
variables. Clippy44551 terminal/reaped0, /tmp/ares-header-clippy.log,4m10
including lock wait; unchanged253core-lib/269core-test/31CLI-parity warnings.
WASM41410 terminal/reaped0, /tmp/ares-header-wasm.log,4m52 including lock
wait; browser bindings4207 terminal/reaped0. Fmt and diff checks0. Workspace
and final actual Chromium execution are pending, so no final approval yet.

Final workspace4388 terminal/reaped0, /tmp/ares-header-workspace.log:
compile6m22 including lock wait,7568PASS/38slow/9skip,269.005s runtime.
Spiral-vase ramps and the retained four-assertion current-spacing behavior
test both PASS. Chromium58760 terminal/reaped0, /tmp/ares-header-browser-green.log:
3PASS4.8m, including CLI comparison dependency rebuild. Full KSR artifact
`/tmp/ares-wasm-gcode-56hp8M/browser.gcode`, exact undated header, binding
privacy and repeated complete raw bytes all pass without timeout/byte-assertion
changes. Source remains frozen; final fmt/diff checks0.

Fresh whole-process CPU0 official84792 and seeded84294 both terminal/reaped100,
/tmp/ares-header-official.log (9.376s), /tmp/ares-header-seeded.log (10.426s):
each single-tool modifier/infill-first full comparison PASS, original full
dual comparison RED. Baseline first difference is now byte1913/line143,
same M73-before-G0 ordering; modifier byte703/line25, same R16 versus R17.
The15-byte offset reduction is only the undated normalization sentinel;
actual body differences persist. Artifacts retain both references under
/tmp/ares-header-{official,seeded}. No fixture/assertion changes or complete
dual-tool GREEN is claimed. All launched handles are terminal/reaped.
Request independent final six-axis approval for the finite16file local
code+docs checkpoint; push/main/global closure remain outside this approval.

Independent final six-axis readback approves all16files as a finite local
code+docs checkpoint with no new must-fix. Scope excludes push/main, complete
dual parity, universal reproducibility and global completion. Retain original
dual RED in the commit body; no source edits after the verified snapshot.
