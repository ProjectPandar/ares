# Latest-main Option matrix: complete execution and remaining scope audit

Actual run: clean main `acf100550e1e6db0ebef2a5202ea462713835362`, exit100.
Fresh required seeded Native cache, no option/value filters.
650 domains:621PASS/28INCOMPLETE/1FAIL;1642 cases generated,1640strictly
compared,1640PASS, **zero DIVERGENT**. Remaining cases: one application-stage
failure and one Native rejection. This is not global coverage PASS.

Independent full-stream reread of all1640 manifests confirms Native/Ares bytes
equal after removing only the entire generated-by line. Actual inputs and raw
SHA256s remain in manifests.
Generic manifest producer fields marked unknown remain unknown; explicit
run/control/native-cache receipts supply separate producer provenance. No
historical summary rows were merged and no failure was retried to PASS.

## Classification (all non-complete domains)

| Key | Raw status | Cause class | Upstream definition |
|---|---|---|---|
| adaptive_pressure_advance_bridges | INCOMPLETE | harness authored probes/context boundaries unproven | src/libslic3r/PrintConfig.cpp:2401 |
| bridge_flow | INCOMPLETE | harness authored probes/context boundaries unproven | src/libslic3r/PrintConfig.cpp:1327 |
| extruder_clearance_height_to_lid | INCOMPLETE | harness authored probes/context boundaries unproven | src/libslic3r/PrintConfig.cpp:2225 |
| extruder_clearance_height_to_rod | INCOMPLETE | harness authored probes/context boundaries unproven | src/libslic3r/PrintConfig.cpp:2215 |
| extruder_clearance_radius | INCOMPLETE | harness authored probes/context boundaries unproven | src/libslic3r/PrintConfig.cpp:2234 |
| extruder_printable_area | INCOMPLETE | harness authored probes/context boundaries unproven | src/libslic3r/PrintConfig.cpp:692 |
| filament_diameter | INCOMPLETE | harness authored probes/context boundaries unproven | src/libslic3r/PrintConfig.cpp:2606 |
| filament_flow_ratio | INCOMPLETE | harness authored probes/context boundaries unproven | src/libslic3r/PrintConfig.cpp:2315 |
| filament_map | INCOMPLETE | harness baseline-only no injection | src/libslic3r/PrintConfig.cpp:2489 |
| filament_ramming_parameters | INCOMPLETE | harness baseline-only no injection | src/libslic3r/PrintConfig.cpp:2833 |
| filament_self_index | INCOMPLETE | harness baseline-only no injection | src/libslic3r/PrintConfig.cpp:5432 |
| initial_layer_print_height | INCOMPLETE | harness authored probes/context boundaries unproven | src/libslic3r/PrintConfig.cpp:3384 |
| internal_bridge_flow | INCOMPLETE | harness authored probes/context boundaries unproven | src/libslic3r/PrintConfig.cpp:1354 |
| layer_height | INCOMPLETE | harness authored probes/context boundaries unproven | src/libslic3r/PrintConfig.cpp:771 |
| master_extruder_id | INCOMPLETE | harness baseline-only no injection | src/libslic3r/PrintConfig.cpp:5399 |
| nozzle_flush_dataset | INCOMPLETE | harness baseline-only no injection | src/libslic3r/PrintConfig.cpp:2602 |
| nozzle_height | INCOMPLETE | harness authored probes/context boundaries unproven | src/libslic3r/PrintConfig.cpp:2242 |
| other_layers_print_sequence_nums | INCOMPLETE | harness baseline-only no injection | src/libslic3r/PrintConfig.cpp:1121 |
| physical_extruder_map | INCOMPLETE | harness baseline-only no injection | src/libslic3r/PrintConfig.cpp:2495 |
| post_process | INCOMPLETE | harness baseline-only no injection | src/libslic3r/PrintConfig.cpp:5068 |
| print_compatible_printers | INCOMPLETE | harness baseline-only no injection | src/libslic3r/PrintConfig.cpp:1832 |
| print_extruder_id | INCOMPLETE | harness baseline-only no injection | src/libslic3r/PrintConfig.cpp:5405 |
| printable_height | INCOMPLETE | harness authored probes/context boundaries unproven | src/libslic3r/PrintConfig.cpp:779 |
| printer_extruder_id | INCOMPLETE | harness baseline-only no injection | src/libslic3r/PrintConfig.cpp:5385 |
| printer_technology | FAIL | schema/application context mismatch | src/libslic3r/PrintConfig.cpp:678 |
| skeleton_infill_line_width | INCOMPLETE | harness context width ceiling | src/libslic3r/PrintConfig.cpp:4074 |
| skin_infill_line_width | INCOMPLETE | harness context width ceiling | src/libslic3r/PrintConfig.cpp:4064 |
| timelapse_type | INCOMPLETE | Native rejection | src/libslic3r/PrintConfig.cpp:5879 |
| wrapping_exclude_area | INCOMPLETE | harness baseline-only no injection | src/libslic3r/PrintConfig.cpp:4120 |

