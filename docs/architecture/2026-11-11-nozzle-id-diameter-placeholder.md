# Nozzle-ID template metadata

## Upstream owner and Rust boundary

This slice rewrites `GCode.cpp::get_nozzle_diameters_by_nozzle_id`, `nozzle_id_for_gcode_placeholder`, `first_nozzle_id_for_gcode_placeholder` and their diameter/initial/current/next nozzle-ID template consumers from OrcaSlicer revision `345efec64c7b95a05ce5cf96df0dde66a158835c`. It extends the private G-code export metadata boundary; it does not design a new slicing pipeline or public nozzle-identity API.

The implementation base is `632803b108e72cc938e35197d1428309b9d04f69`. Its manual physical-group ownership, grouping evaluation, per-head carry, static G-code identity, processor consumers and non-tower timelapse behavior remain unchanged.

### Source contract

- `GCode.cpp:278-291`, `get_nozzle_diameters_by_nozzle_id`: absent group result yields an empty float vector. Otherwise query nozzle IDs 0, 1, …, append `std::stod(nozzle->diameter)` and stop at the first absent ID. Indexing is by nozzle ID, not logical filament, owning head or firmware hotend selector.
- `GCode/ToolOrdering.cpp:1503-1527` and 1894-1908: the physical-group producers assign `NozzleInfo::diameter` using `format_diameter_to_str` before retaining the grouping result.
- `utils.cpp:1784-1791`, `format_diameter_to_str`: choose the nearest of 0.2, 0.4, 0.6, 0.8 using a strict distance comparison in that order, then format to one decimal place. Consequently the template array is not an alias for raw configured `nozzle_diameter`; arbitrary configured values can snap to a supported nominal diameter.
- `GCode.cpp:1192,1292,1589`: diameter arrays in the filament-end/change/start dynamic template configurations.
- `GCode.cpp:3746,4401,4829,11091`: initialization of shared placeholder state, startup/ending configurations and ordinary tool-change configuration. Startup must receive the array; implementing only the first three sites does not unblock an H2D machine-start template.
- `Print.cpp:3700-3711`: Native publishes a print-wide nozzle-aware group result in every static sequential map mode. Without that publication the diameter helper sees no group result and returns an empty array; an empty array is not Native's normal supported sequential behavior.

The adjacent `get_nozzle_volume_types_by_nozzle_id` helper uses max-ID sizing and default filling. Those rules are not the diameter helper's contract.

### Rust destination

| Existing private boundary | Included change |
| --- | --- |
| `tool_ordering/nozzle_group.rs::{NozzleGroup,static_groups}` | Retain the canonical numeric diameter corresponding to upstream `NozzleInfo::diameter` in the existing physical-group record. Keep its extruder, group ID and volume semantics unchanged. |
| `tool_ordering/filament_group/context/inventory.rs::build_nozzle_groups` and its existing context caller | Populate the same metadata when expanding each head into manual physical groups. Snap once per head before expansion. Do not change enumeration, masks, hashes, scoring or candidate selection. |
| `machine/first_filaments.rs::{InitialFilaments,resolve,insert}` | Capture the ordered diameter list from the actual retained `ToolOrdering.nozzle_groups` before groups move to processor consumption. Install a numeric template list through the existing export-metadata/writer refresh path. |
| `motion/writer_state.rs::refresh_writer_state` and existing template callers | Audit propagation to the cited initialization, start, filament-end/change/start and end contexts; add only a missing connection if the existing refresh path does not reach a supported context. |

`InitialFilaments` already carries export metadata beyond first-filament identities. It remains a temporary compatibility shell around upstream shared G-code placeholder state, not a new Ares-owned identity subsystem. No public type, configuration option, new dependency or crate is introduced.

The admitted static group factories produce contiguous IDs starting 0 in vector order. Thus their retained record order implements the upstream consecutive-ID lookup without sorting, reconstructing groups from maps/counts, max-ID filling or repeated searches. An empty retained group list produces an empty template list, not a synthetic0.4 entry. Dynamic/sparse hardware inventories remain outside the admitted boundary.

The canonical numeric value is equivalent to the upstream supported-value string followed by `std::stod`; Rust need not allocate an intermediate formatted string. Preserve the strict nearest-candidate selection and first-candidate tie behavior. Do not change raw configuration values, geometry/nozzle-width inputs or exported configuration footer values to achieve this result.

