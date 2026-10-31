# SAFC concentric Native authority and short-line initialization

Status: stage2 approved; authority selection revised by the coordinator to
existing common KSR initialized paths after the additional four-run preflight.
No Rust or production change. Worker6 owns this bounded
Native-determinism investigation. Stock rescan remains HOLD; no full run is
part of this slice. Source pin: OrcaSlicer8500fcdcc.

## Retained evidence and bounded diagnosis

Existing actual runs of
`safc_coverage::safc_coupled_models_preserve_whole_output_and_segment_consumers`
contain the same on-concentric input ZIP members but different official Native
geometry. The current audit is read-only; it is not a fresh Native execution.

| Retained case | Native segments / Top | Ares segments / Top |
| --- | --- | --- |
| worker3-brim-auto-engineering-v7-official-artifacts/case-r42Qim |1617 /44 |1622 /49 |
| worker3-brim-safc-base-c02-v1-official-1-artifacts/case-T8w6q0 |1622 /49 |1622 /49 |
| worker3-brim-safc-base-c02-v1-official-2-artifacts/case-AdJDbR |1622 /49 |1622 /49 |

All three complete Ares outputs have SHA256
0ffd98e52542df26dfb7bc1babcf250012cdce29d2c6fd3d5a98cc052d774f68.
Native first body difference after the banner is line1009: Top49 variant has
`;TYPE:Outer wall`, Top44 variant has `M73 P32 R5`. First motion difference is
line3360 at final layer Z10: Top49 travels to X110.037 Y109.936, Top44 to
X105.955 Y105.955. The Top49 output contains five positive-E center-path
segments at lines3366-3370, width0.633642; Top44 starts directly with the outer
concentric loop. M73 changes are downstream of geometry/time changes, not the
only discrepancy.

Inputs have fuzzy_skin=disabled_fuzzy, seam_position=aligned and no
`random(` in their G-code templates. Raw ZIP packaging differs, but every
member name and byte hash is equal. Do not classify packaging timestamps as
a changed slicer input. Same complete Ares bytes and different complete Native
bytes establish the reported Native fluctuation; they alone do not identify
the precise indeterminate field values in each process.

Evidence: /tmp/worker3-brim-safc-base-cand-adjudication.json and its original
case roots; repeatable read-only /tmp/worker6-safc-nd/audit.py and
retained-audit.json. Native products stay outside the repository.

## Source-owned path and hypotheses

1. Confirmed upstream initialization defect is the leading cause.
   `Fill/Fill.cpp:1289` sets use_arachne for concentric independently of the
   configured classic wall generator; `FillBase.cpp:136-175` dispatches to
   fill_surface_arachne and variable_width. `FillConcentric.cpp:92-102`
   default-constructs WallToolPathsParams and fills six fields, leaving
   min_length_factor and is_top_or_bottom_layer unset. The analogous internal
   constructor is `FillConcentricInternal.cpp:29-39`.
   `Arachne/WallToolPaths.hpp:23-37` has no default member initializers for
   those two fields. `WallToolPaths.cpp:68` copies the supplied parameter object;
   removeSmallLines at684-699 reads both to retain/delete short odd open paths.
   This defect was independently recorded in
   [ARD-0069](ard-0069-initialized-concentric-short-line-parameters.md).
   The missing center path fits that consumer, but this audit has not observed
   these exact SAFC processes' stack bytes. Do not call their field values
   measured, or claim an independently proved TBB data race.
2. Layer scheduling is a possible exposure mechanism for indeterminate stack
   values, not a proposed algorithmic random choice: PrintObject.cpp:698-721
   fills layers via tbb::parallel_for. One pinned CPU does not initialize local
   storage or prove that the library creates only one thread. Prediction:
   initialized fields eliminate the short-line variance on fixed inputs even
   with the existing scheduler; merely pinning a CPU need not eliminate UB.
3. Intentional RNG is weaker for this witness: fuzzy, random seams and random
   templates are inactive, and FillConcentric has no random draw. Other patterns'
   Fill.cpp random joints and lightning are not selected. Do not equate seeded
   success with proof that the root was an RNG draw.
4. Input/producer drift must still be checked using actual runtime receipts.
   The existing input-member equality rejects input content drift for the three
   captures. New preflight must bind the precise wrapper/ELF/resources identity;
   source inspection is not a substitute for those per-run bindings.

