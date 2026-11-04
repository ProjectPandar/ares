# Slice 6: retain the fuzzy nondeterminism reason in incomplete outcomes

Revised SPEC; docs only. This replaces the wider 13fdb3d94 proposal with a
narrower scope. Order: slice3, slice5, then this slice, each separately
RED → GREEN. This document starts no Native/cargo runs.

## Upstream ownership and existing evidence

libslic3r/PrintConfig.cpp::PrintConfigDef::init_fff_params defines
fuzzy_skin_noise_type:3611, fuzzy_skin_scale:3637, fuzzy_skin_octaves:3647
and fuzzy_skin_persistence:3656. Feature/FuzzySkin/FuzzySkin.cpp::random_value
:26-31 uses thread_local random_device/mt19937 seeded by entropy or thread-ID
hash. fuzzy_polyline:294-342 and fuzzy_extrusion_line:345-412 consume that
random spacing/offset stream; this is the source owner of the existing fuzzy
exception predicate, not lightning's rand().

The [full audit](2026-10-14-option-full-current-audit.md) already retains
42 divergent attempts in14 same-input official groups, with varying Native
SHA and stable Ares SHA; seeded matched those declared cases. That evidence
is sufficient for this reporting correction. Do not collect new provenance
or duplicate the audit's attribution. Historical official four INCOMPLETE
and seeded four PASS results remain unchanged.

## Exact Rust boundary and included changes

Only two changes are included:

1. In crates/ares-cli/tests/orca_parity/option_coverage.rs,
   OptionOutcome::executed must retain INCOMPLETE when compared==0 and
   excused>0, but its detail must also identify ORACLE_NONDETERMINISTIC.
   The current zero/missing-comparison branch precedes excused>0 and loses
   that reason. Preserve the incomplete-comparison reason as well; correct
   the condition/detail handling only to retain both facts. Keep first
   failure precedence, actual compared/excused/rejected counts, legality
   checks and all PASS/FAIL/INCOMPLETE decisions unchanged.
2. In option_coverage/sweep.rs replace the stale lightning rand() explanation
   beside the fuzzy excusal branch with the correct
   Feature/FuzzySkin/FuzzySkin.cpp:26-31 random_value reference.
   Do not change the predicate, excusal behavior, retries or comparator.

These are test-harness reporting/ownership corrections around existing
upstream compatibility witnesses, not a new Ares randomization subsystem.
No production slicing or random-stream implementation is included.

## Public RED before GREEN

Add a public directed regression
asserting the four fuzzy domains' external detail text retains the oracle
nondeterminism reason whenever an actual outcome has excused cases, including
compared==0. The existing noise_type wording already reports that reason;
keep it as a positive control, without requiring a new token or changing its
branch. The repaired zero-compared detail must expose ORACLE_NONDETERMINISTIC.
Also assert that those excused outcomes remain
INCOMPLETE, never PASS. The current numeric-domain details lose that reason;
record the missing-detail assertion as the reporting RED in its own commit
and report it before making either repair.

Use the existing official option sweep and external summary, preserving all
case receipts and strict comparisons; seeded remains the paired strict-byte
control. A stochastic mismatch is not the RED expectation: the missing reason
on an actual excused outcome is. If the fresh official batch produces no
applicable excused outcome and cannot establish this public witness, stop and
report rather than force divergence or fabricate a RED.

GREEN keeps the RED assertions intact and performs only the two scoped
repairs above. Preserve the historical audit and original failure evidence.
Check the zero-compared numeric details and noise_type detail in official;
seeded strict matches must retain their actual PASS status without an ND
label inferred merely from an option name. Existing failures still fail.

## Explicitly deferred and gates

No fuzzy_provenance.rs, producer/wrapper/SHA collector, per-domain provenance
attachments, runner API changes, new artifact schema, exception-set widening,
retry changes, generator normalization changes or weaker comparator.
No new fixtures, seeded-oracle changes, RNG/geometry ports, statistical
comparison or inventory/exclusion changes. Existing42-attempt attribution
stays in the audit and is not reimplemented here.

Directed execution uses a fresh root/cache/target for each mode. After GREEN run
paired directed and required strict clippy/fmt/wasm32; keep files <=400 lines.
Exit: unchanged domain statuses/counts and acceptance, complete ND detail
on actual excused outcomes, correct upstream comment.
