# Authorized retirement of square-ear scaffold tests

Coordinator explicitly classified five public-adapter failures at actual
89527942c as obsolete scaffold assertions, not regressions, and authorized
deleting these five tests in GREEN. No new unit tests are added. Live Native
E0/E1 and six-case authored ear sweep retain entire API/CLI comparisons,
bool neutrality and ear-versus-outer geometry assertions.

| Removed test | Conflict with source-owned automatic ears |
|---|---|
| brim_ears_generate_local_corner_loops_on_first_layer | Width0.4/spacing0.4 expected four square loops and exact four square vertices; Brim.cpp:449-450,529 quantizes width in spacing pairs, :540 computes radius minus gap/spacing, :313-315 returns no ears for nonpositive size; :337-343 uses24-side circular pattern, not four vertices |
| brim_ears_use_width_for_multiple_corner_loops | Expected eight independent square loops for width0.8/gap0.1; Brim.cpp:337-352 produces circular areas, :544 subtracts footprint and :813-839 derives/clips ordered offset loops, not one square per corner per width step |
| brim_ears_max_angle_filters_contour_vertices_by_corner_angle | Triangle width0.4 expected one square at apex, but quantized radius is nonpositive (:449-450,529,540,313-315); candidates follow Polygon.cpp:275-317 and circle source :337-343 rather than a hardcoded apex square |
| brim_ears_max_angle_does_not_treat_reflex_vertices_as_sharp_corners | Concave contour width0.4 expected two square point arrays despite nonpositive radius; same source width/radius contract, signed convex/concave predicates at Polygon.cpp:310-317, circular area clipping at Brim.cpp:540-544 |
| brim_ears_detection_length_simplifies_small_contour_deviations | Width0.4 expected raw square-loop count greater than simplified and exact four; upstream nonpositive radius yields no ear pattern before corner detection, and :321-329 uses closed Douglas-Peucker with >4-point admission, not legacy farthest-pair ring split |

Coordinator additionally authorized these pipeline scaffold retirements after
the interrupted workspace exposed them (raw log preserved):

| Pipeline test | Retired square assertion / retained control |
|---|---|
| brim_ears_reach_print_paths_and_gcode | Removed four-loop counts, literal four-square-vertex BRIM text and square-dependent presence assertions for width0.4/spacing0.4. Brim.cpp:449-450 quantizes width in spacing pairs; :310-352 rejects nonpositive size and constructs24-side circles. Original painted empty assertion/input remain as painted_brims_remain_empty_in_pipeline, not a new unit test. |
| brim_ears_detection_length_reaches_pipeline_and_gcode | Removed square loop/BRIM G-code count inequalities and its exclusively used brim_print_path_count helper. Width0.4 quantizes to0 at :449-450, so :313-315 returns no ears before :321-329 detection-length decimation. Original max-angle-zero pipeline assertions remain untouched. |

Other brims tests, including zero-angle, non-ear, layer preservation, disabled
modes, invalid width, holes, combine/EFC tests remain intact. No allowance,
ignored test, mode disable, legacy fallback, numerical tolerance, filtered
acceptance or diminished E0/E1 comparison accompanies this retirement.

Original failed15PASS/5FAIL log remains
`/tmp/worker3-brim-auto-engineering-v5-public-adapter.log`; actual execution
89527942c is never attributed to later GREEN hashes. These deletions belong
to the responsible GREEN source replacement commit, not RED or independent
main inventory repair. Production requires wholeoption_coverage both modes,
strict/fmt/wasm32, clean workspace and exclusive full four legs before merge.
