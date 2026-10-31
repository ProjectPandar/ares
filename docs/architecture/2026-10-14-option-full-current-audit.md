# Full option matrix audit on f637b4122: two Native modes

Actual execution: `f637b4122ead7b24c9eea52dce7a10dc042ae1cd` (W6 Grid,
stacked on W5 `22ba08ab1`). Clean detached worktree for both complete runs;
this audit adds documentation/data only and does not repair code or comparator.
Date: 2026-10-14. Status: complete execution; **not global coverage PASS**.

## Reproducible execution and raw counts

Canonical `/tmp/ares-parity-control-set-v2.sh` unchanged, SHA256
`7390a44bb957b35da12c9d1af68f0978b424f41bad6368ce21367d36b63edec7`.
Each official/seeded leg used the following command, concurrently:

```sh
env -i PATH="$PATH" HOME="$HOME" RUSTUP_TOOLCHAIN=1.99.0 \
 CARGO_BUILD_JOBS=2 NEXTEST_TEST_THREADS=2 ARES_PARITY_THREADS=2 \
 ARES_PARITY_OPTIONS=1 ARES_ORCA_CACHE_DIR="$root/cache" \
 /tmp/ares-parity-control-set-v2.sh \
 "$mode" "$repo" "$root/target" "$root/artifacts"
```

No outer taskset/pin override; canonical owns per-test CPU pinning.
No ARES_OPTION_KEYS/VALUES/PARITY_PATTERN or test filter. Independent fresh
root/cache/target per leg, no base legs and no manual Native reruns.
Preflight load1=1.02; repo/official/seeded each66 vendors/11895 referenced
files/zero missing. START/END were posted to `/tmp/ares-coord-inbox.log`.
The unchanged official wrapper SHA256 is
`1689a2ff46564cf723ad90c663051b58115920c3cce639bd2961340955cebbf3`.

| Execution | Domains | PASS | INCOMPLETE | FAIL | Generated | Raw compared | Rejected |
|---|---:|---:|---:|---:|---:|---:|---:|
| 10-09 actual acf100550e, seeded only | 650 | 621 | 28 | 1 | 1642 | 1640 | 0 |
| Fresh official | 651 | 642 | 8 | 1 | 1703 | 1688 | 0 |
| Fresh seeded | 651 | 646 | 4 | 1 | 1703 | 1702 | 0 |

Both nextest legs:1169 run/1168PASS/1FAIL/0skipped, exit100, no TIMEOUT.
The only failing test is `option_coverage::sweep::option_coverage`:
official9 domains and seeded5 domains trigger its completeness assertion.
Official nextest elapsed1577.041s, sweep1164.369s; seeded2510.400s,
sweep1987.139s. No other nextest failure or successful retry hides this result.
The raw domain FAIL remains FAIL; case DIVERGENT is reported separately below.

651 means the full registered inventory:650 non-Metadata entries plus the
live `brim_ears` bool appended by domains.rs. `from/name/version` metadata
and241 existing source-key exclusions are outside this inventory; this is
not all PrintConfig keys/all-printer/default/all legal input combinations.
PASS certifies the declared bounded endpoint/interior/fixture scope, not a
universal string/vector/numeric range. No new exclusion/filter was introduced.

## Evidence, reread and classification

Fresh root: `/tmp/worker2-option-full-103wd7e0`, legs `official` and `seeded`.
Each leg retains nextest.log/exit, artifacts/option-coverage-summary.md,
option-domain-plans.json, numeric-boundaries.json, width-domains.json,
application proofs and original case/runner/cache/export receipts.
Root preflight.json records executable/resource/file identities; process-
environment-evidence.json records actual command/env and unpinned outer CPUs.
Generic manifests' unknown producer/revision fields remain unknown; separate
run/cache receipts supply provenance rather than rewriting those fields.

Independent offline reread verified3504 option-labeled generic captures:
3462PASS/42DIVERGENT attempts, all recorded file SHA values checked. Reread
uses the existing comparator: canonical generator identity and branded host
date/time normalization only; BTT exact-byte cases remain exact. No lines,
regions, moves, decimals or mismatches were otherwise removed.
98 specialized printer-physical/ramming/dataset/legacy-brim captures also
passed complete Native/API/CLI reread; API/CLI raw equality98/98. Those
captures include standalone controls and matrix witnesses;3504/98 are
artifact-attempt counts, **not**1703 generated-case counts. Raw compared
counts above come from the matrix outcomes and include specialized witnesses.