SAFC itself is created at GCode.cpp:2548-2549 and changes E at7006/7066/7089/
7214 after paths exist. The compensation flag is not the constructor's missing
field and must not be changed to avoid this case.

## Existing defined authority

Current admitted replay producer is
/nix/store/1k0p5nlbs3zsfdcy08g3y5fb7bcm1qrv-orca-slicer-seed-replay-2.4.2,
ELF SHA25659fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff.
Its existing derivation is72d4hac8qmg5p9m22v2kifbp1hkxjk63; read-only derivation
inspection identifies source g3wj93lyhjsvc32fmgvw0cdbd6191q55. That source sets
min_length_factor=0.5f and is_top_or_bottom_layer=false at standard fill99-100
and internal fill36-37. No derivation was realised or new Native built.

This is initialized replay authority, not untouched official semantics. The
common defaults are source-cited by WallToolPaths.cpp:34-37,56; defective fill
constructors do not define their values. Current Ares
project_slice/fill_entities/concentric/generate.rs:107-112 explicitly uses that
same convention. Do not port undefined bytes to Rust or modify the official
binary. The historical KSR producer jqfsg42 has a different RNG recipe; its
bounded use below was admitted only after fresh concentric output equivalence.
This does not establish interchangeability for patterns that consume RNG.

## Approved bounded correction

Rust destination is test harness only:
option_coverage/safc_coverage.rs plus same-prefix dedicated helper. Keep all
seven requested scenarios, the existing test name, complete output comparator,
role requirements and all coupled consumer assertions. No core, processor,
SAFC formula, input geometry/options, shared cache or production API change.

- Rectilinear scenarios remain on the caller's selected official/seeded Native.
- On/off-concentric form a paired initialized-authority child using the admitted
  jqfsg42 producer, seed0 and existing single-CPU wrapper semantics.
  This is explicit authority selection, never a fallback after comparison fails.
  Parent retains the seven-case inventory and separately reports the selected
  producer per case; official-mode reporting must identify five official cases
  plus two initialized cases, not claim seven untouched official comparisons.
- Reuse common ARES_KSR_INITIALIZED_BIN/NATIVE_BIN/RESOURCES_DIR exactly as
  declared by the control environment. Only the paired child maps them into
  existing ARES_ORCA_BIN/NATIVE_BIN/RESOURCES_DIR and ORCA_REPLAY_BIN;
  ARES_ORCA_BIN selects scripts/orca-seeded-parity.sh, ORCA_REPLAY_SEED=0 and
  ARES_PARITY_SEEDED_REFERENCE=1 are set only in that child. Invoke the same
  OrcaRunner::from_env path; no constructor bypass, process-wide set_var or
  second cache. The official parent keeps ORCA_REPLAY_BIN/SEED unset and retains
  its existing producer/cache identity; seeded parents retain their existing
  1k0p5 selection. Do not promote replay variables into official common refs:
  their presence changes bridge_over_infill_filament.rs:96 and
  narrowed_x2d_whole_file.rs:78, and changes cache_identity.rs:86-91 identity.
  Reuse slice_case's existing producer/request/lock/receipt machinery. Both
  concentric cases use the same producer and original inputs.
- Missing dependencies use tests/support/native_environment: required FAIL;
  optional visible SKIPPED with precise reason. No ignore, silent success,
  synthetic output, frozen output, comparison relaxation or assertions removed.
- Producer components enter existing cache identity and receipt as normal;
  preserve full-file generator-only comparison, segment counts/role/keys/E and
  disabled on/off equality. Do not replace the five lost segments with a count
  tolerance or compare just a polygon/footer.

## Approval-stage diagnostic and RED/GREEN gates

Before consumer edits, use a fixed finite same-input preflight: two official
runs and two initialized runs per on/off case, each fresh cache/output directory, sequential at
load<60/jobs2/test-threads2, keeping every result. Use the actual on-concentric
input plus off control. No retries until PASS, no stress/full run and no Native
rebuild. Official repeats may coincide; that cannot erase the retained ND.
If existing Native debug symbols permit, inspect the two parameters and the
small central path at removeSmallLines with explicit debugger tooling, recording
thread identity and values outside git. Absence of symbols is reported, not a
reason to rebuild Orca. Do not invent per-run field values from source alone.

Report initialized self-replay and complete Ares comparison before proceeding.
If initialized output also varies or differs from Ares, stop with the real RED;
then authority routing is not yet justified. Preserve original official failing
captures as oracle-UB evidence, not a newly credited official PASS.

