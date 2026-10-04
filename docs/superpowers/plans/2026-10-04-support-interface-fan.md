# Support-interface fan complete-output rewrite plan

Source owner, Rust destinations, included/deferred behavior and mandatory
semantics: `docs/architecture/support-interface-fan.md` (this is the slice
specification). Public test seam is the existing approved saved3MF →
`slice_project` → complete native G-code comparison, not internal fan state.

1. Reproduce the unchanged Option domain min/max/seeded on latest main with
   both seededCPU0 and officialCPU0. Preserve the baseline manifests.
2. Add public controls for -1/0/45, overhang fan enable=false, and closed
   first1000 layers, with saved-setting readback, positive native support
   interface extrusion and fan witnesses. Require complete output, retaining
   all failures until GREEN.
3. Port the missing support-interface fan option/state/START-END consumer.
   Extend only the existing source-owned fan transport, with no AdaptivePA,
   timelapse, support geometry or ironing changes. Verify role transitions,
   fixed speed below baseline, first-layer gate and forced-resume semantics.
4. Re-run the unchanged Option domain, new controls and prior finite cooling
   controls against both references. Then workspace Nextest, Clippy, fmt,
   core+WASM adapter compile. Review the exact diff locally for probes,
   comparator changes and unintended cross-owner edits.
5. Commit/push the feature branch, print `REVIEW-REQUEST <hash> <summary>` in
   the terminal and stop. After coordinator `APPROVED <hash>` while main is
   unfrozen, fetch/rebase current origin/main, repeat full gates and fast-forward
   push main. During any renewed freeze, wait for explicit unfreeze notification.

## Evidence / hypotheses

Frozen9a7b46fc full Option sweep: support_material_interface_fan_speed max
and seeded fail; min(-1) passes. First difference byte136356/line5875 is a
missing M106 S255 before native interface extrusion. This old evidence alone
is not a RED on current main. Latest-main repeat session44 uses56e4da53,
`/tmp/ares-pi-interface-fan-56e4da53.gWl35j/{seeded,official}` with matching
logs and exits.txt. Seeded exits100:3 comparisons, max/seeded RED with the
same missing M106 (byte136356/line5875). Official exits100 with3 ARTIFACT_ERROR
and0 comparisons: the overlapping test-control build replaced the executable
before `build_identity` could reopen it. This is an infrastructure failure,
not an official geometry RED. Do not overlap builds with live reference tests
sharing that executable; official output still needs a fresh serial run.

Ranked predictions before production edits:
1. Missing role producer: motion/fan.rs emits only overhang/internal bridge
   requests and options assembly does not consume interface fan speed. Adding
   the source-owned support request should supply missing M106 without any XY
   or E changes.
2. Incorrect fixed/ramped consumer: interface45 below baseline100 must emit
   fixed PWM114 (floor(255.5 *45/100)), not255 or a ramped override; exercise independently of
   overhang enablement and retain full byte comparison.
3. First-layer request lifecycle: closed-layer START must remain inactive and
   its END must not spuriously restore a fan. A closed1000-layer public control
   should preserve ordinary disabled cooling output exactly.

No temporary DEBUG/println instrumentation is planned. Tests must demonstrate
native interface extrusion so footer-only option changes cannot satisfy them.

## Public RED and first implementation

New controls first exited100 at a wrong handwritten PWM witness (expected115,
native114). Corrected the witness using GCodeWriter's floor(255.5*45/100),
without altering comparator or reference. The corrected controls run55 exits100
at `/tmp/ares-pi-interface-fan-controls-red-pwm.log`: all6 native role witnesses
pass, disabled control passes,5 complete comparisons remain RED. In addition
to missing fan overrides, closed1000 reveals native's literal inactive
`;_SUPP_INTERFACE_FAN_END` output (apply_layer_cooldown:875 falls through its
ordinary-line branch). The specification now explicitly preserves this
source behavior rather than removing the comment from expected output.

First implementation extends typed filament fan options, persistent generator
layer-marker state and the existing deferred cooling requests. Interface
requests are kept separately from the existing higher-priority requests so
START/END can restore the correct fan. It preserves inactive END comments,
fixed speed and closed-layer suppression without modifying other owners.
Focused candidate run57 finished serial seeded→official:3/3 functions pass
on each reference,48.059s and35.807s respectively. Each actually compares12
complete outputs: the unchanged3-value Option sweep,6 controls and3 previous
interlaced-cooling/equalizer controls. Root
`/tmp/ares-pi-interface-fan-green.1AZASG`; both exits0 and all24 manifests PASS.
Native interface extrusion/fan witnesses pass for all new controls. The
previous official ARTIFACT_ERROR run is not used as acceptance evidence.

Gate session60 (`/tmp/ares-pi-interface-fan-gates.MYanka`) finished exit0
for workspace Nextest (7593 PASS,9 skipped,3756.718s,2 threads), all-target
Clippy, fmt and WASM compile. Clippy still reported added nesting warnings in
the new support branch, so this snapshot is not final acceptance. No reference
environment was set for workspace tests; live-output evidence remains separate.

Local readback flattened inactive-END handling and stored the higher-priority
role's requested speed as `Option<u8>` instead of a Boolean. Both source
requests now select one target through `role_speed.or(support_speed)`; this
removes repeated nested target selection without inventing new fan policy.
The test witness parser also uses `find_map` rather than filter_map/next.
Final preflight session68: fmt0, all-target Clippy0,247 core-lib /263 core-test /
21 parity warnings. The only remaining warnings in role_fans.rs are its prior
auxiliary/resume branches; no new fan/test warning remains.

Frozen final session70 uses `/tmp/ares-pi-interface-fan-final.A7GGZF`:
archive the test executable first, run both reference suites serially from
that archive, then workspace Nextest (4 threads), Clippy, fmt and WASM.
Build jobs remain2 and live references/Ares remain CPU0. The archive prevents
future test builds from invalidating live executable-identity evidence.
All terminal results are now verified: seeded and official each3/3 functions,
12/12 complete-output artifacts PASS (49.178s /37.806s); workspace7593/7593
PASS,9 pre-existing skips,1743.646s; Clippy, fmt and WASM compile exit0.
Clippy retains247 core-lib /263 core-test /21 parity warnings, with no new
support-interface warning. WASM verification is compile-only, not browser
runtime validation. The final tracked Rust diff SHA256
`59e0da353289bdbe0d56ffad6878519ec3c7a9a8d5140eabdb4296c139057eba`
and new test SHA256
`af1bdbbc6595df41e67dbacdd1a220432c193271215694aec36c295cca53963f`
match the frozen provenance.txt after all gates. Changed Rust files remain
below400LOC (maximum394), and the diff has no temporary diagnostic probes,
comparator changes, unsupported guards or ignored tests.

The coordinator has unfrozen main and announced b64b1d90. This feature slice
is still based on56e4da53 and is ready for branch commit/push and coordinator
review; it is NOT yet approved or merged. Only after matching approval will
it rebase the latest main and repeat the required full gates. Earlier BBL,
Tree and SLA/domain residuals stay assigned/open as recorded in the matrix
continuation plan; this finite fan repair does not close the global matrix.
