# Slice 6: retain the fuzzy oracle reason in incomplete details

Approved narrowed SPEC cd8505877; stacks on reviewed s5 819419cc9 (and s3).
Public RED is separate commit 26136e5f0. Its test and assertions are unchanged.

Only two repair sites change: OptionOutcome::executed adds
ORACLE_NONDETERMINISTIC to the existing incomplete detail when compared==0
and excused>0; sweep.rs replaces the stale lightning comment with
Feature/FuzzySkin/FuzzySkin.cpp::random_value:26-31, whose generator is
thread-local and seeded from entropy/thread identity. Existing failure
precedence, statuses, counts and missing-comparison explanation remain.

## Public Native evidence

Driver: /tmp/worker2-s6-green-leg.sh. Each mode used a fresh root/cache/target,
outer taskset CPU13 (official) or CPU14 (seeded), jobs2/threads2, and the
existing directed control/comparator. Filter:
fuzzy_incomplete_details_keep_actual_oracle_nondeterminism.
Both nextest runs passed 1/1; these are reporting-regression results.
Official coverage remains four INCOMPLETE domains, not four PASS domains.

| Domain | PrintConfig.cpp | Cases | Official RED → GREEN | Seeded RED → GREEN |
|---|---:|---:|---|---|
| fuzzy_skin_noise_type | 3611 | 6 | INCOMPLETE, compared1; existing5-excused reason unchanged | PASS, compared6; unchanged |
| fuzzy_skin_octaves | 3647 | 3 | INCOMPLETE, compared0; ND reason restored | PASS, compared3; unchanged |
| fuzzy_skin_persistence | 3656 | 3 | INCOMPLETE, compared0; ND reason restored | PASS, compared3; unchanged |
| fuzzy_skin_scale | 3637 | 3 | INCOMPLETE, compared0; ND reason restored | PASS, compared3; unchanged |

Both modes still generated15 cases and rejected0. Official compared1,
seeded compared15. Offline comparison of the existing RED/GREEN summary
rows verified identical status/key/type/case/compared/rejected/source columns.
Only the three official zero-compared detail strings gain the reason;
the noise-type and all seeded rows are identical.

GREEN roots:
- official: /tmp/worker2-s6-green-official.YlnmgL
- seeded: /tmp/worker2-s6-green-seeded.nKNCB5

Final RED roots are recorded in the companion RED document. Logs and existing
summaries remain external; the approved audit retains the42-attempt evidence.
No additional provenance collector/schema, exception, retry, comparator,
Native RNG or production change is included.

## Engineering gates

Root: /tmp/worker2-s6-engineering.4hdcyp; CPU13/14, jobs2, all exit0:
- cargo clippy --workspace --all-targets --all-features -- -D warnings
- cargo fmt --all -- --check
- cargo check -p ares-core -p ares-wasm --target wasm32-unknown-unknown --all-features

All touched files remain below400 lines. START/END posted to inbox.
No full/workspace or push was run; stacked CI/full remains coordinator-owned.
