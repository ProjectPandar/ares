# Amended SPEC: 3MF config resolution and loaded-preset precedence

Status: **REVIEW-REQUEST, docs only; no new Native/test/production budget**.
Supersedes rejected dispatch/admission proposal925b13ae5. Seam v2 NO-RED closure
is accepted and unchanged. This is a source-cited configuration rewrite slice,
NOT a perimeter/seam algorithm repair or an Ares-owned pipeline.

## 1. Retained single-control candidate and provenance

Candidate is unchanged v2 single.3mf SHA256
`5518edad86d3af4e8966a8235e0e782b0bc26edf5f56a579673865169595b974`.
Retained root: `/tmp/worker4-seam-project-entry-red-y1PN/official/artifacts/orca-runner-rKLPnw`.
Primary witness is effective config; secondary acceptance is complete ordered
G-code. Native/API full difference M73byte1002/line40, lengths64683/69418;
API/CLI raw equality holds. Do NOT require equal **resolved** configs before RED:
wrong effective config under the same requested source/precedence contract IS RED.

Read-only localization found the following exact source values:

| Key | Explicit 3MF project_settings + API/CLI footer | Explicit loaded process.json + Native footer |
| --- | --- | --- |
| wall_generator | arachne | classic |
| wall_sequence | inner-outer-inner wall | inner wall/outer wall |
| brim_type | auto_brim | no_brim |
| overhang_reverse | 0 | 1 |
| overhang_reverse_internal_only | 0 | 1 |

Embedded `Metadata/project_settings.config` member SHA256
`5622f384e71fb366dd6fe2b99afef14adcb944170e235d4f8ed31776d2fec405`.
All five values are explicit strings, not missing/defaulted. `inherits` absent;
`inherits_group=["","",""]`; no embedded preset members. Named printer/process/
filament IDs are present, but their names do not establish that their values
supersede explicit serialized project values.

**Important retained argv/source distinction:** Native `command-O8knlt/command.json`
contains `--load-settings machine.json;process.json --load-filaments filament.json`.
The loaded process JSON contains all five Native values explicitly; SHA256
`972e234541199a11dea87edeb97a0ddd9cf151fa0fc446a6523f776220d6308d`.
It is from=system, inherits=fdm_process_phoenixProV1_common, and was flattened plus
case-overridden by the frozen test harness. API uses project plate validated with
profile None/default overrides; bare CLI has NO load-settings/load-filaments.
Thus same 3MF bytes presently accompany different additional supplied settings.
Do not hide this distinction or mislabel the explicit Ares values as a defaults/
system-inheritance failure. Retained config/output disparity is the review
candidate; proving product precedence drift versus omitted request sources still
needs a contract ruling. No bare-Native output is retained and none was generated.

## 2. Owning upstream file:function:lines

Pin8500fcdccaa10b5099ac20d252af3a7c560046f1; paths relative OrcaSlicer/src.

- `libslic3r/Format/bbs_3mf.cpp::_BBS_3MF_Importer::_load_model_from_file:1923-1939` dispatches
  project config and embedded presets; `_extract_project_config_from_archive:
  2632-2654` loads Metadata/project_settings.config via config.load_from_json.
- `libslic3r/Config.cpp::ConfigBase::load_from_json:820,925-935` deserializes
  explicit string option values (set_deserialize), with legacy handling. This
  loader is the shared config owner; byte-oriented Rust replaces filesystem I/O.
- `libslic3r/PresetBundle.cpp::load_vendor_configs_from_json:4602,4886-4927`
  resolves inherits/default base then applies explicit child config. Separately,
  `load_config_file_config:4271,4301-4319` redistributes inherits_group metadata.
  This pin has no PresetBundle::load_3mf routine: the actual 3MF owner above must
  not be replaced by an invented named function or an assumed GUI preset path.
- `libslic3r/PrintConfig.cpp::PrintConfigDef::init_fff_params`: reversal defaults
  false at1526-1545, brim Auto at1690-1711, wall sequence InnerOuter at2158-2179,
  generator Arachne at7155-7167. Wall sequence's embedded inner-outer-inner value
  differs from its default, independently ruling out blanket defaults attribution.
