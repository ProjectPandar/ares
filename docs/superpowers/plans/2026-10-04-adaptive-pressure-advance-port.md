# Adaptive PA selective-port plan

1. Confirm the source and current Rust ownership in the matching spec; establish
   a public constant-model RED without a tower dependency.
2. Port model/PCHIP/scanner and PA_CHANGE generation from WIP `a53bc00c`,
   adapting only source-owned behavior to current region/filament interfaces.
3. Make the tracer GREEN, then add nonconstant, parser, cooling/overhang and
   lifecycle cases individually, preserving complete-output comparison.
4. Verify official and seeded references separately, full workspace Nextest,
   Clippy, rustfmt and WASM, then commit and push this slice's feature branch.
5. Per the latest review rule, print `REVIEW-REQUEST <commit-hash> <summary>`
   in the terminal and stop. External Codex/OpenCode review and external
   completion-gate programs are no longer required or permitted. Await the
   coordinator Claude's `APPROVED <same-hash>`; do not start another slice on
   this branch while waiting.
6. Only after approval: fetch and rebase onto latest `origin/main`, rerun
   workspace Nextest, Clippy, rustfmt and WASM, then
   `git push origin HEAD:main` (fast-forward only). If rejected because main
   advanced, repeat rebase and all checks; never force-push.

## Baseline

Worktree `/tmp/ares-pi-tower`, branch `pi/timelapse-tower-adaptive-pa`,
base `c3f61645`. All Cargo commands use `/tmp/ares-pi-tower-target`.
The pristine upstream source is exported locally from the pinned upstream Git
object; the replay tree's FuzzySkin modification is not copied.
Bridge-direction behavior from the WIP is already in main under ARD-0075
(`50fe97a6`); do not overwrite it or replay historical MSVC pins.
The old WIP tower's unsupported-feature guards are explicitly not transferable.

## Current execution

