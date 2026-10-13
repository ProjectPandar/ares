# C2 CI repairs: artifacts and explicit-plate fixture admission

CI run38480861032 self-hosted reported two public test failures at
C2 headaef0fb40f. These repairs change tests/authored inputs only.
The approved C2 importer/centering/sharing production patch stays unchanged.

## Artifact root without an oracle environment

`c2_same_project_selection_a_b_a` always exercises one loaded Project's
A,B,A geometry. Its optional Native comparisons can be unavailable in a
standard workspace environment; that does not remove its public assertions.
The unconditional `root_from_env().unwrap()` then failed because the workspace
has no ARES_PARITY_ARTIFACT_ROOT. Use the existing external-directory pattern:
validate and use the configured root when supplied; otherwise create and retain
a system temporary directory. Always write selection.json and execute every
geometry/repeat-selection assertion. There is no early return or silent skip.
Invalid configured roots still fail instead of falling back.

## The shared role fixture is byte-identical to main

Original `tests/parity/validation/kobra3-centered/0.8.3mf` SHA:
`0dc37a07daedf1aa01a48832a2ff3b4e70f6e367075274391e42884a98db1b0c`.
`git show 96328f269:<path>` has the same bytes/SHA. No truncation, replacement
or fixture mutation occurred. It builds object2 but its plate1 has no
model_instance membership.

`_BBS_3MF_Importer::_load_model_from_file`, bbs_3mf.cpp:2058-2082, selects
obj_inst_map object-ID membership before generating volumes. Plate creation
still records an empty plate (:4652-4666); membership is populated at :4698.
The approved positive explicit import therefore excludes this unassigned
object. Re-admitting it would undo the accepted C2 selected-load contract.

Fresh CPU12 official and seeded admission witnesses, each with separate
cache/artifact directories:
`/tmp/worker3-c2-ci-role-preflight-e17utvr1/evidence.json`.
Both modes produce identical admission decisions:

| Input/request | Native result |
| --- | --- |
| Original, --slice1 | return_code-6, process250, empty-model diagnostic, no G-code |
| Original, --slice0 | process0, plate1 G-code |
| Membership-complete, --slice1 | process0, plate1 G-code |

These are admission witnesses, not strict full-output parity PASS counts.

The dedicated WASM input `tests/parity/role-filament-usage/wasm-plate1.3mf`
adds only plate1 object2/instance0/identify14 membership. Its input SHA is
`f502b7a1ba819a479b7cbae99144fa68b6c5f1b658dbb731b9929c3b7742873c`.
`generate_wasm_plate1.py` reproduces it and its provenance manifest.
Model XML/geometry, transforms, settings and every other archive member remain
byte-identical to the original shared input. The original is kept unchanged.

The role test retains Some(1), the complete serialized-statistics comparison,
eight-role assertion and owned-buffer consumption assertion. Only its positive
input path changes. No guard, fallback, assertion retirement or production
validation-contract change is part of this repair.

## Verification boundary

Re-run the two existing failing tests in a standard env without any ARES_*
variables, using CPU12/jobs1 and the independent worker3 target. This exercises
the exact missing-root workspace path and the positive explicit-plate WASM
adapter path. Capture logs and retained selection.json externally. Then verify
strict/fmt/wasm32; coordinator owns CI/workspace/full and all pushes.

## Same positive-input requirement in runtime role migration

`runner/processor_migration/roles.rs` also used the original membership-empty
input with Some(1), while tools/chain-probe/role-filament-map.py:115 produces
Native --slice0. Its optional Native path was unavailable in workspace, so this
consumer did not fail in the self-hosted job. It would encounter the same
approved explicit-import rejection in full. Switch only its PROJECT input path
to the complete fixture. Preserve Some(1), every whole-stream, footer/map and
f64-bit assertion, producer scripts and cache policy. For this single assigned
object, all-import and explicit import have the same owner/geometry.
Verify the existing runtime roles module in fresh official/seeded Native runs.
