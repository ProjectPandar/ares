# ARD — Native oracle availability policy (slice A)

Status: accepted — slice A, test infrastructure only.
Audit base commit: `7fc1c69e3cf860dbfa9f93bbeb539f2acc8da127`.
Slice A lands before slice B starts.

## Decision and boundary

This is test-infrastructure repair, not a new slicer milestone or an Ares
pipeline feature. Existing source-cited libslic3r/libvgcode parity assertions
remain unchanged. No production `ares-core/src` or `ares-cli/src` changes.
No geometry, slicing, output normalization, golden assertion or cache-key change.
No committed Native output is deleted/replaced in A. That is separately specified B.

Add `ARES_REQUIRE_ORCA=1`. Only the exact value `1` selects mandatory Native.
A Native-dependent comparison that cannot obtain its required reference
configuration must fail, with the exact missing environment variable named.
No `Ok(None)` / early-return PASS for missing Native under this mode.
Unset/other values permit local offline development on all Tier1 platforms,
but every omitted Native comparison must print a stable `SKIPPED:` marker,
its dependency/reason and caller location. Local Ares assertions in a mixed
local/Native test may still pass; the message explicitly says the Native
comparison did NOT run. That local pass is not counted as Native parity evidence.
Configured-but-invalid executable/resource/artifact paths remain failures, not
optional skips. Native launch/parse/comparison failures never become skips.

Rust libtest has no runtime ignored-test return state. We must NOT claim a
runtime `return` changes nextest's PASS into its native SKIP status. The supported
contract is explicit captured `SKIPPED:` output (task allows this), displayed by
nextest `success-output = "immediate"` scoped to Native-comparison binaries and
excluding synthetic launcher/cache probes. Unrelated tests retain normal output
policy. Do not force `process::exit`, panic in
optional mode, or add a different test framework. Cargo/libtest users see markers
with `--nocapture`; supported repository test entrypoint remains nextest.

## Current mechanisms and holes

`crates/ares-cli/tests/orca_parity/runner.rs:208-265`:
`OrcaRunner::from_env` resolves ARES_ORCA_BIN or the repository wrapper; missing
artifact root returns None unless sweep/cluster/options explicitly demand it.
Thus repository wrapper presence does NOT prove Native availability, and absence
of artifact configuration silently returns before actual Native identity checks.

`runner/cache_identity.rs:65-94` already requires ARES_ORCA_NATIVE_BIN and
ARES_ORCA_RESOURCES_DIR and binds executable/resource hashes into producer-v2.
Those checks are currently lazy, reached only by Native-producing stages.
A must check missing availability before an offline return or cached reference
can hide it. Preserve producer-v2 hash semantics and existing lazy full hashing;
do not eagerly rehash the entire resources tree on every test's initialization.

Audit inventory at base:144 files mention OrcaRunner/Runner::from_env; the direct
`let Some(runner) ... else { return; }` pattern finds92 sites in72 files (not a
claim to capture every indirect None-return).56 files have direct ARES_PARITY
selection flags before runner initialization. All are listed in the companion
inventory, including already-failing callers and harness-only callers.
Separate oracle paths exist in gcode_moves_oracle, VG1, color-change, motion
attributes and the optional role-map live branch; changing runner.rs alone is
therefore insufficient. Frozen Native consumers are B's migration inventory,
not proof that live Native was exercised in A.

## Availability versus test selection

Keep existing opt-in sweep/cluster/stock/honeycomb/seed-only controls: A does not
force every experimental/sweep mode to execute. Otherwise official mode would
incorrectly run seed-only tests, and the requested same-environment result
comparison would change its test-selection meaning.

A disabled Native-specific selector prints `SKIPPED: <flag> not enabled; Native
comparison not executed`. Mandatory mode also checks availability BEFORE the
disabled-selector return, so absent Native cannot exploit an earlier gate.
Seed-specific prerequisites (ORCA_REPLAY_BIN/SEED/CPU) are required only when the
existing seeded comparison is selected; official mode does not require replay.
Selection controls are not a fallback oracle and are never represented as
successful Native comparisons. General diagnostic child-entry tests without
CACHE_CONTRACT_ROOT / ARES_RUNNER_PATH_WORK / ARES_STRICT_REFERENCE_CHILD are
not Native comparisons: their parent tests execute them with synthetic helpers.
Do not classify every `return;` in the repository as a Native skip.

## Intended implementation seams

1. Small test-only shared availability/reporting module under tests support;
   reused where policy is identical, not copies per binary. It reads environment
   without mutating process-global state and emits caller-qualified SKIPPED.
   Keep oracle-specific requirements at each caller, not a global rule requiring
   every procgcode/gdb binary for unrelated slicer tests.
