# Initialized concentric parameter implementation plan

Base: 046a1841. Decisions: ARD-0069.

1. Reproduce full filled-output failure with current seed-only reference;
   separate disabled/enabled fuzzy cases at the public byte seam.
2. Build an isolated original with only the two confirmed field repairs added
   to the retained seed patch. Verify source/build identity and self-replay.
3. Observe complete-output RED against that original before changing Rust;
   replace only the empirical min-length-factor split if the evidence confirms it.
4. Recheck the original filled failure and regression cases; keep untouched
   official controls and strict generator-only comparison, without new filters.
5. Full checks, independent six-axis review/repair, commit/push together.

Status: initialized reference built and full-output diagnostics recorded;
the experimental Rust factor change was restored to the base implementation
because no sensitive complete-output GREEN witness is established yet.

- Reference source `/tmp/ares-orca-seed-init-source.5PqJrV`, upstream 8500fcdcc,
  changes only the retained seed patch and the two constructor initializations.
  Both forward and reverse initialization-patch checks pass. Official source
  `/home/indexyz/ares/OrcaSlicer` remains clean.
- Build derivation `/nix/store/abjy7glgvpizgjfa8qja24c7bb38q2fw-orca-slicer-seed-replay-2.4.2.drv`,
  successful process 38016, log `/tmp/ares-orca-seed-init-build.log`.
  Output `/nix/store/ay9g8smyvrzx2a105dxs4hhw35k4k116-orca-slicer-seed-replay-2.4.2`.
  Wrapper SHA256 `484a02a690dccab22cffa00f591091e7e6c0d538ff0a445fa1e516311c837d2d`;
  wrapped ELF SHA256 `64da5246c1fb9a89cb461ad7f40a177a00b092c2e4a4cc1f85b84756aac50855`;
  ELF build ID `8c11ab82a29cee5a3e54ed35aed051bc8ce9f83f`.
- Four cube controls (fuzzy none/external, shells 3/3 and 7/5) PASS with both
  seed-only and initialized reference. They are not sensitive RED witnesses.
  Logs `/tmp/ares-concentric-seed-only-filled.log` and
  `/tmp/ares-concentric-initialized-red.log` retain fresh complete paired artifacts.
- Previously divergent filled project from
  `/tmp/ares-seeded-continuous-parity.HG2UgV/case-IPyzMi/input.3mf`
  matches initialized reference without changing Rust. Two independent native
  runs differ only in the generator timestamp, retained in
  `/tmp/ares-concentric-retained-native.rtcWJF` and
  `/tmp/ares-concentric-retained-native-second.uF9JmQ`.
- KSR initialized-reference full replay remains RED. Original internal factor
  zero produces model time 1h43m52s versus reference 1h43m53s. Experimental
  factor 0.5 fixes that time, but full output still fails on configuration
  comments, object identity, M73 positions and multi-filament summary fields.
  Before `/tmp/ares-concentric-initialized-replay-results.yP2nQp`;
  after `/tmp/ares-concentric-initialized-factor-valid.qG3JSQ`.
  The first factor replay command selected a log instead of a directory and
  failed with `Not a directory`; only the corrected explicit-path run is
  valid behavioral evidence.
- Overhang full-output probes also remain RED with factor 0.5, both with M73
  enabled and with explicit input `disable_m73=1`. The latter still exposes
  bridge geometry/travel and cooling-speed differences; it is not accepted
  as a workaround. Artifacts `/tmp/ares-concentric-overhang-factor.vEfXax`
  and `/tmp/ares-concentric-overhang-no-m73.immqNB` remain intact.
- Independent design review accepts the source/default convention but requires
  actual concentric activation and a sensitive complete-output RED/GREEN.
  Narrow neck geometry with concentric top/bottom also PASS with both factors
  (`/tmp/ares-concentric-neck-factor.log`,
  `/tmp/ares-concentric-neck-original-factor.log`); it is a control, not a
  sensitive regression. The old kernel test requiring the short odd branch
  fails under factor 0.5 (38/39 concentric tests pass). It must not be updated
  blindly: obtain independently initialized source evidence before changing
  that output contract. Next action is isolate the KSR short-branch geometry
  through a public project fixture without masking output differences.