## Authorized scalar extension

The original array-only implementation is component commit `b5b82deda9bd4ba9346991773e37a26e52ffbeba`. In both Native modes its canonical-diameter and retained-seven-slot/context witnesses and sixteen retained controls pass complete-output checks. The unmodified H2D primary still fails emission at `initial_nozzle_id`; no Ares G-code or first-byte mismatch is claimed. Explicit coordinator authorization extends this slice to `initial_nozzle_id`, `current_nozzle_id` and `next_nozzle_id` so that the actual H2D system preset can reach Native-byte-identical output. This section precedes scalar production changes.

Dedupe found partial prior coverage in the accepted H2C rewrite: `tool_transition/change/dynamic/mod.rs:46-51` already publishes current/next nozzle IDs through `motion::tool_selection::nozzle` for ordinary changes. The matching histories include accepted `632803b108e72cc938e35197d1428309b9d04f69` and two other H2C branch histories. This extension reuses that code and fills initialization/shared-state and other context gaps; it does not claim all three scalar names were absent everywhere or reimplement the accepted change pair.

### Scalar source contract

- `GCode.cpp:241-248`, `nozzle_id_for_gcode_placeholder`: with a group result and nonnegative filament ID, return `group_result->get_nozzle_id(filament_id, layer_id)`; otherwise return the supplied owning extruder ID. A missing lookup in a present result is not replaced with a filament ID or an invented default nozzle.
- `GCode.cpp:267-276`, `first_nozzle_id_for_gcode_placeholder`: use `get_first_nozzle_for_filament`; return its `group_id` when present, otherwise the supplied owning extruder ID. This initialization helper must remain semantically distinct from the layer-aware helper even though they agree on admitted static assignments.
- `MultiNozzleUtils.cpp:145-157,381-389,417-464`: the static layered result retains the selected default filament-to-nozzle map and actual nozzle list. The ordinary lookup resolves through that map and returns a physical `group_id`, or -1 when no nozzle is found; the first-use lookup uses the same map on the static branch. The dynamic first-used-layer search is not being ported here.

| Upstream setting point | Included existing Rust context |
| --- | --- |
| `GCode.cpp:3740-3745` | Machine startup/shared state: snapshot `initial_nozzle_id` and initial `current_nozzle_id` through the first-use helper. Initial remains stable after later tool changes. |
| `GCode.cpp:4400` | Initial non-support filament-start configuration: first-use nozzle for that filament, not necessarily the initial filament. |
| `GCode.cpp:1191,1279-1280` | Existing supported filament-end and change configurations: old/current nozzle before selection and next nozzle for the incoming filament. No active old filament preserves the source negative identity. |
| `GCode.cpp:1568-1569,1588` | After selection and filament-start: current nozzle becomes the incoming filament's nozzle. |
| `GCode.cpp:4838,4846` | Ending filament templates use the loop filament or active filament according to single-extruder-multi-material mode. Preserve the subsequent machine-end configuration's source identity rather than blindly substituting the active writer. |
| `GCode.cpp:6089-6090` | Existing layer-change configuration: current writer filament's nozzle. |
| `GCode.cpp:10858-10859` | Single-extruder selection still refreshes current nozzle even when no tool-change command is emitted. |
| `GCode.cpp:11069-11080` | Existing ordinary change configuration: current/next are old/new group-result nozzle IDs, not owning extruder IDs or firmware selectors. |

### Private Rust destination and admission

Extend `machine/first_filaments.rs::{InitialFilaments,resolve,insert,insert_machine}` only as the existing private export-metadata shell. Resolve initialization while actual groups are still retained, before the existing move in `orchestration.rs`. Runtime scalar lookup must use the same selected static group-result mapping already published in `filament_nozzle_map`, together with retained-group presence; that map is a scalar lookup channel, never a replacement inventory or diameter producer. No cloned group inventory or new public group-result API is required.

Connect current identity through `motion/writer_state.rs` and `motion/tool_selection.rs`, and context-specific overrides through the existing `layer_gcode/startup.rs`, `tool_transition.rs`/its dynamic configuration modules, `layer_gcode.rs` and `finish.rs` template boundaries. A small private source-helper module may keep changed Rust files below 400 lines; do not create an independent identity subsystem. `next_nozzle_id` belongs to old/new change configurations, not an arbitrary global default. Writer refresh must not reset runtime current identity to initial identity.