External audit files under root/audit: audit-results.json, case-attempts-*.csv,
native-retry-attribution.json and specialized-whole-stream-reread.json.
All Native3MF/G-code/process logs remain external and are not committed.
Committed CSVs contain every domain and raw detail/reason/source definition:
[domains01](2026-10-14-option-full-data/domains-01.csv),
[domains02](2026-10-14-option-full-data/domains-02.csv),
[domains03](2026-10-14-option-full-data/domains-03.csv),
[old29](2026-10-14-option-full-data/old-29-01.csv),
[remaining and source owners](2026-10-14-option-full-data/remaining.csv).
Each CSV has at most300 data rows; this document stays below400 lines.

The harness maps any first case failure to raw domain FAIL; it has no raw
domain DIVERGENT status. Audit columns preserve raw status, strict divergent
attempts/final events, excused cases and cause classes separately.
Reasons here: COMPLETE_DECLARED_SCOPE; HARNESS_BASELINE_ONLY/context scope;
APPLICATION_REQUEST_EFFECTIVE_MISMATCH; ORACLE_NONDETERMINISM_EXCUSED.
An incomplete scope or oracle instability is not evidence of an Ares runtime
implementation defect. All651 fresh summary keys match plans; the earlier
numeric-report0/0 placeholder is excluded, with no prior summary rows merged.

## Difference from the 10-09 audit

The historical audit was seeded-only, so the like-mode comparison is seeded:
621 prior PASS remain PASS;24 prior INCOMPLETE become bounded-scope PASS;
4 remain INCOMPLETE;1 prior FAIL remains FAIL. New `brim_ears` is PASS in
both modes. Thus seeded PASS increases25 including the new domain, and
INCOMPLETE falls24. Official versus the old seeded table is +1 domain,
+21PASS/-20INCOMPLETE/unchanged1FAIL; this is a cross-mode difference. There is no newly failing old seeded PASS domain.

Official closes the same24 old INCOMPLETE domains. Its four additional
INCOMPLETE fuzzy keys were old seeded PASS keys; a cross-mode comparison
cannot establish a new code regression. Fixture/plan scopes changed since
10-09; closure is within the source-owned finite contexts now declared.
For timelapse, the admitted branch fixture is new and the original smooth
-102 rejection remains a separate passing negative control. The old invalid
emitted path was not silently made legal or excluded from error evidence.

Full old29 comparison; definition lines are in libslic3r/PrintConfig.cpp:

| Key | Line | Old seeded | Fresh official | Fresh seeded |
|---|---:|---|---|---|
| adaptive_pressure_advance_bridges | 2401 | INCOMPLETE | PASS | PASS |
| bridge_flow | 1327 | INCOMPLETE | PASS | PASS |
| extruder_clearance_height_to_lid | 2225 | INCOMPLETE | PASS | PASS |
| extruder_clearance_height_to_rod | 2215 | INCOMPLETE | PASS | PASS |
| extruder_clearance_radius | 2234 | INCOMPLETE | PASS | PASS |
| extruder_printable_area | 692 | INCOMPLETE | PASS | PASS |
| filament_diameter | 2606 | INCOMPLETE | PASS | PASS |
| filament_flow_ratio | 2315 | INCOMPLETE | PASS | PASS |
| filament_map | 2489 | INCOMPLETE | PASS | PASS |
| filament_ramming_parameters | 2833 | INCOMPLETE | PASS | PASS |
| filament_self_index | 5432 | INCOMPLETE | INCOMPLETE | INCOMPLETE |
| initial_layer_print_height | 3384 | INCOMPLETE | PASS | PASS |
| internal_bridge_flow | 1354 | INCOMPLETE | PASS | PASS |
| layer_height | 771 | INCOMPLETE | PASS | PASS |
| master_extruder_id | 5399 | INCOMPLETE | INCOMPLETE | INCOMPLETE |
| nozzle_flush_dataset | 2602 | INCOMPLETE | PASS | PASS |
| nozzle_height | 2242 | INCOMPLETE | PASS | PASS |
| other_layers_print_sequence_nums | 1121 | INCOMPLETE | PASS | PASS |
| physical_extruder_map | 2495 | INCOMPLETE | PASS | PASS |
| post_process | 5068 | INCOMPLETE | INCOMPLETE | INCOMPLETE |
| print_compatible_printers | 1832 | INCOMPLETE | PASS | PASS |
| print_extruder_id | 5405 | INCOMPLETE | INCOMPLETE | INCOMPLETE |
| printable_height | 779 | INCOMPLETE | PASS | PASS |
| printer_extruder_id | 5385 | INCOMPLETE | PASS | PASS |
| printer_technology | 678 | FAIL | FAIL | FAIL |
| skeleton_infill_line_width | 4074 | INCOMPLETE | PASS | PASS |
| skin_infill_line_width | 4064 | INCOMPLETE | PASS | PASS |
| timelapse_type | 5879 | INCOMPLETE | PASS | PASS |
| wrapping_exclude_area | 4120 | INCOMPLETE | PASS | PASS |