- Native CLI glue is provenance, not a new core architecture goal:
  `OrcaSlicer.cpp::CLI::run:1640,1953-2024,2073-2096` loads 3MF then preset files;
  local `update_full_config:2750-2812` sets supplied options; process call3011 uses
  update_all=true for explicitly loaded process; help7277-7280 states command
  line > loaded settings/filaments > 3MF. Direct file loading does not walk the
  system inherits chain to obtain these five already explicit values.

This chain makes config source/precedence, not LayerRegion generator dispatch,
the proposed owning boundary. Dispatch is a downstream effect only.

## 3. Exact Rust destination / existing scaffold

- `project_slice/validated.rs::slice_project_plate_validated:66-86` sets profile
  None and forwards caller overrides; `project_slice/state.rs::prepare_project_source:
  39-54` loads the project directly.
- **Embedded config destination:** `project/load.rs::load_project_with_preparation:
  55,118-150`: deserialize raw JSON, apply supplied overrides, normalize/legacy,
  ProjectSettingsBuilder.resolve. `options/project_settings.rs:29-48,118-127`
  routes fields and resolves builders; typed defaults in object_fields.rs:7/87
  and region_fields.rs:123-127 apply only when a field is absent.
- **Loaded-source destination if that contract is selected:**
  `profiles/load_settings.rs::compose_load_settings:69-124`,
  `profiles/catalog/compose.rs::merge:108-136` (direct imports do not walk inherits),
  `project_slice/profile_entry.rs::load_with_profile_preparation:43-66`, and
  `project/domain/profile_input.rs::from_3mf_with_profiles:44-110` overlay loaded
  raws on embedded config before final overrides. Reuse existing byte JSON inputs
  and composition; no second resolver, filesystem lookup or new public API.

These are compatibility shells for bbs_3mf/Config/PresetBundle concepts. Any
correction must replace incorrect precedence/source handling at the admitted
owner, not patch downstream wall choice or substitute config values to pass.
Current source+receipts show explicit embedded values reaching Ares unchanged
and explicit loaded values reaching Native; no erroneous Rust field parse or
missing system-parent lookup has yet been proven.

## 4. Review decisions and RED gate (no execution authorized)

Retain single-control as config-resolution RED candidate, not an ordinary PASS
control that gets dropped. Preserve its complete ordered G-code and config witness.
Review must choose the intended upstream request contract with all supplied
configuration sources recorded:
- Standalone 3MF contract: project-validated API/bare CLI corresponds to Native
  without loaded settings. Current cached Native includes loaded settings, so it
  cannot silently stand in for that missing baseline.
- Loaded-settings contract: identical retained machine/process/filament JSON
  bytes must be supplied through the existing public load-settings API and CLI;
  compare effective fields and complete output to retained Native. Do not demand
  equal actual config before RED or force it with new override/fixture edits.

No new Native/API run, fixture edit, test amendment or production edit follows
from this spec submission. A separately approved test/budget may establish the
contract, freeze public config-witness + whole-output RED, then request independent
RED approval. STOP on no RED, changed owner, admission/resource failure; preserve
all failures. User's current ruling is read-only localization/docs review only.

Included: four requested key groups (generator, sequence, brim, reversal including
internal-only), explicit project JSON versus supplied profiles/inherits/default
precedence. Deferred: broad PresetBundle rewrite, unrelated config migrations,
seam planes/scoring, perimeter geometry/order, brim geometry, travel/M73/statistics,
loader missing-build finding and lower-slice discovery. No fallback, Unsupported
guard, comparator/assertion weakening, profile/fixture substitution or diagnostic
core I/O. WASM/desktop API separation remains unchanged.

Evidence: `/home/indexyz/.ares-evidence/worker4/single-control-config-provenance.json`
and `single-control-config-resolution-localization.md`; full retained-root archive
SHA `e2c8312e8137f996c99ef28a877bbfd5e7614d0420aa5dd98dc77b23eabc6a1a`.
