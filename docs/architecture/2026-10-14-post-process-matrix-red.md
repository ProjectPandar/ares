# Slice 5: bounded CLI admission matrix RED

Approved SPECd22fca1cf; branch worker2/post-process-matrix-s5 stacks on
code-reviewed s3 6e67e49de. origin/main remains96328f269; no push.
Only a public matrix regression was added. Three existing CLI tests and
all their Native/Ares assertions remain unchanged; no implementation yet.

Owner: libslic3r/PrintConfig.cpp:5068-5079 coStrings; normative embedded
vector-size rejection OrcaSlicer.cpp:1682-1690 precedes Print/overlay.
External PostProcessor.cpp:238-377 execution is a different deferred scope;
the CLI execution call OrcaSlicer.cpp:6247-6248 is commented out.

| Mode | Fresh root/cache/target | Result |
|---|---|---|
| official | /tmp/worker2-s5-red-official.2bS1kL | 4run/3PASS/1expectedFAIL, exit100 |
| seeded | /tmp/worker2-s5-red-seeded.yvyOQG | 4run/3PASS/1expectedFAIL, exit100 |

Driver:/tmp/worker2-s5-red-leg.sh. Outer taskset CPU13 or14 respectively,
jobs2/threads2, existing directed control set, no full/workspace.
Selection: test(post_process_cli::post_process_) plus the new
orca_parity_post_process_accounts_bounded_cli_admission_cases regression.
START/END posted to coordinator inbox; logs and Native products stay external.

The existing [] success, [""] exact -19/237 rejection and owned-script
rejection/no-execution tests PASS in each mode. The public matrix regression
executes and publishes the current baseline first: Native/API baseline
comparison PASS, but one generated/compared None-valued case and INCOMPLETE.
It fails at sweep_tests.rs:235, left INCOMPLETE, expected PASS; detail
`required legal cases not all compared; `. Not a slicer discrepancy or a
Native rejection of an allegedly legal success fixture.

Approved GREEN replaces baseline accounting with three bounded ordinary-CLI
contracts:1 strict G-code comparison and2 expected CLI rejection comparisons.
Keep unexpected rejections distinct, preserve all existing assertions,
record concrete owned-script vector and absent-marker evidence, and defer
external execution. This is not exhaustive arbitrary coStrings coverage.