## Remaining union and source ownership

9 distinct keys remain non-PASS in at least one mode. Definition lines below
are PrintConfig.cpp; semantic owners are under src/libslic3r. Generic config
footer retention is not substituted for actual consumer ownership.

| Key | Definition | Official | Seeded | Owning file/function/lines |
|---|---:|---|---|---|
| filament_self_index | 5432 | INCOMPLETE | INCOMPLETE | PresetBundle.cpp::PresetBundle::construct_full_config:68-195; PrintApply.cpp::Print::apply:1113-1172; PrintConfig.cpp::DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments:9805+ |
| fuzzy_skin_noise_type | 3611 | INCOMPLETE | PASS | Feature/FuzzySkin/FuzzySkin.cpp::random_value:26-31, get_noise_module:41-65, fuzzy_polyline:294-342 and fuzzy_extrusion_line:345-412 |
| fuzzy_skin_octaves | 3647 | INCOMPLETE | PASS | Feature/FuzzySkin/FuzzySkin.cpp::random_value:26-31, get_noise_module:41-65, fuzzy_polyline:294-342 and fuzzy_extrusion_line:345-412 |
| fuzzy_skin_persistence | 3656 | INCOMPLETE | PASS | Feature/FuzzySkin/FuzzySkin.cpp::random_value:26-31, get_noise_module:41-65, fuzzy_polyline:294-342 and fuzzy_extrusion_line:345-412 |
| fuzzy_skin_scale | 3637 | INCOMPLETE | PASS | Feature/FuzzySkin/FuzzySkin.cpp::random_value:26-31, get_noise_module:41-65, fuzzy_polyline:294-342 and fuzzy_extrusion_line:345-412 |
| master_extruder_id | 5399 | INCOMPLETE | INCOMPLETE | GCode/ToolOrdering.cpp::ToolOrdering::get_recommended_filament_maps:1107-1215; GCode.hpp::WipeTowerIntegration constructor:77-104 |
| post_process | 5068 | INCOMPLETE | INCOMPLETE | PrintConfig.cpp::PrintConfigDef::init_fff_params:5068-5079; GCode/PostProcessor.cpp::run_post_process_scripts:238-377 |
| print_extruder_id | 5405 | INCOMPLETE | INCOMPLETE | PrintApply.cpp::Print::apply:1113-1169; PrintConfig.cpp::DynamicPrintConfig::update_values_to_printer_extruders:9634+; ParameterUtils.cpp::get_index_for_extruder_parameter:46+ |
| printer_technology | 678 | FAIL | FAIL | PrintConfig.cpp::PrintConfigDef::init_common_params:674-684 and DynamicPrintConfig::validate:8866-8883 |

**Four baseline-only domains:** filament_self_index, master_extruder_id,
print_extruder_id and post_process each have one compared baseline with
value=None, no injected value, and unproved declared scope. They stay
INCOMPLETE in both modes. Baseline byte equality is not vector/index/script
coverage. See the remaining CSV for per-key deferred behavior.

**printer_technology:** both original requested machine JSON files contain
SLA. Export receipts report successful Native Export, but exported3MF
Metadata/project_settings.config contains FFF. Exact requested preset,
exported project and receipts were copied to root/audit-failures/
printer-technology/{official,seeded} with SHA attribution in root/
printer-technology-{mode}-attribution.json. The failure occurs at
Application/ApplicationMismatch before Native SLA slicing or Ares slicing.
It reproduces the old harness application mismatch, not a missing SLA port.
PrintConfig.cpp:8866-8883 validates FFF and returns no SLA errors; CLI resolved
technology/import handling is OrcaSlicer.cpp:1326,1854-1859,2100-2106,
3511-3512,3619,3627-3633. Neither stage is inferred from the enum definition.