2. `OrcaRunner::from_env`: required mode fails on missing Native executable,
   Native resources or artifact root, names the absent key, and keeps the
   existing Initialization/Io StageError boundary and configured-path errors.
   Optional absence emits SKIPPED before returning None. Existing caller
   `.unwrap()`/panic propagation remains failing; no bulk rewrite of144 callers.
   Shared availability preflight can be called by selection gates without
   constructing a work directory or producing/caching a reference.
3. Replace the56 direct selection gates with a test-only gate that preserves
   flag selection, mandatory availability checking and visible disabled markers.
   Other opt-in forms found during implementation must receive the same policy;
   add to inventory before claiming audit closure. No blanket override of flags.
4. Independent procgcode-only paths obtain their own named executable through
   the shared policy: absence required→FAIL, optional→SKIPPED. Preserve exact
   stable-options/raw distinction; never substitute one for the other.
5. Optional live branches within mixed frozen/local tests must disclose omitted
   comparison in optional mode and fail on every missing prerequisite in
   mandatory mode. Required dynamics needs ARES_PROCGCODE_DYNAMICS_BIN;
   required role-map needs ARES_ROLE_ORACLE_LIVE and explicit ARES_GDB_BIN.
   Invoke that GDB directly; implicit nix-shell tool resolution is not an
   availability fallback. Preserve the existing reader's pinned raw Native ELF,
   SHA and map layout and all assertions. Core subprocess removal remains B.
6. Diagnostic runner path/initialization tests use explicit per-child environment
   isolation: synthetic non-Native launch probes remove inherited required mode,
   while policy probes set it explicitly. No global env::set_var / race-prone
   environment test mutation. No blanket exemption of real reference cases.
7. `.config/nextest.toml`: success-output="immediate" scoped to Native comparison
   tests, not unrelated tests or synthetic probe bodies; keep num-cpus unchanged.
   This exposes offline markers. Native-comparison assertions emit no new
   success logs; explicit user CLI overrides may suppress output, but official
   gates must not hide skip messages with `never`.

## Files and migration methods

The companion `ard-native-required-a-inventory.md` is the exhaustive audited
per-file runner/selector list at the cited base, not a declaration that all144
files will be edited. Most inherit runner policy centrally; selection-gated
files need their gate replaced, with oracle assertions untouched.

| File | A change / classification |
| --- | --- |
| crates/ares-cli/tests/orca_parity/runner.rs | central missing-env fail/visible-skip boundary; module inclusion |
| crates/ares-cli/tests/orca_parity/runner/cache_identity.rs | audit identity contract; preserve keys/hashing; retain named path causes if needed |
| crates/ares-cli/tests/orca_parity/runner/initialization_tests.rs | subprocess policy RED controls; preserve existing configured-artifact errors |
| crates/ares-cli/tests/orca_parity/runner/path_tests.rs | isolate synthetic non-Native child from inherited required mode |
| crates/ares-cli/tests/orca_parity/runner/cache_contract_tests.rs | ensure deliberate Native fixture metadata remains provisioned in child probes |
| crates/ares-cli/tests/orca_parity/runner/strict_reference_tests.rs | synthetic retry contract, not a live Native oracle; preserve all ordering assertions |
| crates/ares-cli/tests/orca_parity.rs | test-only policy inclusion and environment contract docs |
| crates/ares-cli/tests/gcode_moves_oracle.rs | oracle_bin missing ARES_PROCGCODE_BIN policy, assertions unchanged |
| crates/ares-cli/tests/orca_parity/vg1_flush_state.rs | is_some procgcode gates report skip/fail instead of no comparison |
| crates/ares-cli/tests/orca_parity/vg1_flush_state/state_oracle.rs | consumer audit; retain exact Native state/data assertions |
| crates/ares-cli/tests/orca_parity/filament_volume_oracle.rs | consumer audit; retain exact Native volumes assertions |
| crates/ares-core/tests/processed_color_change/native.rs | missing ARES_PROCGCODE_BIN required failure or visible optional skip; retain separate STABLE_BIN requirement |
| crates/ares-core/tests/processed_color_change.rs | test-only policy inclusion as needed, no core API changes |
| crates/ares-core/tests/processed_motion_attributes.rs | optional dynamics live branch is explicit; require named dynamics dependency for mandatory execution; frozen files remain for B |
| crates/ares-core/tests/processed_role_filament_usage.rs | live-flag/Native/gdb environment visibility; provision or report blocker, no role-map assertion deletion |
| crates/ares-cli/tests/orca_parity/replay.rs | explicitly selected legacy replay, not fresh Native evidence; missing replay selector visible SKIPPED; do not require a legacy replay inventory for unrelated mandatory live suite |
| .config/nextest.toml | expose captured optional skips only in Native-comparison test scopes |
| tests/support/native_environment.rs | shared test-only required/value/binary/visible-skip policy |
| crates/ares-cli/tests/orca_parity/runner/environment.rs | bounded availability preflight and reference selection gates |
| crates/ares-cli/tests/orca_parity/runner/native_availability_tests.rs | actual process-boundary initialization/selector/procgcode probes |
| crates/ares-core/tests/support/native_policy_checks.rs | isolated child execution for mandatory mixed Native checks; migrate external core test work in B |