The original CPU0 binary SHA-256 is
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Fresh no-tower constant-model RED:
`/tmp/ares-pi-tower-pa-red.log`, Nextest run
`181886a4-1eca-4c09-8e81-ed03fc07b4bc`, one executed test failed in 1.767s
(after the isolated target's initial 15m52s build). Exported PA activation/model
and the original static/adaptive command sequence passed their assertions.
Complete paired artifacts: `/tmp/ares-pi-tower-pa-red-official/case-pCvyuZ`.
The first mismatch is line 69: missing `M900 K0.045`; 100816 reference bytes
versus 100771 Ares bytes. No movement or timing normalization was introduced.

The first constant-model official GREEN completed in
`/tmp/ares-pi-tower-pa-green.log`: one live test PASS (2.105s), Nextest run
`52862fd5-5aca-487a-8fce-e47856dcbb7d`. The subsequent matrix adds nine active
scenarios: multi-acceleration interpolation, decimal prefixes, invalid/insufficient
models, both disabled-activation controls, disabled cooling, overhang/bridge
and source diagnostic comments. `/tmp/ares-pi-tower-pa-matrix-official.log`
passes both tests (ten full-output cases). These are finite no-tower cases,
not proof of arbitrary calibration strings, multi-tool resets or smooth towers.

The selective implementation is being validated. It retains current region
selection and actual logical tool IDs, unlike the WIP's fixed tool-zero tags.
Source PA resets are captured as generation-layer events, not inferred from
arbitrary user-written M900 commands. Loop clipping/weights and the existing
static formatter are reused without copying the WIP's obsolete emitter.
Checkpoint authorized by the coordinator: preserve the self-contained port
and explicit open regressions on the work branch, then rebase and verify.
This is not completion approval and must not be fast-forwarded to main while
any live parity case remains RED.

## Checkpoint evidence and remaining work

- Earlier workspace: `/tmp/ares-pi-tower-pa-workspace.log`, 7621/7621 PASS,
  nine existing skips, 327.633s. Its binary predates the last get_at/negative-PA
  repairs and newest fixtures; it is not final checkpoint validation.
- Official and seeded runs each execute seven tests: five PASS, two FAIL.
  Logs: `/tmp/ares-pi-tower-pa-bridge-final-official.log` and
  `/tmp/ares-pi-tower-pa-bridge-seeded.log`. All four bridge/affine tests and
  the constant PA tracer pass in both environments. The initialized seeded
  reference is `/nix/store/jqfsg42f6xzq3krnf5scm6w4g1z4kz4c-orca-slicer-seed-replay-2.4.2/bin/orca-slicer`,
  explicitly seed 0 / CPU 0, with a separate artifact root.
- Multi-tool RED is an index-out-of-bounds on the one-element overhang option
  for logical tool 1 (official `case-JXHT9a`). The checkpoint fixes **all**
  PA option reads with upstream `ConfigOptionVector::get_at` first-value
  semantics. The full two-tool and mixed-activation cases still require GREEN.
- Negative-default RED (official `case-LakHOa`) emits erroneous `M900 K-1`
  where upstream emits nothing. The checkpoint moves the existing negative
  suppression into the shared command formatter, preserving reset state.
  This repair still requires the fresh live GREEN run.
- **Open parser:** `std::stod` accepts hexadecimal calibration prefixes;
  the port's decimal-only parser does not. The checkpoint adds a public
  fixture; its post-rebase RED is recorded below. Port source semantics,
  not a rejection or silent exclusion.
- **Open formatter:** four-significant-digit output must preserve integer
  trailing zeroes and scientific notation. The checkpoint adds integer and
  scientific controls. The preliminary review's example claiming 10 becomes
  1 was incorrect: the 10 control passes. The genuine high-value/scientific
  failure is recorded below; no expected-output relaxation is allowed.
- Independent preliminary Codex review:
  `/tmp/ares-pi-tower-pa-preliminary-review.md`, run
  `7be90eb6-86e0-4fc2-8164-02e6c81d7e8c`, verdict changes required.
  It confirms the get_at panic, hex parsing and formatter issues; flags reset
  lifecycle/performance as unverified. This is not publication approval.
- Smooth Type2 tower planning/emission and tower reset are **not ported** in
  this checkpoint. WIP tower guards and unrelated support/AGG changes remain
  excluded. No all-printer/all-option completion is claimed.

Before rebasing, the ongoing pre-rebase Clippy chain was deliberately stopped
(session 24, SIGTERM) after the 7621-test workspace pass; that interrupted chain
is not Clippy/fmt/WASM success evidence. All gates will be rerun after rebase.
`git hash-object` of local `GCode.cpp`, `Config.hpp` and all three processor/
interpolator `.cpp` files exactly matches their blobs at pinned upstream
`8500fcdccaa10b5099ac20d252af3a7c560046f1` (command log
`/tmp/pi-unified-exec-28-93aa8b1f.log`). This closes the preliminary review's
local-source provenance uncertainty without altering the reference binary.

## Rebased checkpoint `81a7a66c`

Coordinator-authorized checkpoint `cc5cc244` rebased cleanly onto
`origin/main=9a7b46fc`, producing `81a7a66c`. Main's test removals are preserved;
no deleted unit tests are restored. There are no tower or unrelated geometry
changes in this commit.

Fresh official CPU0 and seeded CPU0 runs both execute seven tests with five
PASS and two FAIL. Full logs and paired streams:

- `/tmp/ares-pi-tower-checkpoint-official.log`, artifacts in
  `/tmp/ares-pi-tower-checkpoint-official/`, run
  `02d9319b-cb12-43a6-a833-8ba16bb5cae9` (45.884s).
- `/tmp/ares-pi-tower-checkpoint-seeded.log`, artifacts in
  `/tmp/ares-pi-tower-checkpoint-seeded/`, run
  `2416929d-06a1-4778-b2b0-acf0a071e47e` (101.973s).

The constant tracer and all four bridge/affine tests pass. The model matrix
completes all 13 cases: 11 pass, including negative-default and the integer-10
control. Two cases remain strictly RED in both references:

1. Hexadecimal model: expected `M900 K0.031`, actual `M900 K0` at line 69.
   Official pair `case-RnDqVr`, seeded pair `case-oFuJEe`.
2. Scientific model: expected `M900 K1e+05`, actual `M900 K1` at line 69.
   Official pair `case-RmjvSZ`, seeded pair `case-2Ygs2S`.

The multi-tool panic is gone. Its complete comparison is still RED, first at
line 23: source `M205 J0.010 ; Junction Deviation` versus Ares `M205 J0.010`.
Official pair `case-Otc9u6`, seeded pair `case-z8MDJn`. This is a startup-comment
mismatch, not evidence that PA reset is its cause; later differences require
inspection. The test stops on this failed all-enabled case, so the subsequent
mixed-activation case remains unexecuted and unclaimed.

Post-rebase gates on this exact code snapshot pass:

| Gate | Result | Log |
| --- | --- | --- |
| `cargo nextest run --workspace` | 7592/7592 PASS, 9 existing skips, 467.685s | `/tmp/ares-pi-tower-checkpoint-workspace.log` |
| `cargo clippy` | exit 0 (253 warnings), 33.99s | `/tmp/ares-pi-tower-checkpoint-clippy.log` |
| `cargo fmt --check` | exit 0 | `/tmp/ares-pi-tower-checkpoint-fmt.log` |
| `cargo build -p ares-core -p ares-wasm --target wasm32-unknown-unknown` | exit 0, 1m12s | `/tmp/ares-pi-tower-checkpoint-wasm.log` |

Every Cargo command uses `CARGO_TARGET_DIR=/tmp/ares-pi-tower-target`.
Summary: `/tmp/ares-pi-tower-checkpoint-validation.txt`. The ordinary workspace
run is **not** a substitute for the explicit live runs above. Only documentation
is changed after these checks to record their outcomes. Branch checkpoint
publication is separately authorized; main integration remains forbidden while
these complete-output REDs are open.

Before main integration: repair and re-review the open source-owned cases,
retain full official/seeded output, rerun workspace/lint/fmt/WASM on the rebased
commit, and obey fast-forward-only publication. Checkpoint branch push does
not imply permission to publish a known-RED implementation to main.

## Resolution after checkpoint

The numeric matrix is GREEN in the official CPU0 run
`79077228-74ad-4072-a550-1873824ecee7` (13 cases, 14.104s;
`/tmp/ares-pi-tower-pa-numeric-fix-official.log`). Hexadecimal prefixes now
follow `std::stod`, retaining guard/sticky bits for binary rounding. The PA
writer uses four significant digits and chooses notation from the rounded
exponent without stripping integer zeroes.

Official run `3217362c-c8f4-4807-96b5-964bd146f50c` passes all three tests
(16 full-output cases: constant + 13 model cases + both two-tool activation
combinations), 16.835s; `/tmp/ares-pi-tower-pa-tools-fix-official.log`.
Multi-tool residuals were five comment-forwarding omissions at upstream
`GCodeWriter::{set_junction_deviation,_spiral_travel_to_z,travel_to_xyz}` and
`GCode::{extrude_loop,_extrude}`. Repair only their existing emission sites;
no movement, comparator, reset scheduling or activation behavior is relaxed.
Existing private travel test callers only receive the newly forwarded empty
comment argument; no assertions or tests are removed.

Final validation supersedes the checkpoint RED status above. The source snapshot
is frozen after hoisting the arc comment outside the segment loop and removing
a needless borrow, eliminating all newly introduced Clippy warnings.

A first final workspace build failed in `rust-lld` with undefined hidden
symbols (`/tmp/ares-pi-tower-pa-final-workspace.log`). The older overlapping
validation runs were stopped, and only this target's ares-core incremental
caches were moved to
`/tmp/ares-pi-tower-core-incremental-quarantine.wbp1BI`. With **no source change**,
the fresh build and entire sequential gate chain pass (session 60, exit 0).
Interrupted `ready`/`release` runs are not completion evidence.

| Final gate | Result | Log |
| --- | --- | --- |
| Workspace Nextest | 7592 PASS, 9 existing skips, 322.963s | `/tmp/ares-pi-tower-pa-rebuilt-workspace.log` |
| Original official CPU0 | 3 tests / all 16 complete-output cases PASS, 36.788s | `/tmp/ares-pi-tower-pa-rebuilt-official.log` |
| Seeded reference, seed 0 / CPU0 | 3 tests / all 16 complete-output cases PASS, 72.550s | `/tmp/ares-pi-tower-pa-rebuilt-seeded.log` |
| `cargo clippy` | exit 0, 253 baseline warnings; no new warnings | `/tmp/ares-pi-tower-pa-rebuilt-clippy.log` |
| `cargo fmt --check` | exit 0 | `/tmp/ares-pi-tower-pa-rebuilt-fmt.log` |
| Core + adapter WASM build | exit 0, 20.06s | `/tmp/ares-pi-tower-pa-rebuilt-wasm.log` |

Workspace run: `e2020d71-297a-4c0a-b3f6-ef0f5a798724`;
official run: `1e6ec851-7bc1-44e3-9589-8e0ca9ba0834`;
seeded run: `a4bcdca0-1661-466a-bd5f-64dd4f04ccdf`.
Paired streams are in separate `pa-rebuilt-official` and `pa-rebuilt-seeded`
artifact directories under `/tmp/ares-pi-tower-`.
These are bounded, no-tower cases, not exhaustive calibration-input or
printer/option coverage. No comparator changes, ignored tests or new rejection
guards were added.

Publication state: ready for feature-branch push and coordinator review, **not
approved for main**. Latest reported main is `7ac55555`; after the coordinator's
matching `APPROVED`, fetch/rebase the then-current main and rerun gates. Only
documentation changed after the final code checks to record these outcomes.

## Next-slice handoff (not part of Adaptive PA)

After this slice is approved and integrated, reproduce the coordinator's stock
BBL H2S / P1P / P1S non-0.4-nozzle discrepancy: after the first timelapse block
and first object travel, upstream emits two `G1 Z.1` lines before unretraction;
Ares emits one. Evidence:
`/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-AHpvpi`, byte 44919 / line 1090.
Investigate upstream `GCode::process_layer` timelapse and
`GCodeWriter::travel_to_xyz` / Z-restatement state against the existing Rust
layer/motion emit sites before changing behavior. This is an unverified
handoff, not a diagnosed PA bug. Schedule it with smooth Type2 tower work,
then finish the bridge-direction audit. No implementation of that next slice
will begin while this branch awaits coordinator approval.