After approved preflight: Native-backed authority-policy RED, then harness
GREEN. Provide sorted assertion-text/count diff demonstrating unchanged seven
scenario and consumer contracts. Required cold/hot runs in both control modes,
then the entire option_coverage test group (including non-Native tests), fresh
default workspace, strict clippy/fmt/wasm gates as scheduled. No new full scan
is authorized here. Report every producer and test-status change, retaining
source/build conventions and all untouched official failures as evidence.

## Approved stage1 results

Eight fixed sequential direct Native CLI executions completed with exit0, no
timeout, retry, Native build or consumer edit. Each used --slice 0 on an exact
copy of its captured input, a new output directory and empty ARES_ORCA_CACHE_DIR,
seed0 for initialized runs, and existing ares-pin-runner.sh pinned to CPU4.
These are uncached diagnostic executions, not OrcaRunner cache receipts.
Official's existing runtime-library cache was checked before execution so its
wrapper did not evaluate/build missing dependencies. Start load was 9.586-9.638.
Every request, stdout/stderr, result, input, Native output and segment list is
retained outside git at /tmp/worker6-safc-nd/preflight-v1; the exact command
script is /tmp/worker6-safc-nd/preflight.py and aggregate is summary.json.
The repeatable post-run integrity/pair audit is
/tmp/worker6-safc-nd/preflight-audit.py; its audit.json verifies the input and
producer file hashes, empty caches and complete initialized comparisons.

| Case | Producer | Repeat | Segments / Top | Generator-only SHA256 | Equal to retained Ares |
| --- | --- | --- | --- | --- | --- |
| on-concentric | official |1 |1622 /49 |2d04dbb0ebdb97744bf69540be0d93d6f1f67cf8bfb7c1fd286c2ca916e16ce1 |yes |
| on-concentric | official |2 |1617 /44 |c3e916db3b6a40fc64bae87dd928027a089b5e87529ed03bed35c9432d839012 |no |
| on-concentric | initialized |1 |1622 /49 |2d04dbb0ebdb97744bf69540be0d93d6f1f67cf8bfb7c1fd286c2ca916e16ce1 |yes |
| on-concentric | initialized |2 |1622 /49 |2d04dbb0ebdb97744bf69540be0d93d6f1f67cf8bfb7c1fd286c2ca916e16ce1 |yes |
| off-concentric | official |1 |1622 /49 |883f67d254a67796a8a0c0f073d0b3a26f69d99548b4016b8ed84d136155204c |yes |
| off-concentric | official |2 |1622 /49 |883f67d254a67796a8a0c0f073d0b3a26f69d99548b4016b8ed84d136155204c |yes |
| off-concentric | initialized |1 |1622 /49 |883f67d254a67796a8a0c0f073d0b3a26f69d99548b4016b8ed84d136155204c |yes |
| off-concentric | initialized |2 |1622 /49 |883f67d254a67796a8a0c0f073d0b3a26f69d99548b4016b8ed84d136155204c |yes |

The official on pair reproduces the retained failure: first normalized line
1009 is Outer wall versus M73 P32 R5; first motion difference is line3360,
X110.037 Y109.936 versus X105.955 Y105.955. Both initialized self-replays are fully
equal after replacing exactly one generated-by line. Both initialized on/off
pairs also preserve every observed segment's layer, role, XY endpoints, order
and exact E, not merely their counts. On/off full files differ in the requested
configuration flag; no claim that their configuration blocks are identical.

Full comparisons above use the existing worker3 c02 retained Ares bytes from
case-T8w6q0 and case-0ylXAs, not a fresh current-main Ares execution or a current
gate. Stage2 must perform that fresh comparison through the unchanged harness.
The observed initialized result supports bounded authority routing; it does
not prove the exact uninitialized values in each official process. No extra
debugger execution or inferred stack-field measurement is credited.

Per-execution raw hashes and launcher/endpoint/ELF hashes are in result.json.
Official ELF is 64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60;
initialized ELF is the 59fb88a2 hash above. The two source inputs remain exactly
the captured bytes throughout all four executions of each case. Source-owned
UB diagnosis and stage2 approval remain distinct from these finite observations;
the official failure is retained and is not relabeled PASS.

## Additional common-producer admission