Tests reading frozen Native gcode/maps and hand-written G-code inputs remain
unchanged in A except optional live-check availability described above. B must
have a separate exhaustive tracked-artifact inventory and consumer migration
spec, moving core external-oracle work to CLI E2E without weakening assertions.

## Failure modes / accepted TDD seam

Seam: invoke the real test executable in a child process
with isolated env and an exact existing test selector; inspect child exit and
captured diagnostics. The parent test never mutates env used by parallel tests.
This verifies externally observable test-policy behavior, not private helpers.

Before implementation, add independent RED checks for:
- required1 + unset ARES_ORCA_NATIVE_BIN => child fails, names exact key;
- required1 + missing resources / missing artifact root => named failures;
- no required flag + genuinely absent reference configuration => success only
  with captured SKIPPED/dependency/nonexecuted message, not parity proof;
- configured invalid bin/resource/root still fails, never SKIPPED;
- missing Native cannot hide behind a disabled Native selector;
- independent procgcode test has identical required/optional semantics;
- run a optional skip through nextest and retain output proving visibility;
- cache-hit/provisioned path cannot silently pass required mode with Native env
  removed; existing cache contract assertions remain unchanged.

No fake Native fixture may count as behavioral Native parity RED. These are
intentionally environment-policy E2Es; fake helpers can test launcher/path/cache
contracts only and must be labeled as such. Preserve independent RED commit,
then minimal GREEN, then repeat exact process probes. New tests are written
before code; do not add retrospective unit tests.

## Parity control gate environment

The parity control-set gate adds `ARES_REQUIRE_ORCA=1` to its final
`exec env` block, alongside CARGO_TARGET_DIR/CARGO_INCREMENTAL; both official
and seeded inherit it. Preserve all reference-selector variables and identities.
No cache/input/Native comparison flags or normalization changes.

Workspace gates: add `ARES_REQUIRE_ORCA=1` to env before
`cargo nextest run --workspace`, while also supplying the full
Native/procgcode resources/identity/reference-selector env already used by the
corresponding controls. Its additional shared environment must include
ARES_PROCGCODE_DYNAMICS_BIN, ARES_ROLE_ORACLE_LIVE=1 and ARES_GDB_BIN.
Mandatory mixed checks have no exemptions. Keep the role reader's existing
pinned raw ELF/SHA and runtime libraries; it must not reinterpret the selected
seeded/official ELF using different ABI assumptions. Per-test role input/output
remains locally supplied, not a global resource substitution.
Preserve the actual replay seed 0 single-core policy and Native ELF identities.
Optional visibility demo: no required flag, nextest success-output immediate.
Mandatory absence demo: `env -u ARES_ORCA_NATIVE_BIN ARES_REQUIRE_ORCA=1 ...`
with the other prerequisites retained and real selected test, expect exit nonzero
and exact missing-key message. Do not turn expected failure into suite pass proof.

## Exit gates and sequencing

No new production logic. clippy workspace/all-target-Dwarnings, fmt, actual CLI
wasm32-Dwarnings and workspace nextest mandatory-mode all-green (9 intentional
static ignored tests remain ignored unless separately requested).
Fresh same-main base/candidate paired seeded AND official control suites plus
workspace, identical per-test names/outcomes except added policy E2Es. Report
inherited Native ND/setup failures with exact counterpart evidence, not fakePASS.
Raw Ares bytes across common labels unchanged; assertions/Native stream hashes
never relaxed. Record env policies, artifacts, logs and candidate/base hashes.
Result equality is not enough: mandatory-mode absence demonstration must fail,
and optional absence demonstration must visibly declare no parity assertion.

Then land slice A fast-forward only.
Only afterward write slice-B deletion/runtime/cache/gdb migration spec.
Mandatory role/dynamics/gdb prerequisites are part of the gate contract:
availability failures cannot be exempted because a frozen golden is present.
