# Option slices 2 and 4: bounded master and filament identity witnesses

Status: Proposed.
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Evidence and ownership

The option audit `96328f269` scanned both modes:
651 domains, official642 PASS/8 INCOMPLETE/1 FAIL, seeded646/4/1.
`master_extruder_id` and `filament_self_index` each have one None baseline,
zero injected values, and INCOMPLETE in both modes. Their schema has no
numeric range proving arbitrary integer values safe. The baseline result
is not an active consumer witness or an unbounded-domain completeness proof.
Source ledger: `2026-10-14-option-full-current-audit.md`, Source-cited slice
inputs 2/4.
This document proposes bounded harness coverage, not new core functionality.

This slice covers only these two keys. print_extruder_id, post_process,
fuzzy and printer_technology coverage, inventory/exclusions, runner/cache, CI
and production code are out of scope.
The current Ares mapping/variant machinery is the existing temporary shell
around the cited libslic3r consumers; this slice measures it, not redesigns it.

## Slice 2: master_extruder_id

| Upstream boundary | Behavior used by the witness |
|---|---|
| PrintConfig.cpp:5399-5403 | coInt, default1, default extruder for filament placement; not a schema max2. |
| GCode/ToolOrdering.cpp:1107-1154 | Build recommendation inputs; subtract1 from master, initialize the map. |
| ToolOrdering.cpp:1165-1207 | Two-nozzle BBL: collect used TPU and select its explicit grouping branch. |
| FilamentGroup.cpp:40-49 | TPU goes to master; other filaments to 1-master (zero-based). |
| ToolOrdering.cpp:1208-1213 | Non-BBL multi-nozzle replaces the initial map with identity; unsuitable as the active master witness. |
| ToolOrdering.cpp:1288-1308 | Auto mode calls recommendation, restores one-based maps, updates Print and checks printability; Manual bypasses recommendation. |
| Print.cpp:3166-3175 | Changed recommended maps update config and rematerialize filament variants. |
| GCode.hpp:77-107 | WipeTowerIntegration initially uses master offset, then replaces each entry from filament_map. This initial value alone does not prove an observable master effect. |
| PrintConfig.cpp:2502-2515 | Auto For Flush / Auto For Match / Manual are distinct wire modes. |

Rust production boundaries, read-only: options/filament_map_recommend.rs,
options/project_variants, and existing tool-ordering/G-code consumers.
The already ported apply_used_tpu_map is the candidate consumer; no extension
of FilamentGroup optimization, wipe tower or physical-tool emission is included.

Proposed public context: existing BBL H2D two0.4mm Standard-nozzle input,
one by-layer cube10, two real used logical filaments (TPU wall1, PLA infill2),
ordinary positive density/layer height, support/raft/brim/timelapse disabled,
no prime tower. Use coherent existing printer variant/physical-map metadata.
Physical printability masks and filament metadata must be explicit and valid;
Native admission, not this proposal, establishes their legality.
Both filaments retain real stock start/change templates and distinct valid
material temperatures; mapped tool/retraction markers may follow the existing
G4 template witness. No marker containing only master itself counts as activity.

Finite cases, with geometry and every dependency fixed within each pair:

| Case | Master | Mode | Expected active or negative-control observation |
|---|---:|---|---|
| M1 | 1 | Auto For Flush | TPU1/PLA2 map becomes [1,2]. |
| M2 | 2 | Auto For Flush | Non-default master swaps map to [2,1]. |
| MC1 | 1 | Manual [1,2] | Fixed map bypasses master recommendation. |
| MC2 | 2 | Manual [1,2] | Same mapped tool/motion as MC1; key echo alone is not a witness. |

Before accounting changes, Native official and seeded must accept M2,
retain requested master2, and expose the swapped effective map plus a real
mapped template/tool/motion change against M1. Record actual first command
change outside CONFIG_BLOCK. Native refusal, substitution, identical active
output, or a deterministic Ares mismatch is STOP: report it, do not replace
TPU with an unrelated context or port production behavior here.
Manual controls prove bypass rather than all-mode master completeness.
Wipe-tower offset effects, Auto For Match, non-TPU grouping/AMS optimization,
non-BBL printers, invalid0/3 indices, SEMM and arbitrary printer shapes defer.

Harness destinations: new option_coverage/master_extruder_id.rs and a
key-prefixed input helper if needed; narrow registration/domain/sweep hooks.
Reuse OrcaRunner export/slice, project and plate map-mode binding,
application proof, current G4 conventions, public validated API and CLI.
Existing tool_domains/tool_context consumers must remain unchanged.

## Slice 4: filament_self_index