**Official fuzzy:** noise_type has5 excused cases (billow/classic/perlin/
ridgedmulti/voronoi); octaves/persistence/scale have3 each. Existing canonical
retry budget produced42 retained DIVERGENT attempts over14 same-input groups.
Every group has multiple normalized Native SHA values and one Ares SHA;
no new Native runs were used to establish this observation. Seeded matches
all these cases and its domain statuses stay PASS. Native variation proves
instability on these exact inputs, not that Ares's stochastic semantics are
correct. Official remains INCOMPLETE. Zero unexcused strict divergence was
observed; raw FAIL is still the application mismatch above.
Some raw fuzzy details only say required cases not all compared (the zero-
compared branch precedes the excused-detail branch); cause classification
uses recorded ORACLE_NONDETERMINISTIC events and retained streams, without
changing that raw text or the comparator.

## Reconcile the 10-11 gap ledger on this exact snapshot

The [10-11 gap audit](option-coverage-gap-2026-10-11.md) was a static
case-existence audit, not a runtime verdict. Its25 inventory-external keys
include its5 strict no-case (N) keys; those are **not another5 keys**.
We additionally reconcile its5 baseline-only (B) keys:30 distinct rows in
[the current gap ledger](2026-10-14-option-full-data/gap-2026-10-11-current.csv).
Every row includes definition, current membership/plan/case count, raw modes,
source owner, evidence and proposed action. NOT_IN_MATRIX is accounting,
not PASS, FAIL or a new exclusion introduced by this audit.

The source JSON still has650 non-Metadata keys. All25 old external keys
remain absent from that JSON; domains.rs now appends only brim_ears, so
24 still lack a matrix plan. The old675 live-FFF registration set therefore
has651 matrix domains and24 outside:17 metadata/network/preset-selection,
3 authored companions,4 old N keys. No inventory or exclusion file changed.

| Old N key / PrintConfig.cpp | Current evidence | Recommendation / source boundary |
|---|---|---|
| brim_ears :1751 | Appended6-case bool × selector plan, PASS/PASS | Domain correction already implemented: bool echo versus brim_type geometry; Brim.cpp:455-458,536-554. No new ear algorithm inferred. |
| tree_support_with_infill :6550 | Still no plan | Evidence-based exclusion from active motion domain; handle_legacy:8263 discards it; Support/TreeSupport.cpp:653 derives with_infill from support_base_pattern. A legacy-ignore control is separate work. |
| enable_filament_dynamic_map :2518 | Still no plan | Define separate plate serialization domain; Format/bbs_3mf.cpp:4554-4558 imports it. No exact-key live GCode optimizer established. |
| inherits :4183 | Still no plan; load_settings_3mf_inherits and profile_inherits_group tests PASS both legs | Define preset composition/admission domain, not arbitrary strings; Preset.cpp:1783-1786,2148; PresetBundle.cpp::construct_full_config:97-236. |
| inherits_group :4193 | Still no plan; profile_inherits_group slot test PASS both legs | Define derived process/filament/machine slot domain; PresetBundle.cpp:97-236; OrcaSlicer.cpp:2816-2817,2840-2842,2983-2985,3163; legacy rename PrintConfig.cpp:8100. |

Of the old5 B keys, printer_extruder_id now has3 declared cases,
physical_extruder_map3, nozzle_flush_dataset6 and filament_ramming_parameters7;
all4 are bounded-scope PASS in both modes. master_extruder_id still has one
None baseline and remains INCOMPLETE. Their definition/consumer citations
and deferred dimensions are per-key in the gap CSV; existing test presence
is not substituted for matrix integration or global vector coverage.

The remaining20 old external keys retain their original accounting:

- Three companions: extruder:2200 is normalized into filament selectors by
  DynamicPrintConfig::normalize_fdm_1:8619-8632; first_layer_sequence_choice
  :1125 and other_layers_sequence_choice:1136 are GUI choices mapped by
  GUI/Tab.cpp:3082-3093 and OptionsGroup.cpp:1070-1079. Existing plate fixtures
  exercise effective sequences, not exhaustive coverage of those exact enums.
- Seven host/auth keys at PrintConfig.cpp:836-900 remain network/upload scope;
  preset_names:909 and preset_name:929 remain physical-printer identity/legacy
  compatibility slots. Proposed evidence-based exclusion is from the direct
  FFF motion sweep, not a claim they are tested or irrelevant everywhere.
- Six compatible_* keys at :1781-1825 have profile-selection/aggregation
  behavior: Preset.cpp::is_compatible_with_print:771-792 and
  is_compatible_with_printer:810-842; PresetBundle.cpp:97-236. Correct their
  domain to preset resolution; do not label them universally inert metadata.
