# Slice 5: bounded post_process CLI admission accounted in the matrix

Approved SPECd22fca1cf; public REDc8c541129. Branch
worker2/post-process-matrix-s5 stacks on reviewed s3 6e67e49de;
origin/main at start96328f269. This is a tests/docs slice, no production,
shared outcome-format, comparator or inventory changes.

## Source-owned bounded contract

PrintConfig.cpp::PrintConfigDef::init_fff_params:5068-5079 defines coStrings.
OrcaSlicer.cpp:1682-1690 normative embedded-vector admission rejects any
nonempty vector by size, before external settings composition:1953-3495.
It records -19/Unix237 and plate0; channels/result use:398-460.
External GCode/PostProcessor.cpp::run_post_process_scripts:238-377 is deferred;
the CLI call OrcaSlicer.cpp:6247-6248 is commented out. Internal
GCodeProcessor post-processing is a separate unchanged boundary.

The original three post_process.rs tests still use the same comparison body,
now also callable with the matrix's required runner. All5 assert! and16
assert_eq! checks remain, including unchanged other ZIP/settings values,
exact input after slicing, Native/Ares marker absence, exit/channels/result
JSON and full successful G-code comparison. No original assertion or test
name was changed, and the RED regression is unchanged fromc8c541129.

post_process_matrix.rs supplies three finite cases and the scoped executor;
domains.rs/sweep.rs dispatch only this key. The existing root module wiring
stays intact; option_coverage.rs adds the Linux-only matrix module. Non-Linux
keeps an explicit unresolved baseline/omission, not a false PASS or ignored
key. No filesystem/process behavior was added to core/WASM.

| Case | Compared contract | Matrix credit |
|---|---|---|
| empty-vector [] | actual CLI success and whole G-code | 1 strict G-code |
| empty-string-vector [""] | -19/237, plate0, exact channels/full result JSON, no G-code | 1 expected CLI rejection |
| owned-script singleton | same rejection plus owned executable and absent marker | 1 expected CLI rejection |

The owned-script plan value is explicitly a request template: each invocation
creates its temporary owned path; the comparison receipt records the actual
absolute vector. Input ZIP settings independently match that concrete
request. The template is never injected into Native or Ares as a substitute
for the owned executable witness. Every other input member/setting is held.

compared=3 counts these contracts, **not three G-code outputs**. The row detail,
post-process-comparison-counts.json and each kind-tagged comparison receipt
state1 strict G-code +2 expected CLI rejection; rejected=0 means no unexpected
rejection. Existing comparisons must return successfully before credit.
Case/count receipts publish atomically for sweep/regression coexistence.
Partial value selection still clears the plan's completeness; existing
OptionOutcome status logic and summary format are unchanged.

## Final paired evidence

| Mode | Independent fresh root/cache/target | Result |
|---|---|---|
| official | /tmp/worker2-s5-green-official.zhOUae | 4/4PASS exit0; matrix3 contracts/0 unexpected rejection/PASS |
| seeded | /tmp/worker2-s5-green-seeded.on9ERA | 4/4PASS exit0; matrix3 contracts/0 unexpected rejection/PASS |

Outer taskset CPU13/14 respectively, jobs2/threads2. Driver:
/tmp/worker2-s5-green-leg.sh. Only the existing three CLI tests and new public
matrix regression ran; no full option resweep, full or workspace. START/END
were posted to inbox. No cache/artifact/target was reused between Native runs.

Independent offline reread verifies6 actual contracts per mode (three
standalone plus three matrix): total4 whole G-code comparisons and8 exact
expected CLI rejection comparisons. It verifies concrete requested vectors,
all other ZIP members/settings, Native receipt input/output SHA and exit237,
raw channels, -19/plate0 JSON, Ares exit, no G-code on rejection and no marker.
G-code uses only existing generator normalization; result JSON uses exactly
the existing first prepare_time/export_time integer-span replacement.
No additional normalization, golden authority or Native rerun was introduced.

Reproducer:/tmp/worker2-s5-verify-final.py; SHA/receipt report:
/tmp/worker2-s5-green-review-evidence.json. Final source hashes match
/tmp/worker2-s5-final-source-snapshot.json. Native products stay external.

Engineering root:/tmp/worker2-s5-engineering.XuzDO9
Strict clippy --workspace --all-targets --all-features -- -D warnings:0;
fmt --all -- --check:0; check -p ares-core -p ares-wasm --target
wasm32-unknown-unknown --all-features:0. All ran on CPU13,14 with jobs2,
standard clean env and no Native before the frozen-source paired GREEN.

Exit: independent s5 REVIEW-REQUEST, no push; heavy gates await coordinator GO.
PASS covers only these embedded ordinary Linux CLI admission witnesses.
Arbitrary scripts/coStrings, external execution, load-settings composition,
normative_check=false, other platforms and extra whitespace/multi-entry
cases remain deferred. s6 has not started.