The coordinator rejected adding replay BIN/SEED to official common refs for
the side effects above. Four additional fixed sequential fresh uncached CLI
runs used the existing common jqfsg42 producer on the exact same on/off inputs,
two executions each. All exited0, with1622 segments/49 Top and no timeout,
retry or rebuild. On normalized SHA is2d04dbb0ebdb97744bf69540be0d93d6f1f67cf8bfb7c1fd286c2ca916e16ce1;
off is883f67d254a67796a8a0c0f073d0b3a26f69d99548b4016b8ed84d136155204c.
Each equals both original1k0p5 initialized executions and the corresponding
retained Ares complete output. Both on/off pairs also have identical segment
keys/order/exact E. No RNG-equivalence claim extends beyond this fixture.

ELF SHA is8f0cf69dd9494ccfe53bf6749d6b685ccb5a0dd0df206694ce50d421007cc03d.
The explicit install is
/nix/store/jqfsg42f6xzq3krnf5scm6w4g1z4kz4c-orca-slicer-seed-replay-2.4.2;
its source rebuild/identity proof is the existing
[KSR authority ARD](ard-ksr-initialized-runtime-authority.md) and
[recipe](../../tools/orca-probes/ksr-initialized.nix), not a new Native build.
All four requests/raw outputs/logs/hashes are outside git in
/tmp/worker6-safc-nd/preflight-jqf-v1/summary.json and audit.json;
command script is /tmp/worker6-safc-nd/preflight-jqf.py.

The initial GREEN and unfinished workspace using the rejected routing are
SUPERSEDED, not acceptance evidence. Revised gates use the unmodified external
control script, without extra official parent replay variables. CI needs only
its already-used common KSR initialized three paths, ordinary Native/artifact
configuration, and inherited replay CPU from the existing runner. The internal
ARES_SAFC_CONCENTRIC_REQUEST variable carries the paired child request and does
not select a producer or require control/CI configuration.

## Revised harness acceptance

RED f5569c187 ran the real seven-scenario official harness; the new authority
policy failed because the old report lacked producer identity. Its exit100
log is /tmp/worker6-safc-stage2/red-official-v2/run.log. The earlier compile
errors are retained but are not credited as RED. The independent eight-run
preflight above retains the official whole-byte UB divergence.

GREEN 50948f7b0 changes only the harness: the five non-concentric cases retain
the caller producer, and the two concentric cases use the admitted common
KSR producer through a child and existing from_env/slice_case/cache identity.
No production code, comparison rule, shared cache or control script changed.
No Native artifact is committed. The previous global replay-variable routing
is superseded and not credited.

| Revised gate | Result | Retained log root |
| --- | --- | --- |
| seeded cold/hot entire option_coverage |200/200 each, exit0 |/tmp/worker6-safc-stage2/gates-v3-directed/seeded-{cold,hot}/run.log |
| official cold/hot entire option_coverage |200/200 each, exit0 |/tmp/worker6-safc-stage2/gates-v3-directed/official-{cold,hot}/run.log |
| strict clippy / fmt / wasm32 |all exit0 |/tmp/worker6-safc-stage2/gates-v2/{strict,fmt,wasm32}/run.log |
| fresh default workspace |8339/8339, exit0 |/tmp/worker6-safc-stage2/gates-v4-default-workspace/run.log |

All gates ran on 50948f7b0 with jobs2/threads2. Default workspace removed
ARES_/ORCA_ and runner variables and started after the coordinator released
the W5 full slot. The interrupted v2 seeded cold and superseded workspace
remain outside git as non-evidence. Cold caches were fresh and separate by
mode; each hot run reused only its mode cache. The group includes non-Native
option_coverage tests; no new full scan was authorized or claimed.

/tmp/worker6-safc-stage2/gates-v3-directed/producer-audit.json checks eight
seven-case reports: first five cases retain each caller ELF and official has
no replay seed/endpoint; only the concentric pair uses jqfsg42. The adjacent
ares-raw-audit.json proves the same seven labels' raw Ares SHA across all
56 records. All whole-byte and segment comparisons remain PASS.

Original nine assertion texts sort to SHA256
509ab2013415cc7bba584a5e94014b5200a2e706e0cfc941c312ee61212f195a
in both versions; assertions.diff is empty. Seven case/five coupled-pair
definitions and the observation module are byte-identical. Original
application checks, role requirements, whole-output and coupled-consumer
calls are unchanged; evidence is /tmp/worker6-safc-stage2/assertion-audit.json
and original-consumer-calls-*.txt. Official's retained UB failure is not
reclassified as an untouched official PASS.