The source helpers' null-result/negative-filament extruder return is included source behavior, not a new Ares legacy fallback. Keep it distinct from a failed lookup in a present result and from the array's genuinely empty null-result semantics. Global physical nozzle ID, owning physical extruder and static firmware hotend selector remain distinct. Do not modify the existing command-selector behavior merely to make a template scalar pass.

The included subset is the already admitted by-layer static single-head, dual-head and retained manual multi-nozzle result. Its one selected nozzle per filament makes first-use and ordinary lookup agree for mapped filaments. Dynamic per-layer/H addressing, sparse inventories, sequential publication, new tower scheduling and changes to grouping admission remain deferred. Existing scaffolding stays a temporary compatibility shell around the cited upstream result and placeholder state.

## Authorized startup logical-filament alias

After the three nozzle-ID scalars, both modes qualify four strict array/scalar witnesses and sixteen retained controls, but the unchanged H2D primary stops at startup line 102 on `initial_no_support_filament_id`. Native still succeeds; no Ares output or byte comparison exists for that primary. Separate coordinator authorization includes exactly this observed alias, with supplemental spec and test commits before its production edit.

`GCode.cpp:3744` publishes `initial_no_support_filament_id` in the initialization block beside `initial_filament_id` (:3743) and `initial_nozzle_id` (:3745). Its value is the logical initial non-support filament, not the owning physical extruder, nozzle group or firmware selector. Port only the machine-start setting into `machine/first_filaments.rs::InitialFilaments::insert_machine`, using the existing `non_support` snapshot also exposed as `initial_no_support_extruder`. Do not add another selection algorithm, state field, public API or global writer-refresh alias.

The exact source startup contains 73 references to this name; the exact change template contains none. Current core publication and exact-symbol histories are empty. `initial_filament_id` has zero references in these two templates and remains deferred despite belonging to the same upstream block. Other adjacent vocabulary remains outside this authorization.

The existing real H2D system-template test is the Native-first RED witness. Bind both unchanged composed templates and Native's evaluated filament-type command, then record the public API/CLI startup102 emission failure on the committed supplemental spec/tests with the already qualified six-file nozzle-scalar working patch unchanged. This is a dirty-tree, source-bound RED, not a clean-tip claim. Only then add the startup alias. Require both modes' full 21-test suite, including all sixteen retained controls, to pass; preserve complete-output comparisons, independent raw API=CLI equality and all earlier failures. Finish with spec→tests→feat commit ordering and one final RR.

## Included and deferred behavior

Included:

- Numeric `nozzle_diameter_at_nozzle_id` arrays for existing supported by-layer static single-head, dual-head and retained manual multi-nozzle results.
- Physical inventory order and length, including unused physical nozzle slots, independently of used logical filament count.
- The canonical diameter producer needed by the source helper and the existing supported template contexts listed above.
- The three explicitly authorized initial/current/next nozzle-ID scalars and the supported setting-point subset specified above.
- The separately authorized startup-only `initial_no_support_filament_id` logical alias from the existing non-support snapshot.
- Preservation of typed unsupported-path boundaries and complete output comparison.

Deferred:

- `nozzle_volume_types`, `initial_filament_id`, identity vocabulary beyond the three nozzle-ID scalars and the single startup alias, and any other adjacent missing placeholder.
- Dynamic per-layer/H addressing, additional hardware-declaration/inventory support, preload/loaded-state/AMS rematching, flow matching and sparse hardware-group support.
- Hybrid grouping, clustering and custom-sequence redesign.
- Tower insertion/ordering, retraction, object labels, farthest/photo producers or generic timelapse changes.
- The complete `Print.cpp:3700-3711` all-static-mode sequential group-result publication rewrite. Existing `filament_group::check_scope` rejects dual-head automatic grouping for multiple sequential objects; `build_recommended` also gates that route. This slice does not relax those guards, change when grouping runs or synthesize a replacement group result from maps/counts.