- different_settings_to_system:1828 retains a separate bookkeeping_export
  PASS test in each leg; PresetBundle.cpp:104-109,203-236 collects settings
  metadata (its final aggregation line is commented). has_filament_switcher
  :2524 is a plate descriptor imported at bbs_3mf.cpp:4560-4564 and exported
  :8219-8220. Neither earns a matrix PASS by exclusion.

Case-existence and domain completion also differ for post_process and
print_extruder_id: their existing3 CLI admission tests and6 identity tests
respectively PASS in each leg, yet the matrix still generates only their
None baselines. Filament self-index has active companion injections too.
Next proposals should integrate or accurately account these bounded tests,
not duplicate them or infer an absent implementation from INCOMPLETE.

## Source-cited slice inputs for review, not implementation approval

1. **Technology application-phase fixture (test-only):** own
   PrintConfigDef::init_common_params and DynamicPrintConfig::validate,
   plus the CLI import/resolution sites above. Proposed Rust test boundary:
   orca_parity/option_coverage/printer_technology.rs and runner/application.rs.
   Include an exact requested/effective mismatch control and a genuinely
   resolved non-FFF admission witness before claiming coverage. Defer SLA
   slicing and all production changes without an admitted public RED.
2. **Master selector finite scope:** ToolOrdering::get_recommended_filament_maps
   :1107-1215 and WipeTowerIntegration constructor:77-104. Target existing
   option_coverage/tool_domains.rs/tool_context. First establish a public
   two-extruder Native mapping/template or motion witness, exact application,
   paired control and complete API/CLI parity. Defer automatic grouping,
   invalid-index policy and arbitrary printer contexts; stop if no witness.
3. **Print variant identity accounting:** Print::apply:1113-1169 and
   DynamicPrintConfig::update_values_to_printer_extruders:9634+. Existing
   option_coverage/print_extruder_identity.rs has six passing finite tests;
   integrate their admitted identity cases into matrix accounting first.
   Add new cases only for a demonstrated uncovered consumer dimension. Defer global vector shapes and
   printer/filament identity redesign; a runtime disparity needs its own RED.
4. **Filament self-index expansion:** PresetBundle::construct_full_config
   :68-195, Print::apply:1172 and update_values_to_printer_extruders_for_multiple_filaments
   :9805+. Proposed option_coverage/filament_self_index.rs with finite
   multi-filament/variant inputs, active consumer proof and raw full outputs.
   Defer arbitrary indices/vector shapes and production changes without RED.
5. **post_process bounded admission:** continue the existing approved
   [CLI boundary spec](option-post-process-cli-boundary-spec.md), owned by
   PrintConfigDef::init_fff_params:5068-5079 and
   GCode/PostProcessor::run_post_process_scripts:238-377. Existing test destination
   option_coverage/post_process.rs has three passing embedded-vector CLI
   admission/no-execution witnesses; reconcile that bounded admission domain
   with the matrix baseline. Arbitrary native script execution stays deferred.
   The CLI external runner call at OrcaSlicer.cpp:6247-6248 is commented out;
   do not confuse it with internal GCodeProcessor post-processing or add
   filesystem/process behavior to core/WASM.

Recommended action classes for the remaining9 keys: add legal non-default
bounded consumer witnesses for filament_self_index/master_extruder_id;
domain/accounting correction for the already-authored print_extruder_id and
post_process tests; application-phase/domain correction for printer_technology
while preserving SLA FAIL; official stochastic-domain/provenance correction
for each of the four fuzzy keys, preserving official INCOMPLETE and seeded
PASS. None is an evidence-free exclusion or a weakened byte comparator.

The four official fuzzy gaps call for retained source/seed/producer evidence,
not an invented Ares production fix or weaker comparator. Their actual
owners are Feature/FuzzySkin/FuzzySkin.cpp random_value/get_noise_module and
fuzzy_polyline/fuzzy_extrusion_line. No unexcused Ares byte discrepancy was
established here, so the proposed list starts with source-owned witness and
harness admission slices. Each slice requires separate SPEC review before
implementation; this audit authorizes none and preserves all raw failures.

Source PrintConfig.cpp SHA256:
`f635101adb6da3ee657656c6fcaa9449871b7c17446b5e2eb59b560a0eb836aa`,
matching Orca source8500fcdccaa10b5099ac20d252af3a7c560046f1.
Existing upstream exclusion file SHA256:
`597becfd3d78b8fb0f94a0d7bb25659f6f626e65461f503bc4f3c554b3972b97`.
