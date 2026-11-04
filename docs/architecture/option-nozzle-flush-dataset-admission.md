# Nozzle dataset six-case Native admission

All six standard seeded Native launches exit0/result0, non-VOID.
No Native output golden is committed.

Fixed source input comes from the unchanged existing
`x2d_flush_matrix_standard_predictor_hit_matches_native` test. All other ZIP
members/settings are identical; only dataset and explicit-matrix presence vary.
Colours are black → #545454 and reverse, multiplier `[0.3,1]`, map `[1,1]`.

| Case | Four input slots | Exported selected datasets | Exported matrix |
|---|---|---|---|
| D0 | 0,0,0,0 | 0,0 | 0,98,46,0,0,328,152,0 |
| D1 | 1,1,1,1 | 1,1 | 0,135,27,0,0,450,90,0 |
| D2 | 2,2,2,2 | 2,2 | 0,81,45,0,0,270,150,0 |
| DM | 1,2,2,1 | 1,2 | 0,135,27,0,0,270,150,0 |
| C1 | 1,1,1,1 | 1,1 | 0,42,42,0,0,210,210,0 |
| C2 | 2,2,2,2 | 2,2 | 0,42,42,0,0,210,210,0 |

## Source attribution and limits

`OrcaSlicer.cpp:3351` rebuilds an absent matrix, whereas C1/C2 retain their
explicit matrix with unchanged nozzle types. Lines3436-3463 select datasets
from printer variants; lines3468-3495 calculate each ordered colour pair per
nozzle. `FlushVolPredictor.cpp:312-327` selects standard, dual-standard and
dual-highflow resources for IDs0/1/2. `FlushVolCalc.cpp:48-62,102-124` owns
prediction and minimum-volume/cap behavior. D0 exports first-head98/46 and
second-head328/152. No intermediate predictor values or exact rounding path
were instrumented, so these outputs do not establish a floor rule. D1 scales
450/90 to135/27;
D2 scales270/150 to81/45. Do not invent a floor or tolerance to replace exact
Native values. C1/C2 scale the supplied first-head140 to42 and preserve the
second-head210. DM combines D1's first-head matrix and D2's second-head matrix,
proving selected-dataset consumption rather than only input serialization.

Complete pairwise audits D0/D1, D1/D2, D1/DM and D2/DM differ only in generated
wall-clock banner, computed matrix and dataset config lines. C1/C2 differ only
in banner and dataset config. All compared command streams are equal in this
fixed one-object scene. This is computed configuration/reconstruction coverage,
not a tower/tool-change-motion witness. No stronger slicing claim is made.
The matrix is derived upstream output, not merely the serialized input key.

Diagnostic command-only classification is not an acceptance comparator.
Every CLI E2E still compares each complete validated API and real CLI output
against freshly produced Native with the unchanged strict comparator. Existing
`x2d_flush_matrix` assertions remain untouched. Production mismatches stop for
source-owner review rather than unilateral repair.
