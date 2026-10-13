# C2-S public admitted RED

SPEC `9e69d1b8c` approved, base merged `a120ebd5c`.
This test-only commit precedes any production repair.

Source: PrintObject.cpp:3602-3709 local PrintRegionConfig merge;
Fill/Fill.cpp:891-893 selects ordinary internal-solid pattern, separately
from sparse-only anchor generation at :1394-1446 and bridge_over_infill
PrintObject.cpp:2735-2754. No W6 Grid arm/predicate changes or sparse Grid
inputs: sparse is rectilinear20, global solid is monotonic, top monotonicline.

The public exporter creates a cube10 ceiling, or a1.4mm short body by
scaling pinned binary STL vertex Z by0.14 before export. Exactly one local
internal-solid pattern metadata key is inserted at object or normal part
scope; every other ZIP member is SHA-checked unchanged. Final bytes are used
for both on-disk Native and public API/CLI input. Metadata and source/mesh/
input SHA are retained in each external witness.

Three E2E tests pair rectilinear with Grid: active object, active model-part,
and measured geometry with no bridge candidates. Native must complete a
CONFIG_BLOCK_END stream, have positive XY extrusion moves under Internal
solid infill, and change that role's move SHA on Grid. Positive Internal
Bridge moves distinguish active ceiling from the short control. Whole-output
PASS, public API/CLI raw equality and CLI whole Native comparison stay the
permanent expectation; failures are not converted into passing assertions.

Official CPU13 and seeded CPU14, outer cargo taskset, jobs2/threads2,
independently fresh cache/artifact roots, both1PASS2FAIL exit100:

- official `/tmp/worker2-c2s-red-official.xjtFxE`.
- seeded `/tmp/worker2-c2s-red-seeded.dNyAGW`.

Both modes measure52 positive bridge moves for active object/part controls
and Grid. Rectilinear solid moves287; Grid277, with different role SHA.
Both active Grid API outcomes are ARES_ERROR with the exact unsupported
internal_solid_infill_pattern detail; CLI stderr is exactly
Error: UnsupportedProjectFeature("internal_solid_infill_pattern"), exit1.
Only the whole-output PASS assertion fails, after all Native witness checks.

The1.4mm control has zero bridge moves, solid moves275 rectilinear/265 Grid,
with different role SHA. Both whole API/CLI comparisons already PASS,
as do the two active rectilinear controls per mode. No Native rejection,
missing role, setup or timeout failures. Tests117LOC, fixture182LOC;
production and all existing assertions are untouched.

External joined evidence `/tmp/worker2-c2s-red-evidence.json` retains all12
case paths, producer/launcher receipts, raw/error streams and SHA. Native
artifacts remain external. The next repair remains only the internal-solid
gate/newly-unused import in anchor_projection.rs; if insufficient, STOP and
request PLAN, without editing RED assertions or W6 files.