Further public geometry evidence:

- Original CLI `--export-stls` succeeds for the checked-in KSR project,
  producing a 611784-byte STL. The public test exports it again with Ender-3
  profiles, without rewriting output bytes or changing the comparator.
  Old internal factor yields RED at initial M73 R289 versus reference R288;
  full paired artifact
  `/tmp/ares-concentric-ksr-ender-red-rebuilt.NKkogl/case-atLu6C`.
- The same exported input/reference replayed under experimental factor 0.5
  is still RED at the same initial M73 instruction, retaining 6706354 actual
  versus 6720620 expected normalized bytes. Artifact
  `/tmp/ares-concentric-ksr-ender-factor-results.hOjrra/case-bHNKgX`,
  log `/tmp/ares-concentric-ksr-ender-factor.log`.
  This does not establish a completed short-line fix; production was restored.
- A separate source defect explains an object-label difference: `Print.hpp:582`
  declares `PrintObject::m_id` without initialization, and
  `PrintObject.cpp:85-116` leaves it unset. Its only object-ID setter is the
  conditional object-exclusion definition in `GCode.cpp:8096`, while
  `GCode.cpp:5348-5351,5471-5475` reads it for ordinary enabled labels.
  `exclude_object=0,gcode_label_objects=1` appears in both exported controls;
  cube reference reports ID 0, but KSR STL reports 13994935845302207070 and
  original KSR reports 3766686319534. This is not RNG noise. Initializing this
  additional field was deferred to the separately identified build/source slice
  recorded below; the binary used by the full option scan remained unchanged.
  A second fresh native process with the exact same KSR Ender input, seed 0
  and CPU 0 reports ID 13957499370569321568 instead of
  13994935845302207070 (process 16830 exits successfully). This confirms
  cross-process object-label instability even in the two-field replay build;
  full native output is retained under the `ares-concentric-ksr-ender-native-repeat`
  external artifact directory. Source initialization alone, not output
  normalization, is the candidate next reference repair.
- A zero-test run caused by shared target artifacts from the other worktree
  returned nextest exit 4 and is explicitly excluded as evidence. Touching
  this worktree's test entrypoint and rebuilding restored the actual test;
  the authoritative RED log is the `-rebuilt` run above.
- Latest immutable 046a1841 all-option scan first exited before slicing because
  its worktree lacked profiles. After provisioning a read-only OrcaSlicer
  symlink, a fresh run completed as process 68575 (exit 100), log
  `/tmp/ares-options-046a1841-seed-init-provisioned.log`, artifacts
  `/tmp/ares-options-046a1841-seed-init-provisioned.8d7rT7`.
  It uses the retained two-field initialized reference and original committed
  Ares implementation, not experimental code. Final coverage is recorded below.

Object-ID prerequisite implementation:

- Independent reviewer confirms the missing field initialization, sole
  conditional setter and unconditional label readers; `ObjectBase` has a
  different same-named member and does not initialize this one.
- New upstream source `/tmp/ares-orca-seed-init-id-source.kjDHGl` at 8500fcdcc
  changes exactly four files: prior seed patch, prior two fill constructors,
  and `Print.hpp` member initializer `m_id{ 0 }`. The new zero-context
  `scripts/orca-replay/object-id-initialization.patch` passes forward check
  on untouched official source and reverse check on the new source; all
  three patch assets reverse-check on the new source, and its diff is clean.
- Build completes with process 22531 exit zero, log `/tmp/ares-orca-seed-init-id-build.log`,
  derivation `/nix/store/4n1w0b6jgnfcwb6m9jfssndribyyg3v6-orca-slicer-seed-replay-2.4.2.drv`.
  Output `/nix/store/jqfsg42f6xzq3krnf5scm6w4g1z4kz4c-orca-slicer-seed-replay-2.4.2`.
  Wrapper SHA256 `3d05d4ccab740ca5e13aa94d0d00867a75f867ba2159b9f5b024dc0fcaa3bfaa`;
  ELF SHA256 `8f0cf69dd9494ccfe53bf6749d6b685ccb5a0dd0df206694ce50d421007cc03d`;
  ELF build ID `d1d964043a0097f4c33db3fa1bbd5991477aeef9`.
