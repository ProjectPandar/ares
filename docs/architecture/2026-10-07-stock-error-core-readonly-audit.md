# Historical stock rejection families: read-only core audit

Date: 2026-10-07. Source base: `57d63ba52`; own changes are documentation only.

## Boundary and evidence policy

This records the remaining **36** rejection rows from the `5cde90c23`
single-attempt stock matrix:20 loaded-config validation,8 minimum-width,
4 no-suitable-objects and4 postprocess. It is an audit of existing core
return values, source and controls, **not a fresh current-main replay** and
not a CLI exit-code gate. No Native retry or slicing job was started.

The owning upstream boundaries are `Slic3r::validate` / loaded-config
validation (`PrintConfig.cpp:10391-10394,10504-10559`,
`OrcaSlicer.cpp:3647-3653`); `Print::validate` width checks
(`Print.cpp:1524-1556,1654-1671`); scene/build-volume eligibility
(`Model.cpp:692-702,3423-3469`, `BuildVolume.hpp:91-106`,
`OrcaSlicer.cpp:5710-5724`); and CLI embedded `post_process` rejection
(`OrcaSlicer.cpp:1682-1690`). Rust destinations remain existing core
options validation, print validation, project-slice state and validated
entry orchestration. No Ares-owned new pipeline or duplicate CLI mapping
is proposed.

## Read-only result

| Family | Rows | Preserved matching core errors | Remaining exact-case observation |
| --- | ---: | ---: | --- |
| loaded-config validation |20|19| Snapmaker U1 0.4+0.6:1 |
| `Too small line width` |8|8|0 |
| no suitable objects |4|0|4 |
| postprocess unsupported |4|0|4 |
| total |36|27|9 |

**No missing detector is established.** The remaining9 have source-level
checks and existing controls, but that is not a substitute for each stock
input's current-main core result. Do not relabel these9 as fresh verified
matches or infer production changes merely from historical `ORCA_ERROR`.

The14 O3 range-family outcomes have exact Native/Ares aggregated error
text in `/tmp/worker3-c1-b18-seeded-artifacts/orca-runner-6PSMXr/o3-sweep-report.json`.
The5 RH3D firmware/wipe outcomes have both `validated.Err` and `sliced.Err`
matching the Native error in saved `/tmp/o2-rh3d-*/result.json` receipts;
these are existing receipts, not reruns or a newly established build identity.
The8 O4 unchanged fixture outcomes under the same b18 artifact root all
record both `ares_validation` and `ares_validated_slice` returning exactly
`Too small line width`; fixture hashes and Native evidence are bound there.

Remaining checks already exist:

- Snapmaker bridge width: `options/validation/extrusion_width.rs:57-62`
  emits `bridge_line_width: Bridge line width must not exceed nozzle
  diameter: 0.600000`. Existing option-validation controls cover the mixed
  nozzle bound and `runner/loaded_config_ranges.rs` covers public validated
  profile composition. The exact original selection remains to be replayed.
- Kobra3: `project_slice/state.rs:92-96` returns `InvalidInput` with
  `One of the plate is empty or has no object fully inside it. Please check
  that the 3mf contains no empty plate in Orca Slicer before uploading.`
  `ares-core/tests/o6_print_volume.rs` includes the real Kobra3 matrix
  fixture and geometric controls; it does not establish all four stock
  rows independently. Historical stdout only reported the generic CLI
  failure code, so do not invent a historical stderr text.
- MK3S: `project_slice/validated.rs::validate_post_process` rejects a
  nonempty list including `[""]` with
  `post_process is not supported under CLI.` Existing O5 real-fixture
  control is `runner/profile_entry_3mf.rs:639`; all four original stock
  inputs still need current-main confirmation.

Every row, exact stored core message or pending designation, source/control
and evidence path is enumerated in the [machine-readable record](2026-10-07-stock-error-core-readonly-record.json)
(original extraction: `/tmp/worker3-stock36-core-readonly.json`).
The counts concern error detection/text only; CLI status is deliberately
excluded. b18's approved607/607 seed receipt is existing evidence, not a
new36-case gate.

## Follow-up gate

Wait for worker4's CLI error-family commit and BROWSER-END. Fetch/rebase
main, then replay the preserved inputs or explicitly provenance-labelled
reconstruction where historical export presets were deleted. Compare core
text and CLI classification separately. Only a demonstrated missing core
check justifies a source-cited RED/spec/implementation slice; CLI mapping
remains worker4's scope. Native UB and uncaught exceptions remain separate
from these36 and never become numerical parity PASS.