13 authored-probe domains compare successfully but their endpoint/context
claims are unproved;12 baseline-only domains never inject an option value;2
width domains cover one nozzle context but not global bounds. These27 are
**harness scope gaps, not observed Ares byte discrepancies**.
`option_coverage/domains.rs:69-91` marks authored numeric probes incomplete,
`:142-156` retains baseline-only unbounded domains; sweep.rs records incomplete
when values are absent or legality is unproved. `OptionPlan` records exact
source definitions and actual values in external option-domain-plans.json.

Upstream semantic consumers bound potential future legal probes: numerical
validation PrintConfig.cpp:10337-10568 and Print.cpp:1524-1692 mean schema
endpoints alone are not always feasible inputs; extrusion-width validation
PrintConfig.cpp:10488-10517/Print.cpp:1543-1556,1662-1691 constrains the two
context-width domains. Free-form geometry/strings and mapping vectors require
individual source-owned consumers before designing multi-volume/multi-extruder
contexts. The misleading shared authored omission text for
extruder_printable_area is a domain-classification issue, not proof that its
polygon schema has numeric endpoints. No new default or capability guard is
justified by these results.

### printer_technology: application mismatch, not G-code DIVERGENT

PrintConfig.cpp:678-684 declares FFF/SLA. The FFF smoke export requested SLA but
exported effectiveFFF, producing ApplicationMismatch before Ares slicing.
OrcaSlicer.cpp:1326 captures technology;1854-1859/2100-2106 resolve imported
technology;3511-3512 default unknown toFFF;3619 restores the resolved technology
after applying options;3627-3633 rejects a truly resolved nonFFF technology.
Therefore the raw FAIL must remain FAIL; this run does **not** prove that an SLA
request reached a native SLA validator, nor establish a missing Ares SLA port.
A bounded follow-up can assert the observed requested/effective mismatch and
independently test the proper nonFFF phase, but must not silently excuse the
value or certify the FFF fixture as full SLA coverage.

### timelapse_type=1: actual Native rejection in this fixture

PrintConfig.cpp:5879-5894 declares smooth/traditional. Fresh Native result.json
for commandRnYKWl reports return_code=-102, exact error_string:
`Found G-code in unprintable area of multi-extruder printers after slicing. Please make sure the 3mf file can be successfully sliced in the latest Orca Slicer.`
stdout is `run found error, return -102, exit...`; stderr empty.
Utils.hpp:74 defines -102; OrcaSlicer.cpp:158 owns that text and6221-6227
records the post-GCodeProcessor unprintable-path result then exits. This is
**a Native-admitted enum value with an invalid emitted path in the current
fixture**, not a universally invalid timelapse value. Existing source-owned
GCodeProcessor.cpp:1866-1999/GCode.cpp:2166-2173 rejection has a public-API
E2E in gcode_printable_area.rs. The matrix fixture itself still needs an exact
fresh rejection E2E before claiming current Ares error/retained-byte parity;
this sweep stopped at Native rejection and never made that comparison.
Do not reclassify the domain PASS or add an Unsupported guard.

## Ares-fixable selection and next admission

No Ares implementation gap or concentrated remaining DIVERGENT group is
observed. Thus **no production-fix spec is invented**. Largest observed cluster
is27 harness coverage gaps, requiring individually source-bounded admission
rather than a new Ares pipeline. SLA and timelapse follow-ups are test-only
behavior/provenance work unless independent public-API runtime RED establishes
a missing cited consumer. They must preserve current raw statuses and full
failed attempts; successful controls compare complete streams.

A fresh all-printer stock matrix on latest main with first-difference
clustering is a separate follow-up; historical823/100/66/10/2 counts are not current
results. That separate run must have a fresh pinned commit/cache/artifact root
and retain crash/UB/read/vendor failures separately. No production change or
new Option test has been made by this audit.
