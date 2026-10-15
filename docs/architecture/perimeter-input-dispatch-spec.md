# Amended SPEC: loaded system-preset inherits_group ownership

Status: **REVIEW-REQUEST: SPEC + fixture/budget proposal only**. No production edits
or new runs authorized. Supersedes b7's undecided request-contract proposal.
Seam v2 NO-RED closure stays accepted. Owning rewrite slice is preset group metadata
and CLI loaded-source precedence, not seams, geometry or an Ares pipeline.

## 1. Identical-input witness

Coordinator ruled Native and Ares must receive identical 3MF AND loaded JSONs.
One authorized external check reused complete retained Native output and invoked
existing `ares_core::slice_with_load_settings` +CLI --load-settings/--load-filaments,
no extra overrides, matching generation metadata, plateNone/placementNone/seed0.
No new Native command; existing mainc251 core/CLI artifacts reused, digests saved.

Unchanged v2 single.3mf SHA
`5518edad86d3af4e8966a8235e0e782b0bc26edf5f56a579673865169595b974`;
loaded machine/process/filament SHA respectively:
- `9b1fe473c5add85ba8b9240135b4e04127e4a68ce215f22699fc0ae9c5ef8b6b`
- `972e234541199a11dea87edeb97a0ddd9cf151fa0fc446a6523f776220d6308d`
- `82b6ae0d147822cd90c38ed5368eda0767430316dcd996bbd629bebeeab10c41`

API/CLI exit0 and raw equality PASS; both raw SHA
`2cbd00bd4f11f24288a65af60623282441029c020d2a75b52ff44f8762afa095`.
Five previously differing effective fields now match Native: Classic,
inner wall/outer wall, no_brim, reversal1/internal-only1. Those previous deltas
were the bare-versus-loaded harness mismatch, not a product resolution finding.

**Actual same-request RED candidate:** complete established generator-only comparator
fails for both public entries at byte51464/line2151/column20, lengths64683/64761:

```
Native:  ; inherits_group = ;;
API/CLI: ; inherits_group = fdm_process_phoenixProV1_common;fdm_filament_pla;fdm_machine_phoenixprov1_common
```

All three loaded JSONs explicitly declare from=system and nonempty raw inherits.
Complete CONFIG_BLOCK comparison finds only inherits_group differs. Complete raw
line comparison finds only generated-by identity and this field. Do not remove
or normalize the metadata to claim NO-RED; full output remains acceptance.
No fresh paired/seeded Native RED or permanent test-only RED is claimed yet.

## 2. Exact owning upstream boundary

Pin8500fcdccaa10b5099ac20d252af3a7c560046f1, paths relative OrcaSlicer/src.

- `libslic3r/PresetBundle.cpp::construct_full_config:69-236` owns positional
  process/filaments/machine inheritance metadata; collection at109-112/129-131
  and export at225-236. This is the shared preset group concept being ported.
  Its general bundle semantics must not overwrite different CLI file-source rules.
- `OrcaSlicer.cpp::CLI::run:2038-2047,2079-2086` derives machine/process system
  status from loaded file `from`; groupslots at2838-2842 and2981-2985 become
  empty for system sources, parent name for User/user sources.
- `CLI::run:2189-2193,3161-3163` retains filament parent only for User/user and
  assigns the loaded filament slot. All three retained sources are system, hence
  Native empty slots despite raw nonempty parent strings.
- Local `update_full_config:2750-2812`, especially2799-2802, excludes raw inherits;
  CLI priority7277-7280 is CLI > loaded settings/filaments > embedded3MF. Metadata
  ownership is source-aware, not generic option copying.
- `libslic3r/GCode.cpp::GCode::append_full_config:5591-5648` exports full config.
  This consumer is evidence only; no footer/comparator suppression is permitted.
- Embedded config source remains `Format/bbs_3mf.cpp::_BBS_3MF_Importer::
  _extract_project_config_from_archive:2632-2654` / Config.cpp::load_from_json820.
  System vendor inheritance loading at PresetBundle.cpp4886-4927 is deferred:
  loaded CLI files already contain flattened values and do not resolve parent files.

## 3. Rust destination and witnessed handoff

Existing public loaded-entry route: profiles/entry.rs::slice_with_load_settings:
197-206 ->compose_load_settings69-124; CLI supplies the same JSON texts.

**Correction destination:** loaded-entry group/source-status preparation in
`crates/ares-core/src/profiles/load_settings.rs::compose_load_settings:69-124`
and existing group handoff, as needed:
`profiles/catalog/compose.rs::Catalog::compose:87-91`,
`profiles/composition/metadata.rs::ProfileGroupMetadata::from_profiles:23-32`,
`project/domain/profile_input/overlay.rs::merged_profile_raw:247-273`.
No broad parser/default/system-lookup rewrite or new API.

Witnessed source chain: load_settings imports documents; catalog/store.rs::
push_import57-62 uses user holding scope. inheritance.rs::merge_metadata328-346
retains both raw inherits and from. Group constructor collects every inherits
without consulting from; compose inserts that group; overlay copies those slots
into embedded metadata. Loaded system status must survive the CLI-specific
handoff. General bundle/catalog parent grouping is not blanket-authorized to
change; preserve its upstream semantics. These existing structures are temporary
compatibility shells around PresetBundle/CLI metadata, not an independent policy.

Required invariant: for explicitly loaded system files, corresponding inherits_group
slots are empty; for User/user preserve upstream parent policy; preserve positional
order, embedded filament count and unloaded slots. Do not fix output by clearing
all groups, deleting raw inherits, rewriting fixture JSONs or skipping the footer.

## 4. Proposed fixture and fresh RED budget (approval required)

Retained single-control is now the **positive** for this different metadata slice;
seam-control failure remains in history, not reclassified as a seam RED.
Propose freezing exact retained3MF and all three exact positive JSONs, with hashes,
source argv and public request metadata; no Native products committed.

Proposed control: same3MF/from=system/preset IDs/options/filament slots, with ONLY
raw inherits strings blanked in the three loaded JSONs. Native system metadata
should be invariant to those parent strings; current Ares is expected to agree
for this control. This is a new narrowly defined control requiring explicit
fixture approval; it has NOT run and byte equality is NOT claimed.

Proposed budget **three comparisons per mode**: exact positive, fresh positive
repeat, blank-system-parent control; official then seeded, CPU15/jobs1/threads1,
load<60 before builds/eachNative, fresh cache/artifact roots, START/END receipts.
Require Native normalized complete repeat equality, public API/CLI raw equality,
control complete PASS and Native positive/control complete neutrality. Effective
inherits_group divergence under identical requests is RED, not excluded admission.
Freeze separate public test-only RED commit ->independent REVIEW-REQUEST before
any production correction. Stop on no RED, control/admission/resource failure or
changed source boundary. No retry or extra cases automatically approved.

Included only loaded system source metadata precedence/positional group fidelity.
Deferred User/user matrix expansion, optional/unloaded-kind behavior changes,
vendor parent resolution, compatibility metadata, other config migrations,
seam planes, geometry/order/brim/travel/M73/statistics and loader/lower discovery.
No fallback/Unsupported guard, new resolver, core I/O or comparator weakening;
WASM/Windows/macOS/Linux byte API separation remains unchanged.

Evidence: external `identical-loaded-single-report.md`, F2JO/identical-input-check.json,
unchanged whole comparator source and driver/binary/input digests. Full same-input
external-check archive receipt is published with REVIEW-REQUEST. Earlier b7 source
provenance and both failed harness histories remain preserved separately.
