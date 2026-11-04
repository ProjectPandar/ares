# Residual stock CLI rejection parity:13 defined errors

Status: proposed source-only. Historical/current matrix captures are leads, not
a new production RED. This is a libslic3r/CLI boundary rewrite slice,
not an Ares pipeline feature.

## Cohort and source ownership

Exactly13 current stock ORCA_ERROR rows after excluding the #7 cohort28 and
Native crash rows12:

| Rows | Native code | Upstream rejection boundary / exact text |
|---|---|---|
| FLSun QQ-S Pro0.4 | -100 / Unix156 | GCode.cpp:3636-3645 `Failed to generate G-code for invalid custom G-code.\n\n` plus ordered template failures and `Please check the custom G-code or use the default custom G-code.`; PlaceholderParser.cpp:931 `Variable does not exist`, machine_start_gcode line51, original offending line/caret retained |
| Ginger G1 1.2/3.0/5.0/8.0 | -100 / Unix156 | same GCode aggregation; PlaceholderParser.cpp:999 `Not a variable name`, machine_start_gcode line1; entire original line/caret and blank lines retained, not merely generic prefix |
| Anycubic Kobra3 0.2/0.4/0.6/0.8 | -50 / Unix206 | OrcaSlicer.cpp:137 `One of the plate is empty or has no object fully inside it. Please check that the 3mf contains no empty plate in Orca Slicer before uploading.`; Model.cpp:692-702/3423-3469 volume state, OrcaSlicer.cpp:5710-5724 pre-Print::apply eligibility guard |
| Prusa MK3S0.25/0.4/0.6/0.8 | -19 / Unix237 | OrcaSlicer.cpp:131 `post_process is not supported under CLI.`;1682-1690 embedded post_process load guard, including nonempty list containing empty string |

Custom-template exception is printed in full on stderr then recorded as generic
CLI_SLICING_ERROR at OrcaSlicer.cpp:6286-6291. Eligible-object/postprocess cases
can have empty stderr and detailed metadata instead. Never compare a generic
result error_string against a detailed Core parser payload or invent a channel.
OrcaSlicer.cpp:416-460 owns record_exit_reson serialization; stdout/exit/result
are adapter behavior, not an independent core validator policy.

Rust boundaries: existing project_slice/gcode_emit/template_failure.rs and
PlaceholderParser implementation; project_slice eligibility state; validated
postprocess guard; ares-cli/src/failure.rs family classification/result transport.
Reuse these owners; no parallel error registry or API expansion. Existing
orchestration is a compatibility shell around the cited upstream phase only
where fresh evidence demonstrates a mismatch.

## Existing harness and non-overlapping ownership

- Reuse runner::OrcaRunner, existing native cache producer identity, locks,
  publication recipe and custom_gcode_reference. Existing
  cli_custom_gcode_exit.rs (#97) already compares five named inputs and cache
  hits. Do not reimplement Native invocation/cache logic or commit products.
- cli_exit_channels and the cli_result cache migration for
  postprocess/eligibility are a separate slice. **Do not edit these files**;
  reuse an existing recipe if13-case coverage needs additions.
- For metadata bytes call shared orca_parity/native_wall_clock.rs::
  result_comparison_bytes. Only prepare_time/export_time numeric wall-clock
  values are ignored; no JSON reorder/reserialize, family-only comparison,
  missing-field exception or additional output normalization. Stdout/stderr
  compare in full without normalization. Core public byte APIs have no process
  exit code; CLI E2E is required for process-code/channel claims.

## Admission and TDD

Failure inventory: wrong/missing original inputs; selection reconstructed but
mislabeled original; wrong plate/placement; Native input-read failure confused
with desired error; stale/frozen Native cache; code low-byte mapping wrong;
parser detail lost, caret/line/blank lines reordered; metadata generic/detailed
messages confused; result field/first-error phase differs; false final G-code;
wall-clock normalization hides other bytes; sibling-owned harness changed.

Plan:
1. Bind all13 exact surviving current stock exported inputs and hashes. Failed
   import/export without a surviving artifact may require reconstructed stock
   selection, explicitly labeled, no hand-picked preset/filament repair.
   Reuse existing fixture bytes only after recording whether they equal the
   current-stock inputs; differing hashes are separate provenance, not silently
   the same exact case. Source/binary/resources/argv/cache/commit/status records
   stay outside the repository. Missing input or wrong Native error is INCONCLUSIVE/non-PASS.
2. Independently execute Native via existing recipe and the public Ares CLI on
   the same bytes/plate. Assert exact exit code and full stdout/stderr/result.json
   (shared two-field clock rule only). Reuse existing Core public APIs to check
   detailed typed rejection where phase-appropriate; no expected Unsupported
   guard assertions. Rejection does not make raw stock ORCA_ERROR valid-PASS.
3. If all channels already equal, deliver docs/test-only conclusion. If a real
   public CLI/API mismatch appears, commit unchanged failing E2E/input first,
   then port only the source-owned missing behavior in its existing Rust owner.
   Do not modify expected output, input or comparator to get GREEN. Harness
   infra/input inequality is not production RED. Bigger upstream boundary requires revised spec.
4. Preserve all attempts and cache provenance; no retry-to-PASS, crash imitation,
   preset repair, blanket guard or frozen-output fallback. Default missing-
   Native tests visibly SKIPPED before artifact-root access; required missing
   Native fails closed. A cache hit must not conceal missing required producer.
5. Gates: production work requires latest-main directed +workspace/clippy/fmt/WASM
   and ordered seeded/common audit; docs/test-only uses default/no-ARES workspace,
   required directed cases and clippy/fmt.
6. Conclusion ARD.

## Deferred / excluded

12 SIGSEGV/SIGABRT rows remain listed in the stock audit, no signal emulation or
new guard. #7 already-covered28 excluded. INPUT_REJECTED9 validation parity and
VENDOR_INCOMPLETE10 catalog-selection audits are separate; no preset selection
policy changes here. SLA/timelapse Option-context behavior remains a separate
pending test-only task. No new slicing/G-code feature, option family, API/crate,
MM/fuzzy/spiral/adaptive/ZAA behavior.

Exit: each admitted13 input independently matches all CLI channels and its
phase-owned public error, or has an honest failed/inconclusive disposition;
no unsupported production conclusion, Native product or owner conflict.
