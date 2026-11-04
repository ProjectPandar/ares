# PrintConfig option-coverage gaps — 2026-10-11

Method: offline source/test audit; no Native run, compilation, stock resweep,
or new parity verdict. Ares snapshot: `e6e9af34d74753ac04d65c093efdc1b1280d66cf`.
Scope: **live FFF print/process, printer and filament input definitions** in
`OrcaSlicer/src/libslic3r/PrintConfig.cpp`; test boundary:
`crates/ares-cli/tests/orca_parity/option_coverage/` (including children).

## Non-negotiable interpretation

A missing literal grep hit is NOT a missing test: the inventory-driven
`domains.rs::load/plan` + `sweep.rs::option_coverage` generates
cases without naming each key in Rust. Conversely a baseline-only plan with
`value: None` compares the default fixture, not the option's behavior.
Keep these separate:

- **N: no case**: no inventory plan and no authored/companion case in the
  requested directory after exclusions.
- **B: no value-bearing case**: inventory plan exists but only a None-valued
  baseline, with no authored option case in this directory. NOT zero tests.
- **C: case exists**: generated value plan, authored case, or verified fixture
  companion. Existence does NOT imply complete domain coverage, execution,
  legal input, enabled downstream consumer, or PASS.
- An outside-directory case prevents a project-wide “untested” claim, but
  does not close the requested directory's authoring/accounting gap.

No acceptance changes: ARD-0025 and strict whole-output comparisons remain.
Existing Ares scaffolds are temporary upstream compatibility shells, not an
independent slicing pipeline or a rationale to create new consumers.

## Exhaustive registration reconciliation

Source fingerprint (actual local source, not an inferred upstream Git SHA):

- PrintConfig.cpp SHA256:
  `f635101adb6da3ee657656c6fcaa9449871b7c17446b5e2eb59b560a0eb836aa`
- `tests/ksr_fdmtest_v4/stock_project_options.json` SHA256:
  `33c99ee71594ed7f80b44abc3007df8e9ae4ec0800411e3b5dba500f47fd085b`

Registration boundary: `init_common_params():674` and
`init_fff_params():934`, ending before `init_extruder_option_keys():7330`.
Strip comments before finding `this->add`, retain string literals and original
line positions; never count commented registrations. Expand the axis loop
`:4502-4568` to x/y/z/e and `filament_extruder_override_keys:63-84` via
`add_nullable:7288-7320`. Result: **675 unique live FFF inputs**:
647 literal keys + 12 generated axis keys + 16 generated nullable keys.

Reconcile all keys with the inventory, not only grep matches:

| Partition | Keys | Meaning |
| --- | ---: | --- |
| Inventory intersection | 650 | Dynamic sweep plans (including baseline-only) |
| FFF registrations absent from inventory | 25 | Individually classified below |
| Inventory outside live FFF registration set | 3 | Metadata `from`, `name`, `version` |

The inventory has653 rows: process352/printer132/filament122/residual47.
`raw_scope=residual` is a harness classification, not proof of no consumer.
`domains.rs` filters only `option_type=Metadata` before generating650 plans.

Durable per-key ledger (all675 keys, alphabetically split to keep files<400LOC):

- [catalog1](option-coverage-gap-2026-10-11-catalog-1.csv):338 keys.
- [catalog2](option-coverage-gap-2026-10-11-catalog-2.csv):337 keys.

Ledger states:642 `sweep-value-plan`,5 `sweep-baseline-only`,6
`authored/companion`,17 explicit extra-registration metadata/network exclusions,
5 `no-case`. The642 value plans include already-covered metadata: their state
only records existence, not eligibility for new behavioral work. This audit
excludes GUI/network/model identity from the **gap list**, not from the existing
harness or upstream registrations. No options removed or acceptance excused.

`C` numeric/bool/enum generation: `domains.rs:154-199`,
`numeric_domains.rs:6-168`; generated definition resolution:
`bounds.rs::definition_block`. Authored strings/points:
`domains/probes.rs::authored_probes`. This source inspection covers the entire
registered set; it does not run650 domains or certify their legality.

## N — no case in requested directory (5)

