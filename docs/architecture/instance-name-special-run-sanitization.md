# Instance-name special-run sanitization

Status: docs-only SPEC, then separately committed Native-first public RED.
Base47821c760d5ea86102a34bbc78c092bbaf381214; independent branch
worker6/instance-name-sanitize-spec. CPU15/jobs1/test-threads1; no FULL.
Only SPEC+RED is authorized now; production GREEN requires separate approval.
Earlier approved chains20e41b63f,8ddfa8ca5,360e9aa90 remain clean/frozen separately.
COORD's eventual W6 window carries four chains; no current integration permission.

## Upstream owner / Rust destination / architectural decision

This is a source-cited libslic3r string-operation rewrite, not an Ares object
naming policy, UI feature, new exclusion subsystem or slicing pipeline feature.

`OrcaSlicer/src/libslic3r/GCode.cpp:4364-4376::sanitize_instance_name`:
std::regex_replace applies the explicit22-character special class in one-or-more
runs, replacing EVERY run with ONE underscore, INCLUDING edge runs. It then
erases AT MOST ONE leading underscore and AT MOST ONE trailing underscore, each
only when nonempty. This is NOT trim-all-underscores or an ASCII-word whitelist.
Keep the exact existing special set: literal space and the21 named punctuation
characters; other characters, including other whitespace/Unicode, unchanged.

`GCode.cpp:4378-4382::get_instance_name` sanitizes the raw object name, composes
name_id_object_copy_instance, then sanitizes AGAIN. These TWO calls and all IDs
are source-owned behavior; do not collapse them or change their consumers.

Rust destination ONLY the BODY of private
`project_slice/gcode_emit/footprint.rs:305-326::sanitize_instance_name`.
It currently holds the same22-character constant, but its separator is emitted
only if output is already nonempty; a final pending separator is dropped. It
thus drops edge special runs BEFORE stripping another original edge underscore.
This partial compatibility shell must become the cited replace-every-run then
single-edge-strip operation. Preserve the charset constant and existing public
interfaces; no new regex dependency, escaping policy or name validation.

## Non-negotiable freeze / included and deferred behavior

- Future production-only GREEN may change ONLY that private function BODY's
  run-emission behavior: emit '_' for every run, including the first/last, THEN
  strip at most one edge underscore at each end. Never strip ALL edge underscores.
- Keep the exact charset, two-call instance_name composition, IDs and traversal,
  M486/EXCLUDE command definitions/placement, labels, model/3MF loading and ALL
  consumers. No caller workaround, renamed fixture, parser guard or fallback.
- footprint.rs already ends with `#[cfg(test)] mod tests`. Its declaration and
  existing footprint/tests.rs MUST remain byte-identical; do NOT add/modify unit
  tests there. The public Native byte seam below is the sole new test location.
- No footprint geometry/Klipper hull correction, object grouping or slicing,
  writer numeric formatting, template engine, GCodeProcessor/M73/time change.
  All adjacent behavior is deferred. Core remains portable WASM/Windows/macOS/
  Linux with no native process/filesystem dependency or new API.
- No queued painted/side-cut, brim/EFC, combine-solid, Aligned-rectilinear GEOMETRY
  template identity, concentric sparse, spiral/processor, seam occlusion,
  zero-area winding or placeholder-double rendering changes.
- Any strict residual during a separately authorized GREEN means STOP/review;
  do not broaden the boundary or modify tests/inputs/helpers/SPEC/comparators.

## Approved discovery / first producer proof

External authority:
`${EVIDENCE_ROOT}/w6-47821-object-label-name-nswzrbdw/`.
DISCOVERY-RR.md,verification.json,verify-discovery.py,pair-proof.json and trace.