| Upstream boundary | Behavior used by the witness |
|---|---|
| PrintConfig.cpp:5432-5437 | coInts default[1], internal preset field, nocli. It is not an arbitrary command-line numeric setting. |
| PresetBundle.cpp:68-94,122-135 | Compose presets/map sizes; single-filament identity resized to its variant count. |
| PresetBundle.cpp:136-195 | Multi-filament variant vectors append; when apply_extruder=false, assign each expanded slot its one-based logical filament ID. |
| PrintApply.cpp:1164-1172 | Print::apply selects printer/process variants then materializes filament options using filament_self_index. Audit shorthand Print.cpp:1172 actually refers to this split source file. |
| PrintConfig.cpp:9805-9839 | For each logical filament, use its mapped physical nozzle/type/volume and get_index_for_extruder(logical_id,...). |
| PrintConfig.cpp:9004-9054 | Match variant plus complete explicit ID vector; first matching slot wins. |
| PrintConfig.cpp:9892-9910 | coInts materialize the selected slot for each filament. |
| PrintConfig.cpp:8399-8400 | Initial/normal nozzle_temperature belong to filament_options_with_variant. |
| GCode.cpp:870-871,1217-1220 | Selected filament temperatures drive emitted change/temperature commands. |

Rust production boundaries, read-only: profiles/composition.rs,
options/project_variants/{index,filament,select}, project profile-loading and
current emission. Source IDs and materialized values are different layers:
record both; do not mistake an input CONFIG_BLOCK echo for variant selection.
No new diagnostic API or private-only acceptance test is proposed.

Use the existing two-nozzle H2D by-layer cube context with two PLA logical
filaments, Manual [1,2], both actually used (wall1/infill2), no tower/support.
Keep printer/process identity and physical map fixed. Begin from a runtime
Native export, then author only the declared complete filament variant-slot
bundle into project_settings.config before both slicers. Plate map/mode must
agree with that project. Export rewriting must be disclosed and verified;
it must not be silently accepted as the requested value.

Finite proposed cases (Standard slots use valid distinct temperatures211/233;
unused High Flow slots use distinct valid values, not out-of-range sentinels):

| Case | Source self-index | Source variant layout | Required selection proof |
|---|---|---|---|
| F12 | [1,2] | Standard,Standard | Source slots0,1 selected, both logical filaments emit. |
| F21 | [2,1] | Same two Standard slots | Slots1,0; active temperatures reverse relative to F12. |
| F1122 | [1,1,2,2] | Standard,High Flow,Standard,High Flow | Both active nozzles Standard, select slots0,2. |
| F2211 | [2,2,1,1] | Same four-slot payload as F1122 | Select slots2,0; prove effect outside CONFIG_BLOCK. |

All variant-valued payload vectors must cover those slots coherently;
non-variant palette/type metadata remains logical-filament-sized. Native
acceptance and selected temperature commands must validate each proposed
case; the table is a source-derived hypothesis, not an observed PASS.
The paired identity-only change must leave geometry/placement and unrelated
settings byte/member-identical. Compare complete public Ares API/CLI G-code
with Native, including all headers/templates/config/footer and line counts.

No short/empty IDs, missing variant, invalid ID, generated-index fallback,
active High Flow nozzle, exotic stride or arbitrary vector shapes are included.
Do not relabel a malformed-vector refusal as an acceptance witness. If valid
F21/F2211 is rejected, reset by import, or cannot produce active consumption,
STOP and report the bounded domain unclosed. A true Ares difference also
stops here; production changes belong to a separate slice.

Harness destinations: new option_coverage/filament_self_index.rs and
same-prefix helpers, narrow domains/sweep registration; no change to the
production materializer, existing print_extruder_identity consumers or its assertions.

## Separate RED/GREEN and exact accounting

Implement master first, then self-index, each with separate Native-backed
RED/GREEN commits.
RED must first prove requested/effective input, accepted runtime Native and
complete public comparison plus actual consumer evidence. Then the new
key-local completeness test expects PASS while main actually reports
INCOMPLETE. If main already meets that expectation, report NO RED FOUND.
Do not invent a numerical bound or manufacture a production mismatch.

GREEN only wires the demonstrated finite cases into that key's domain and
sweep dispatch. Keep schema metadata intact, mark bounded source context and
explicit deferred dimensions. Other649 plans must be fingerprint-identical
when both keys are complete; compare all other plans separately per slice.
This SPEC updates the existing self-index negative
expectation in numeric_metadata_tests.rs: !provably_legal/omission.is_some()
becomes a positive bounded-case/metadata expectation once Native proves the
scope. This is an expectation update, not removal; all other assertions stay.
Any additional existing assertion retirement is reported before changing it.

## Runs and exit conditions

Directed runs use a fresh target/cache/artifact root each run.
Use existing native_environment: required missing dependencies FAIL;
optional visibly SKIPPED, never frozen fallback. Test fixtures are inputs;
Native G-code, exports, process results and receipts remain outside git.

For each slice: two-mode fresh RED, two-mode GREEN and repeated fresh-cache
verification, exact key-local sweep (ARES_PARITY_OPTIONS=1 plus that key),
entire option_coverage group including all non-Native tests in both modes,
strict Clippy/fmt/wasm32, and the CI workspace.
Use the unchanged whole-output comparator; only its existing generator-line
normalization. Publish input hashes, first active Native change, applied and
materialized identities, full comparison/line counts and raw invariance of
all other callers. No new exclusions, ignores, suppressions or retries to
hide divergence.
