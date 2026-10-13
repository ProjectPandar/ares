# Slice 6: official fuzzy stochastic-domain provenance and truthful reporting

Status: SPEC-REVIEW-REQUEST; docs only; follows separately reviewed slices3/5.
Baseline: audit96328f269, executed f637b4122. Historical raw results remain
immutable: official four INCOMPLETE domains, seeded four PASS domains.

## Source-owned random behavior

All paths below are under OrcaSlicer/src/libslic3r, unless qualified.

- PrintConfig.cpp::PrintConfigDef::init_fff_params defines noise_type:3611,
  scale:3637, octaves:3647 and persistence:3656; these configure FuzzySkin,
  not an independent Ares randomization facility.
- Feature/FuzzySkin/FuzzySkin.cpp::random_value:26-31 uses thread_local
  random_device/mt19937, seeded by entropy or thread-ID hash. Disabling ASLR
  does not supply a reproducible RNG stream or thread identity contract.
- get_noise_module:41-65 creates the requested coherent-noise module or
  UniformNoise. Even coherent modes use random point spacing in the caller;
  a fixed module seed alone is not a whole-output determinism proof.
- fuzzy_polyline:294-342 and fuzzy_extrusion_line:345-412 consume random
  spacing/offset values. Ripple branches instead use their existing separate
  implementations; fuzzy_extrusion_line_ripple:223-291 is not the stochastic
  extrusion-line implementation at :345. Keep the branch distinction.

The native seeded oracle is a distinct recorded producer/replay contract,
not the unmodified official producer. Its byte equality is useful evidence
for that contract, not proof of official stochastic byte reproducibility or
of all Ares random-distribution semantics.

## Established evidence, not a new production RED

The fresh official sweep retained14 same-input groups: five noise modes
(billow/classic/perlin/ridgedmulti/voronoi) and three cases each for
scale/octaves/persistence. Canonical existing retries produced42 divergent
attempts; every group has multiple normalized Native SHAs and one Ares SHA.
Seeded passes the same declared cases. This proves variation on those exact
official inputs, not Ares stochastic correctness. The raw domain FAIL in the
audit is printer_technology application mismatch, outside this slice.

Current reporting loses the reason when compared==0: OptionOutcome::executed
in option_coverage.rs checks zero/missing comparisons before its excused>0
branch. Three numeric fuzzy rows therefore say only required cases not all
compared, although their case events record ORACLE_NONDETERMINISTIC.
The sweep comment also describes lightning rand(), while the predicate
currently identifies fuzzy options. Correct ownership/reporting, not geometry.

## Rust destination and included work

Own crates/ares-cli/tests/orca_parity/option_coverage/sweep.rs stochastic
classification evidence, OptionOutcome/detail/report wiring in
option_coverage.rs, and a small option_coverage/fuzzy_provenance.rs test helper
if required. Reuse existing runner/parity retained case manifests and retry
receipts; inspect their exact types before any approved code diff. Read
activation.rs/seeded_context.rs for requested/effective context but do not
change fixtures, random seeds, production slicing or oracle executables.

Attach source/config/producer evidence to those four domains and their
existing cases: requested/effective noise context, same-input identity,
actual producer/wrapper identity, attempt artifact paths, raw and comparator-
normalized Native/Ares SHA, and exact comparator identity. Unknown provenance
stays explicitly unknown; never manufacture an upstream revision or seed.
Use the existing file-backed artifact convention, no new terminal prints.

Keep raw outcome and cause separate. When zero strict comparisons and
excused cases coexist, preserve INCOMPLETE and expose both missing-comparison
and observed oracle-variation causes. Reconcile each excused case with its
retained evidence; a whitelist entry alone is not a measured variation proof.
Distinguish observed variation from source-expected randomness and unknown
provenance. An all-identical retained Native SHA set cannot be relabeled as
observed nondeterminism; stable divergence remains a discrepancy to report.
If stronger evidence-gating changes current excusal/status, submit its exact
diff and attribution for coordinator approval before extending this scope.

No exception-set widening, extra retries, retry-to-green, entropy pinning,
normalization changes, removed G-code lines or coordinate tolerances. Preserve
every comparison attempt and raw result. Successful strict matches retain
their actual result, not an invented ND label based on a known option name.
Historical official INCOMPLETE remains published even if later samples differ.

## Public reporting RED, then GREEN after SPEC approval

First commit a public directed matrix regression over these four keys that
checks current generated domain/source metadata and external per-case proof
receipts. It must fail for absent stochastic ownership/producer/attribution
or the lost zero-compared reason. Record official Native controls and seeded
strict matches; do not assert every newly drawn official sample must diverge.
The deterministic reporting expectation, not a probabilistic mismatch, is
this slice's RED. Report before implementing the metadata/detail repair.

Use retained historical42-attempt evidence for the known ND attribution
control; it is external execution evidence, never committed Native output
or a replacement authority for a fresh parity run. If current runner receipts
cannot establish same input/producer, report the gap before adding a collector
or changing runner APIs. No fabricated behavioral RED or production fix.

GREEN must show truthful source/evidence metadata, INCOMPLETE still non-PASS
and completeness-gate failure when official cases are excused, seeded strict
comparisons unaffected, and stable/non-comparison failures not hidden by the
ND label. Existing comparator and all acceptance assertions remain intact.
Other domains, notably printer_technology, retain their existing statuses.

## Deferred, gates and stop conditions

Defer Ares RNG/distribution changes, deterministic official oracle patches,
Arachne geometry ports, new noise modes/fixtures, generic retry infrastructure,
full legal numeric/random-stream coverage and statistical-equivalence tests.
Existing scaffolds are compatibility/reporting shells around the cited
FuzzySkin config/path behavior; this slice creates no Ares-owned algorithm.

After approval, directed uses CPU13/14 outer taskset, fresh root/cache/target,
jobs2/threads2, only within an open window. Keep the existing canonical retry
budget; no full/workspace without separate GO. Strict/fmt/wasm32 must pass.
Stop for unexcused byte differences, uncertain provenance or scope requiring
production/runner redesign; provide source/receipt evidence for review.
Each file <=400 lines. Exit requires a separate code REVIEW with raw mode
results and provenance, never an official coverage-PASS claim from excusal.