All definition paths below are `src/libslic3r/PrintConfig.cpp`; consumer paths
are under `OrcaSlicer/` unless noted. No missing Perimeter/Fill scalar family
was discovered: those inputs already enter the generic sweep. They may still
be incomplete, inert or unexecuted, which is a separate audit.

| Consumer group | Option / definition | Upstream owning boundary | Disposition |
| --- | --- | --- | --- |
| Brim / legacy setting | `brim_ears` :1751, coBool | `Brim.cpp:455-460,536-554` uses **brim_type**, not this standalone bool | Registered but no live exact-key slicing reader found; do not invent brim behavior |
| Support / obsolete setting | `tree_support_with_infill` :6550, coBool | `PrintConfig.cpp::handle_legacy:8263` discards key | Legacy-ignore admission gap, NOT missing active support-fill algorithm |
| Project/plate serialization | `enable_filament_dynamic_map` :2518, coBool | `Format/bbs_3mf.cpp:376,4554-4556` plate metadata importer | No live exact-key ToolOrdering/GCode consumption found; report storage gap, no dynamic optimizer design |
| Preset resolution | `inherits` :4183, coString | `Preset.hpp:319-334`; `Preset.cpp:1783-1786,2148`; `PresetBundle.cpp:203,236` | Behavior-bearing composition, not GUI-only metadata; separate preset-inheritance slice, do not duplicate |
| Preset aggregation | `inherits_group` :4193, coStrings | `PresetBundle.cpp:97-236,2006`; legacy rename `PrintConfig.cpp:8100` | Same preset-inheritance boundary; no new implementation claim here |

`brim_ears` also appears as an **enum wire value** for `brim_type` (:427,1697).
That does not exercise the separately registered coBool. A lexical grep alone
would conflate them. The CSV marks that bool N, not the working brim_type case.
`tree_support_with_infill` being registered and ignored is recorded explicitly,
not treated as a GUI exclusion or a new fill implementation requirement.

## B — only generic baseline, no injected case here (5)

These have a baseline comparison, so they do NOT satisfy the literal “no test
at all” criterion. They are the meaningful next coverage-authoring gaps.
Unbounded coInts: `numeric_domains.rs:151-168`; unbounded coStrings:
`domains.rs:190-199`. None of the five is in `domains/probes.rs`.

| Consumer group | Option / definition / scope | Upstream owning behavior | Evidence outside requested directory |
| --- | --- | --- | --- |
| GCode tool identity / parameters | `printer_extruder_id` :5385, printer | `PrintConfig.cpp::get_index_for_extruder:9004-9055`, `update_values_to_printer_extruders:9634ff`; `PrintApply.cpp:1166-1167`; `ParameterUtils.cpp:46-78` | `runner/generated_printer_ids.rs`, `runner/loaded_config_probes.rs` already test bounded identities; not project-wide untested |
| GCode physical identity / templates | `physical_extruder_map` :2495, printer | `GCode.cpp:2833-2847` first-tools permutation, :4679/:5162-5163/:5222-5223 runtime placeholders; `GCode/GCodeProcessor.cpp:1366,1381,2050` | No literal key-specific CLI Rust case found; defaults/stock presets still exercise it incidentally; core unit tests are outside this audit |
| ToolOrdering actual-used grouping | `master_extruder_id` :5399, printer | `ToolOrdering.cpp:1278-1316`; `Print.cpp:3166-3200` | Newly merged `cli_master_extruder_id.rs` A1/A2; **closed bounded slice**, no duplicate rewrite proposed |
| CLI variant selection → purge | `nozzle_flush_dataset` :2602, printer | `OrcaSlicer.cpp:3436-3463,3487` selects predictor dataset per nozzle; PrintApply materializes variant | `runner/x2d_flush_matrix.rs:186-236` active probes already exist; not a new DIVERGENT |
| Wipe tower / SEMM ramming | `filament_ramming_parameters` :2833, filament | `GCode/WipeTower2.cpp:1378-1390` profile parser under m_semm; **WipeTower.cpp:1591-1601 copy is #if0** | `runner/filament_map_ramming.rs:138-235` default/short + swap Native whole-output case already exists |

