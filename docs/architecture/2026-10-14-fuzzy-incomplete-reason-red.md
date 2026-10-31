# Slice 6: public missing nondeterminism-detail RED

Approved narrowed SPECcd8505877. Branch worker2/fuzzy-incomplete-reason-s6
stacks on reviewed s5 819419cc9 (including s3); no push.
Only the new four-domain public regression changed; no repair yet.

Upstream owners: PrintConfig.cpp:3611/3637/3647/3656 and
Feature/FuzzySkin/FuzzySkin.cpp::random_value:26-31. Existing audit already
owns the42-attempt provenance; this slice adds no collector or schema.

| Mode | Fresh root/cache/target | Test result |
|---|---|---|
| official | /tmp/worker2-s6-red-final-official.3Jwag3 | 1expectedFAIL, exit100 |
| seeded | /tmp/worker2-s6-red-final-seeded.Lhv93w | 1PASS, exit0 |

Driver:/tmp/worker2-s6-red-final-leg.sh; outer taskset CPU13/14 respectively,
jobs2/threads2, existing canonical comparison/retry behavior. Only the new
fuzzy_incomplete_details_keep_actual_oracle_nondeterminism test
ran. Four live domains execute before summary publication/assertions.

| Key | Cases | Official compared/status/detail | Seeded compared/status |
|---|---:|---|---|
| fuzzy_skin_noise_type | 6 | 1/INCOMPLETE;5 cases excused, nondeterministic reason already present | 6/PASS |
| fuzzy_skin_octaves | 3 | 0/INCOMPLETE;required legal cases not all compared; ND reason missing | 3/PASS |
| fuzzy_skin_persistence | 3 | 0/INCOMPLETE;same missing reason | 3/PASS |
| fuzzy_skin_scale | 3 | 0/INCOMPLETE;same missing reason | 3/PASS |

Both modes have0 rejected cases. Official has14 existing
ORACLE_NONDETERMINISTIC case events; its actual zero-compared outcomes
establish the reporting witness. The assertion fails at sweep_tests.rs:343
on octaves missing ORACLE_NONDETERMINISTIC. This is the missing-reason RED,
not a requirement that newly sampled official outputs must diverge. All
actual strict matches remain valid; seeded gives the paired strict control.

Initial roots /tmp/worker2-s6-red-official.Ns8i1v and
/tmp/worker2-s6-red-seeded.NSV3Op already produced the same real witness, but
the test's reference_enabled mode query printed a misleading SKIPPED line
in official after comparisons had run. The getter was corrected to direct
env==1, with assertions unchanged, then both legs were rerun fresh above.
No test mode-query message or build failure is presented as behavioral RED.

Logs, existing summaries and case artifacts remain external. No additional
Native attribution reruns, producer/wrapper/SHA collection, per-domain
provenance fields, comparator/exception/retry changes or Native products
were introduced. START/END posted to inbox. GREEN is limited to the existing
zero-compared/excused detail and the stale lightning source comment.