Sequential consequence: existing typed unsupported outcomes remain. Already admitted contexts may inherit the diameter metadata through their existing retained groups, but this is not a claim of all-static-mode sequential parity or complete H2D sequential startup support. An unsupported path without the required group result cannot be repaired by inventing default diameter slots; an empty result followed by indexed template access still cannot supply the required value. Full sequential publication needs its own source-cited rewrite and Native end-to-end witness.

Exposing this metadata in an already supported template context does not authorize adding new tower/runtime behavior. Missing placeholders beyond the three nozzle-ID scalars and the separately authorized startup alias remain separate residuals; do not extend the slice silently.

## Failure modes and end-to-end evidence

Before implementation, reproduce the public API/CLI failure against successful Native slicing and retain actual input bytes, selected profiles/catalog, template source, Native output and all raw streams.

1. Missing array: an H2D startup template that indexes this array cannot render. Reuse the existing `profile_entry_3mf/machine_start_end_marker.rs` H2D 0.2/0.4/0.6/0.8 complete-output cases as regression controls, not as proof that an indexed lookup executed. The primary missing-array discriminator must use the unmodified H2D 0.4 system preset through profile-entry/catalog include resolution, with Native reading the actual system leaves through `--load-settings`/`--load-filaments`. Bind `Bambu Lab H2D 0.4 nozzle template machine_start_gcode.json` and `change_filament_gcode.json`, confirm the expanded startup text and actual evaluated diameter commands in both Native modes, then run the public API and CLI. Do not replace startup text, use smoke-template normalization, or substitute marker-only evidence. The bounded indexed-lookup derivatives are auxiliary canonical-value/inventory witnesses, not a substitute for this system-preset reproduction.
2. Wrong source value: raw configured diameters are exposed instead of canonical physical nozzle diameters. Qualify one settings-only H2D derivative with distinct noncanonical values and indexed output premises while preserving stock startup behavior. Native must demonstrate both raw footer values and canonical array values before this derivative is used as a discriminator.
3. Wrong index/length: an array reconstructed by logical filament or head count truncates a seven-nozzle physical inventory. Qualify one derivative of the existing H2C three-used-filament, counts 1/6 input, reading nozzle slots 0, 1, 6 in start/change/end template contexts. Preserve geometry, grouping and timing settings. Native must establish the seven-slot premises.
4. Partial propagation: startup works but filament-end/change/start or ending templates lack the array. Use the inventory derivative's context-specific behavioral comments to localize this; equality remains a complete-output assertion, not a marker-only comparison.
5. Unrelated output changes: retain the accepted H2C/gate/carryover/Hybrid/X2D and STL/3MF count controls. Unused new metadata must not change their Native/API/CLI output.
6. Scalar identity/epoch errors: initial changes after selection; current reports a logical filament, owning head or hotend selector instead of the physical group; next is installed too early; initial non-support startup or ending-loop templates use the active writer incorrectly; null-result return is conflated with an absent nozzle in a present result. Before scalar production changes, requalify the unmodified system primary's Native success and scalar-emission RED on the committed scalar spec. Preserve the initial array witnesses, then qualify immutable settings-only scalar-context versions within the same two derivative families where needed to discriminate old/new/initial IDs. Native must establish the values before Ares runs; existing archives remain unchanged. No third derivative family or replacement of the real system template is authorized.

The two derivatives above are bounded validation inputs for distinct source contracts, not an open-ended fixture search. Archive provenance must identify all changed settings and prove every other member byte-identical. Use behavioral marker names without worker/process labels. Do not edit existing accepted input bytes.

Native success and premises precede Ares reproduction for each new input, in official and seeded modes using their existing frozen authorities. A template-emission error that produces no Ares G-code is recorded as that error, not described as a first-byte comparator mismatch. Setup/static failures and invalid Native premises are separate outcomes.

After implementation, the unchanged generator-only complete-output comparator must cover Native/API and Native/CLI, including timing and footer; retain raw API/CLI equality independently. No normalization, timing/footer omission, marker-only substitution or downstream compensation is permitted.

Register any new cases beneath an existing family submodule. Do not grow `runner.rs`. Changed Rust files must remain at most 400 lines. Run fmt, locked strict workspace/all-target clippy and a fresh core/wasm target check, then bind clean exact-tip source, fixture, binary and environment snapshots to both-mode results.