Both inputs: one Classic/unpainted object, Marlin, exclude_object=1, interlocking/
polyhole/brim/skirt/EFC OFF. Only logical GUI property ModelObject.name differs:
target !__probe / control probe. Eight fresh successful Native slices establish
Native2 stability in BOTH modes BEFORE current-main actual public byte API/CLI.
Two actual API+two CLI runs (outputs compared in both modes);2 raw pairs equal.
Both official/seeded: target COMPLETE Native/API+Native/CLI FAIL, control COMPLETE
PASS. First strict NON-M73 difference byte468,line19,col7 in the M486 definition;
the surrounding M486 selection/reset records match. No output masks.

Two VALID normally exiting read-only GDB probes preserve COMPLETE observer
outputs: Native canonical stability and Rust RAW equality. First argument SAME
!__probe. Next actual Native argument __probe_id_0_copy_0 vsRust _probe_id_0_copy_0.
Thus the first producer diverges BEFORE the next sanitize/definition consumer.
Native first result __probe is INFERRED from next argument+literal composition,
not a captured return. Rust actual pre-suffix stripped values _probe and then
probe_id_0_copy_0. Native-v1 optimized-out/stopped return probe is EXCLUDED.
No inferior calls or memory/register writes; may-call-functionsOFF.

## GUI reachability and derived ZIP-name differences

GUI_ObjectList.cpp6055-6073 Rename accepts the names; Plater.cpp248-256 bans
<>:/\\|?*\" but not ! or _. update_name_in_model1175-1211 sets object and part
names: single-part RenameObject also changes part name; RenamePart then restores
the SAME original part label in BOTH projects. The final sole object-name delta
is GUI-reachable; do not claim a single click preserves a single-part label.

Actual Native-exported inputs name the bundled geometry member from object name:
!__probe_1.model vsprobe_1.model, with corresponding main-model/relationship URIs.
Geometry member PAYLOAD is BYTE-IDENTICAL. model_settings changes ONLY direct
object-name metadata; part name/settings JSON/all other payloads identical.
pair-proof.json explicitly checks the exact named-member/ref equivalence; this
is input-scope proof, NOT a G-code normalizer/mask. No G-code archive member.

## Frozen input-only public RED matrix

Copy the ALREADY qualified Native exports unchanged; no regeneration or edits:

| Role | tests/parity/instance-name-sanitize/ | SHA256 |
|---|---|---|
| Target | leading-special-target.3mf | `72ffb0bf2f159c54f21577abe5ccb650521f191f3bcea3fd5e514879ab27fc67` |
| Sole logical-name control | plain-name-control.3mf | `b28411348c6bd4dc30c5ec412403f389697fd8362026a64472b4926ec4a635b9` |

Two direct unchanged native_stream_case::check ->NativeCase calls, meaningful
labels, source citations, input SHA freezes, assert_two_tools=false (single
object). Dynamic live Native expected bytes, NO hardcoded Native output/name,
M486 string, coordinate, mismatch expectation, exception-as-success or private
sanitizer test. No Native G-code/golden output committed; inputs only.

## Approval sequence / verification contract

1. Docs-only SPEC commit, then input/test-only RED commit; production and the
   existing #[cfg(test)] module stay frozen. All changed text<400LOC.
2. On that exact RED, official AND seeded EACH target full FAIL/control full
   PASS; Native2 stable FIRST, actual API success/CLI0 and BOTH COMPLETE ordered
   Native/API+Native/CLI comparisons. Keep M73, motion, time, comments and footer.
3. Target canonical panic occurs BEFORE final raw assertion/adapter update:
   independently inspect actual raw API/CLI files; do not claim the assertion ran.
4. fmt and strict workspace/all-target clippy PASS; helpers/golden/tests/production
   scope proof. Send SPEC+RED RR and STOP BEFORE GREEN; no FULL/rebase/push/merge.
5. If separately authorized later, production-only function-body GREEN must make
   the frozen pair fully pass in BOTH modes plus relevant existing object-label/
   exclusion neighbors and fmt/clippy/WASM gates. Do not add/change existing unit
   tests or claim unobserved universal name/firmware parity from these two cases.
6. Four-chain W6 integration still requires explicit COORD window/main, preserved
   refs and exact integrated-tip gates. Historical FULLs provide no new credit.
