# SupportCubic B-S: cube-wall continuation threshold

Status: SPEC-REVIEW-REQUEST, docs only. No fix implemented.
Base main caac760d464112b076821975b67232fdd0f14bf0.
Branch worker3/support-cubic-b-s-gap-fix-spec.
Approved localization e2ec23616 followed the residual on frozen1fc075c55.
The original worker3/support-cubic-b-s-red branch still points to e2ec23616;
1fc075c55 was not amended. This docs branch rebases the same nine commits,
all range-diff entries `=`, mapping production1fc tobf0c8c4d5 unchanged.

## Exact upstream boundary and minimal Rust change

OrcaSlicer/src/libslic3r/Fill/FillAdaptive.cpp::
generate_infill_lines_recursive:483-491 owns the temporary wall's continuation:
create new_line with Point::new_scale, then flush the previous line when
`(new_line.a - last_line.b).cwiseAbs().maxCoeff() > 1000`, updating last_line.a
and finally last_line.b. The threshold is a literal in the current scaled
coordinate domain, not25mm. Point.hpp:204-206 and libslic3r.h:60-61/70/93-94
define scaling; the active normal factor in this S45 case is1e-6mm.

Ares destination: crates/ares-core/src/fill/adaptive/filler.rs::fill_surface
initializes FillContext::connect_epsilon, consumed by
::generate_infill_lines_recursive:124-132. Its current
`(25.0 / scale.factor()) as i64` yields25,000,000 in the measured Normal
domain. The comment incorrectly converts the upstream literal through an
old40/mm scaffold. This scaffold is a temporary compatibility shell around
FillContext; replace only this incorrect threshold conversion with literal
`1000`, and correct its source comment to FillAdaptive.cpp:487.

Keep the existing strict `gap > connect_epsilon` comparison, max-absolute gap,
slot/address/traversal order, flush/update logic and constructor carrier.
No rescaling of1000 to a physical distance: upstream keeps the literal in
the active domain, including its large-printer SCALING_FACTOR setting.
No helper extraction, API/option change, guard, suppression or fallback.
This is shared Adaptive/Support Filler behavior; no fixture/pattern predicate.

## Bounded observed cause and artifact provenance

The approved60min investigation used1796.457s active (~30min), excluding the
FULL13 freeze. All runtime investigation ended before the deadline.
Only read-only GDB over existing binaries was used: no source instrumentation,
inferior calls/data writes, Native/Rust rebuild or trial production edits.
Nine probes inherited CPU12, each fresh root/cache/output. All GDB and actual
inferior exits0. All five Ares runs were raw-output neutral against frozenCLI;
all four Native runs differed from their frozen mode reference only in the
already-approved generated-by metadata line. No normalization was introduced.

At layer13/Z2.6, actual Ares FillContext.connect_epsilon is25,000,000.
Native's actual compiled comparison uses1001 as the inclusive branch boundary
(cmp0x3e9 followed by jl), matching the source strict `>1000` expression.
The first differing continuation decision is direction0/address13 with gap
4,894,845 scaled units: Native flushes; Ares merges. In the Normal domain the
source threshold is0.001mm (1micrometre), versus Ares25mm.

All282 from/to wall pairs were observed before Native scaling. Native integer
endpoints were DERIVED from those actual doubles using the verified division
and truncating cvttsd2si sequence; they are not mislabeled as integer-register
reads. Each derived endpoint matched its actual Ares new_a/new_b observation.
This gives an independent common endpoint input for source continuation replay.

Offline source replay of the SAME captured endpoints with threshold1000
exactly reproduces all54 ordered Native clipping-input lines. With the actual
Ares threshold25,000,000, it exactly reproduces all43 ordered Ares input lines.
This is an analysis receipt, not a modified Ares build or claimed GREEN.
Replay script is retained with the runtime evidence; no Native arrays enter Git.

Native and Ares clipping contours are byte-for-byte equal integer vertices.
Both produce37 clipped lines; zero-based indices28 and36 differ. Index28 contains
the final S45 X endpoint difference: Native18,039,499 vs Ares18,039,501 in
local scaled coordinates. The final Native filler point was independently
observed as18,039,499, and writer/formatter inputs in both modes were
128.039499; frozen Ares formatter input was128.039501 with origin110.
The pre-format difference is0.000002mm (2nanometres), which crosses the
0.001mm output rounding boundary. Earlier commentary incorrectly called the
pre-format difference2micrometres; the measured numeric values are unchanged.
No formatter, extrusion or travel adjustment is proposed.

## Ruled-out observations and retained partial captures

Native and Ares octrees contain the same129 node paths. Pre-transform root
centers match exactly; seven rotation coefficients and128 transformed centers
have tiny floating differences (Quaternion versus axis-matrix multiplication).
They did NOT change the measured integer wall endpoints, so they are not
claimed as the cause and are deferred. Do not fix rotation alongside this slice.

Both trees' observed coordinates are object-local: actual Ares preparation
offset0, emitter origin110. An early comparison wrongly added110 and the
historical event label tree_world_centers is misleading for this input.
That comparison is preserved and explicitly superseded by the local-frame
analysis; its differences are not evidence for a translation fix.

The first seeded capture's wall-symbol installation failed while root/final
point/formatter reads succeeded. A subsequent fresh capture corrected only
the reader using the actual image mapping and validated symbol-relative RVA.
Another fresh capture added the clipping boundary; these are additional
debugger observations, never retries counted as parity PASS. The Ares hook
locals were optimized out in14 observation sites; they remain unavailable,
not reconstructed. Clipping divergence precedes hooks, so no hook fix is needed.

## RED, GREEN and review plan after approval

Use the existing approved public RED a5f485f15/05c6ad0b6 and S45 residual;
do not add late unit tests, change fixtures, normalize or weaken assertions.
1. Make the single threshold/source-comment production change separately from
   the frozen B-S object-preparation commit; no other production edits.
2. Fresh official and seeded directed on CPU12, jobs1/threads2: complete
   S45/S50/S125/SAUTO plus A45/A50/A125/AAUTO/R1/L1 all strict PASS. Check
   every control's API/CLI raw bytes against the accepted RED baseline.
3. Re-run B-T runtime GLU-oracle directed and existing octree/filler controls,
   plus strict workspace clippy/fmt/wasm32 gates on the exact final head.
   Full/workspace TEST runs remain coordinator-only.
4. Request review of the complete RED+B-T+B-S+threshold stack; merge together
   only after coordinator allocation/approval. No push or partial merge.

Any full-output residual or changed control remains STOP for localization/
amendment. No test retirement is proposed; request separately if any existing
assertion conflicts. Adjacent rotation, child/SAT order, clipping/hooks,
chaining, flow/E/ETA and G-code coordinate contracts remain deferred.

## Durable evidence

/home/indexyz/ares-evidence/worker3/support-cubic-b-s-gap-localization/
localization-evidence.json SHA256
7acca75e4c573f2598d30878ce3bc364a3829f2b127723cd7780ae200345e7ff.
Archive s45-gap-root-cause.tar.gz in that directory, SHA256
64eb97917f40e0b4eca364fcad3113bf98ba62c455b951ee2081fdc0e384a724.
It retains all roots, full output neutrality receipts, actual state, readers,
symbol-relative assembly, replay script/input receipts, corrected/partial
observations, window/freeze records and9/9 range-diff. Binaries/caches excluded.
Committed summaries survive /tmp cleanup without frozen Native test outputs.
