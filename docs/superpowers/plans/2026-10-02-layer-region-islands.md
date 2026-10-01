# Layer-region aggregate islands plan

Source/contract: [spec](../specs/2026-10-02-layer-region-islands.md).
Architecture: [ARD0087](../../architecture/ard-0087-layer-region-islands.md).
Base: d364da4f8997eed44f2eb1b3f4309de3a0841159.

## Scope and ordered checks

1. Retain prior source-owned supplemental RED: two multi-region geometry
   cases panic at the old single-region unreachable branch. Public modifier
   E2E remains RED at an earlier scaffold guard; never call that parity.
2. Isolate only make_slices, existing caller/error propagation and tests.
   Exclude Normal raw-slices fields, scheduling and slicing-parameter changes.
3. Verify isolated workspace, unchanged KSR complete output, lint/fmt,
   WASM/Windows/macOS compilation and actual browser output. Run a finite
   untouched-official complete-output control through the public project API.
4. Freeze patch/tree, independent six-axis review, repair/re-review, then
   commit/push only this independent prerequisite with its documentation.

## Prior development evidence

Supplemental RED: /tmp/ares-layer-make-slices-multi-region-red.log,
exit100, two old-unreachable failures. Dirty-tree final eight ordering/
compensation helper tests PASS in0.030s after repairing a private-module
import mistake. Full dirty workspace7478PASS1unchangedTreeFAIL9skip;
live finite controls5PASS1modifierRED. These dirty-tree results do not
prove this independent patch; isolated results must be recorded below.

## Remaining project gates

Modifier all-region output, original KSR Tree width test, Tree type/style
output matrix, known custom-end native path-order replay difference and
all-printer/all-option coverage remain open. No reference cache/scheduling
policy changes or convenient reference selection are authorized here.

## Isolated evidence

Tree: /tmp/ares-make-slices-isolated.e2Pd30, only the independent slice.

- Workspace log /tmp/ares-layer-islands-isolated-workspace.log, exit0:
  7464run7464PASS9skip36slow,269.085s. Both new geometry tests pass;
  unchanged KSR CLI complete bytes pass. The existing support-width test
  also passes on this HEAD adapter; that header observation does not prove
  real Tree support generation or close its required complete-output gate.
- Clippy /tmp/ares-layer-islands-isolated-clippy.log exits0 after5m50s,
  268existing core-test warnings (252duplicates). fmt/check and diffcheck0.
  Changed Rust modules are92/380/73 lines; no include-based split.
- WindowsGNU /tmp/ares-layer-islands-isolated-windows.log exits0/5m17s;
  macOSx86_64 ...-macos.log exits0/5m35s. Compile-only, not native runtime.
- WASM release ...-wasm-release.log exits0/5m10s; bindgen exits0 into
  /tmp/ares-layer-islands-isolated-bindings.YthVR6. WASM SHA256
  d3dc1f980db52f735ff93dfc0f96659ddc7f11075f5f37f464bacf971ee06199.
- Official controls ...-official-single-run.log exits100/0.888s:
  Ender-3 PASS, top-concentric FAIL. ARES_ORCA_RUNS=1, fresh artifact root
  /tmp/ares-layer-islands-isolated-official-single-run-output. Initial
  invocation omitted the single-run limit and was SIGINT130 cancelled
  while waiting for the build lock, before tests/slices; no retry selection.

Ender-3 case-iWv3Gl inputSHA256
6e72f9f5a562e057bea4b45440ca61a276b937c42351b80e0222af55fb3da39d,
native G-codeSHA256
bde8afb59c57ee8c43306b137e98f400b60d388a857c995298c878c7047c707d.
The harness cache manifest says producer unknown; it is not identity proof.
Separate fresh command journals record the actual wrapper's explicit
ORCA_APPDIR=/tmp/orca-official-242/squashfs-root and successful reaped
export/slice processes; no reference replay attempt is used. Official
executable SHA256
64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60.

Browser log /tmp/ares-layer-islands-isolated-browser.log exits0; actual
Chromium143.0.7499.4 invokes the isolated WASM public project API, not injected
G-code. Complete browser/host/reference bytes match except generator line,
nonfinite motion=false. Retained directory
/tmp/ares-layer-islands-isolated-browser-output.w0CEmt, browserSHA256
467bbbe44fa1db6a904bf14683ccba964521d840613d127e175cc96421691230.
This is finite Ender-3 browser evidence, not all-printer/browser completion.

## Retained concentric failure and baseline classification

case-cEh74m inputSHA256
04eb01c8c985fe382c18ac9c10abc9c9c304a2c2c7519978de6084fe18ff9f82.
First native difference byte3265/line149: M73 placement, with later complete
stream differences retained. Do not classify the whole difference as timing
only, harmless noise, or proven native UB. The original HEAD is replayed on
the exact same input, not another native output:
/tmp/ares-layer-islands-baseline.kt0tUV at d364da4f, CLI exits0/40.97s build.
/tmp/ares-layer-islands-baseline-concentric-compare.log exits0 and proves
baseline and patched complete streams equal except generator line, while
both differ from the retained official stream. Shared normalizedSHA256
2eaa4db352cdc1487e01a38c400494c3200300255dff533062f8d4b685bf498e.
Thus this captured failure is inherited, not an islands regression; its
underlying output-parity/root-cause gate remains open. No native rerun,
comparator weakening or additional production instrumentation was used.

## Review and submission gate

Six-axis static scope review found no required repair. Freeze the exact
independent patch/tree including these execution records and seek final
review. The retained inherited failure does not count as a PASS. No commit
or push has occurred at this checkpoint.