Consumer grouping is source-owned. Neither `physical_extruder_map` nor
`printer_extruder_id` is excluded as “machine metadata”: they change runtime
placeholder values and select machine parameter vectors used in generated
motion. 0-based physical map and 1-based printer IDs are different concepts,
not substitutes for each other or for the 1-based filament_map.

## False positives explicitly discharged

Eight possible baseline-only plans were examined. Three already have concrete
cases here and therefore are NOT B:

| Option | Existing directory case / reason |
| --- | --- |
| `post_process` :5068 | `post_process.rs:20-30` empty vector, empty-string element, nonempty script; real CLI accept/reject and script-must-not-execute proof; wired by `orca_parity.rs:41-42` |
| `print_extruder_id` :5405 | `print_extruder_identity.rs` six whole-output identity cases; parser/fixture adaptation is not a gap |
| `filament_self_index` :5432 | `tool_context.rs:108-126` injects `[1,2]`, verifies effective settings and compares four Manual maps; companion coverage exists, but is not exhaustive identity-domain coverage |

Three additional non-inventory keys have concrete companion inputs:

- `extruder` :2200: object metadata in `extruder_printable_area.rs` and
  `timelapse_coverage.rs`, not a free generic numeric sweep.
- `first_layer_sequence_choice` :1125 and `other_layers_sequence_choice` :1136:
  `tool_sequence_context.rs` wires custom plate selections. They have fixture
  coverage, not a proved exhaustive enum domain; no N claim.

## Exclusions (remaining17 non-inventory registrations)

These are exclusions from requested gap work, not acceptance suppressions:

| Group | Registered keys | Source / reason |
| --- | --- | --- |
| Network/upload (7) | `print_host`, `print_host_webui`, `printhost_apikey`, `printhost_port`, `printhost_cafile`, `printhost_user`, `printhost_password` | PrintConfig.cpp:836-900; network transport/auth, no slicing behavior |
| Preset display identity (2) | `preset_names`, `preset_name` | :909,929; UI preset selection/display, not effective parameter payload |
| Compatibility/UI dependency expressions (6) | `compatible_printers`, `compatible_printers_condition`, `compatible_prints`, `compatible_prints_condition`, `compatible_machine_expression_group`, `compatible_process_expression_group` | :1781-1825; profile-selection compatibility metadata; distinct from behavior-bearing inherits resolution and from existing print_compatible_printers slicing rejection |
| Settings-diff metadata (1) | `different_settings_to_system` | :1828; existing `bookkeeping_export.rs` already supplies export case; no geometry/emit port proposed |
| Hardware/plate descriptor (1) | `has_filament_switcher` | :2524; `bbs_3mf.cpp:4562-4564,8219-8220` stores hardware descriptor; no exact-key libslic3r runtime reader found |

Further non-input exclusions: SLA definitions begin at7395; CLI action/
transform flags begin at10588; custom G-code placeholders at11407 are outputs,
not configurable print/printer/filament inputs. `adaptive_layer_height`,
`spaghetti_detector`, and the old coInts `filament_extruder_id` registration
at5419 are commented out. The later coInt template variable with the same name
at11424 is not an input option. Do not inflate gap counts with any of them.

## Next slice priority and scope decision

The only strict N keys with active nontrivial logic are the inherits
composition (a separate slice); remaining N keys are storage/legacy slots. It would be incorrect
to manufacture a new GCode algorithm to satisfy a “largest N group” request.
Propose **GCode printer-parameter/physical-identity coverage** from
B (`printer_extruder_id`, `physical_extruder_map`) as the highest-impact
remaining directory-local authoring gap: variant selection affects per-tool
retraction/motion across layers, while physical identity exposes startup,
layer/timelapse and processor contexts. This is a source-based priority,
**not a measured regression or an asserted Native divergence**.

Next is SEMM ramming sweep integration (already tested outside this directory);
then nozzle dataset sweep integration (also already tested); do not reopen
master A1/A2 or inherits. Native crash clusters remain separate.

Chosen coverage spec:
[printer/physical identity](option-printer-physical-identity-coverage-spec.md).
Both proposed input families remain **unadmitted** until Native runs. No coverage cases, core changes, experiments, heavyweight gates or
Native build were executed for this audit.