- Public native harness regression
  `orca_parity_initialized_reference_repeats_complete_ksr_output` uses the
  checked-in KSR input in two fresh processes; retains input, reference
  selection metadata, both full outputs and native command evidence.
  It compares the entire streams through the existing generator normalizer
  and separately requires single-object unassigned ID zero. It does not
  claim Ares parity, all-device coverage or multi-object uniqueness.
- Observed RED against previous two-field reference: 44.732 seconds,
  first difference line 1099, object ID 13881826882987006752 versus
  4711471787139923968. Log `/tmp/ares-object-id-old-reference-red.log`,
  full artifacts `/tmp/ares-object-id-old-reference-red.xUW8Sc/initialized-object-id-epFxGL`.
  New-build self-replay passes the post-review activation/all-label assertions
  and whole-stream comparison in 42.130s; process 14719 exits zero, log
  `/tmp/ares-object-id-initialized-reference.log`, artifacts
  `/tmp/ares-object-id-initialized-reference.rOx4mW`.
- Pre-review requested fixture activation locks to prevent a future legal
  object-exclusion setter from hiding the missing-initializer path. The test
  now reads public project settings and requires `exclude_object=0` and
  `gcode_label_objects=1`; it checks every ordinary start/stop object label
  for ID zero and asserts labels are actually present. The fixed test has
  new-reference GREEN. Independent six-axis review approves this reference
  prerequisite only: both complete outputs contain 6321730 bytes and all
  920 ordinary object labels satisfy the zero-ID convention. The Rust
  concentric consumer and global parity goal remain incomplete.
- Initial and post-review fmt/workspace all-target Clippy checks pass
  (existing warnings), latest logs
  `/tmp/ares-reference-initialization-fmt-check-updated.log` and
  `/tmp/ares-reference-initialization-clippy-updated.log`, processes 99477 and
  14697 exit zero. Full workspace process 37810 passes 7451 tests, 9 skipped,
  in 461.994s, but compiled before the label-activation review fix.
  Final exact prerequisite module tree passes 7450 tests, 9 skipped, in
  262.224s (process 13359 exit zero), log
  `/tmp/ares-reference-prerequisite-final-workspace.log`. Final fmt and
  workspace all-target Clippy pass (processes 8084 and 36116 exit zero), logs
  `/tmp/ares-reference-prerequisite-final-fmt.log` and
  `/tmp/ares-reference-prerequisite-final-clippy.log`. Core library and adapter
  source trees are unchanged from 046a1841; its Tier-1 compiler evidence is
  retained, not claimed as a newly executed platform matrix.
  Offline Rust checks do not establish native reference GREEN.

Latest immutable complete-domain scan 046a1841 is terminal (process 68575,
exit 100): 386 PASS, 262 INCOMPLETE, 2 FAIL out of 650 domains;
1661 generated, 1517 strictly compared, 1 rejected, 2740.606s. Reference is
the unchanged two-fill-field build, not the newer ID-initialized build.
`enable_support=false` complete-output divergence and SLA request applied
as FFF remain failures. Full summary and all inputs/outputs are retained in
`/tmp/ares-options-046a1841-seed-init-provisioned.8d7rT7`.

Captured 55-vertex short-branch domain extruded to a 0.6mm diagnostic prism
(216 triangles) is retained outside the repository in
`/tmp/ares-concentric-prism-fixture.UHUYJ9/short-branch.stl`. The temporary
isolated diagnostic test references this path and is excluded from the
committed module tree. Old-factor complete comparison is RED at line 273: reference
width 0.394158 versus actual 0.394156; paired streams
`/tmp/ares-concentric-prism-old-factor.e5kmYO/case-Qor4sH`. No consumer fix
or sensitive public GREEN is established. Remove temporary diagnostic paths
from the committed module tree, and preserve this failure evidence.
