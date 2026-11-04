# Legacy brim_ears six-case Native admission

Six standard seeded Native launches exit0/result0/non-VOID.
All non-settings ZIP members identical; each bool pair differs only in
`brim_ears`. Shared width4/gap0.1/angle125/detection1, no support/prime tower.
Input originates from unchanged existing brim_type option sweep.

| Selector | bool0 / bool1 brim extrusion command count |
|---|---|
| no_brim | 0 / 0 |
| outer_only | 384 / 384 |
| brim_ears | 443 / 443 |

Within each bool pair, every non-comment command is identical. Complete output
pair differences are only generated wall-clock banner and `; brim_ears = 0/1`.
Outer and ear brim extrusion streams differ; actual selector has geometry
witnesses whereas the bool only has configuration-preservation coverage.
This diagnostic command projection is not the acceptance comparator.

Source ownership: PrintConfig.cpp:1751-1756 defines standalone coBool;
PrintConfig.cpp:427,1697 defines the separate enum token.
Brim.cpp:455-458 chooses btEar/btPainted from brim_type; :536-554 dispatches
make_brim_ears/make_brim_ears_auto. Native supports the no-bool-consumer source
finding, not an invented legacy alias to brim_type. Existing bool/enum exporter
owns echo preservation; no production changes needed before E2E verification.

The historical options-v242 inventory does not contain the standalone key.
The bounded test catalog explicitly appends its source-cited definition to
plans; it does not mutate the historical inventory or substitute enum coverage
for boolean coverage. All six authored values and selectors are recorded in
sweep metadata. Existing generic brim_type tests remain unchanged.

Four non-ear input-only fixtures are committed. The E0/E1 ear fixtures and
assertions are split out, not removed or accepted; the production chain must
restore both ear cases and sweep cases and land them only with GREEN. The
authored plan covers N0/N1/O0/O1 only, explicitly deferring ear coverage.
Each E2E and authored sweep case compares the entire validated API and real CLI
output against live Native with the original strict comparator/wall-clock
policy. bool1 cases also produce a live bool0 Native control. Secondary command
neutrality assertions do not replace or filter full comparisons.
Any bool-dependent Native geometry would stop admission; no such difference
was observed. Broader geometry, painted ears, holes and angle grids remain
deferred.
