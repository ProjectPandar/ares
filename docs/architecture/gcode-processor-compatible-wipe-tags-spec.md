# GCodeProcessor compatible WIPE tags: source boundary and RED design

Status: SPEC REVIEW REQUEST; no implementation or new RED execution.
Baseline: `ca1d32ea5` (role filament GREEN merged). Upstream: OrcaSlicer
`8500fcdcc`, `src/libslic3r/GCode/GCodeProcessor.{hpp,cpp}`.
Independent finding: `/tmp/worker6-processor-bbl-wipe-tag-finding.md`.

## Upstream producer / printer-dialect selection

There are two related decisions; the flag is not inferred from WIPE comments.

| Source | Behavior |
| --- | --- |
| `GCodeProcessor.cpp:2618-2633::process_file` | For OrcaSlicer/Slic3rPE/Slic3r-produced files, load embedded configuration; a nonempty `printer_model` sets `s_IsBBLPrinter` from the `Bambu Lab` prefix |
| `GCode.cpp:2047::GCode::export_gcode` | During slicing/export, set the same processor flag from `print->is_BBL_printer()` |
| `GCodeProcessor.hpp:375::reserved_tag` | Select BBL `Reserved_Tags` or non-BBL `Reserved_Tags_compatible` using `s_IsBBLPrinter` |
| `GCodeProcessor.cpp:58-61,81-84` | BBL wipe comments contain a leading space after `;`; compatible comments are `;WIPE_START` / `;WIPE_END` |
| `GCodeProcessor.cpp:3146-3155::process_tags` | Prefix-match the selected comment token; set `m_wiping` true/false; unrelated spelling does not change state |
| `GCodeProcessor.cpp:3835-3851::process_G1` | Wiping takes precedence over normal extrusion/retraction/travel classification |
| `GCodeProcessor.cpp:3895-3911` | Ordinary extrusion updates filament caches; a wiping move does not take that branch |

The compatible check preserves the leading space distinction. It uses
`starts_with`, not equality or whitespace normalization. Prefix suffixes may
therefore follow the recognized token. Both tower tag tables instead contain
leading spaces (`cpp:74-75,97-98`); they are not part of this correction.

## Current Ares boundary and included correction

`crates/ares-core/src/project_slice/gcode_emit/processor/estimate.rs:76-78`
unconditionally accepts both spellings, after trimming the line, regardless
of `config.limits.bbl_printer`. `MotionState::set_wiping` then affects the
same-pass accounting and `MoveRecorder` classification. The producer/dialect
flag already exists in `ProcessorLimits`; this slice does not redetect it.

Port only the **non-BBL** branch of Native `process_tags`:

- With `limits.bbl_printer == false`, inspect the comment after the semicolon
  without stripping its leading space. Only compatible WIPE_START/END
  prefixes toggle state; BBL-spaced forms have no effect.
- Preserve ordering: toggle before recording the line's motion. An unrecognized
  END must leave an already active compatible wipe unchanged.
- With `limits.bbl_printer == true`, preserve current Ares behavior. This slice
  changes no BBL processing path or output. The existing BBL acceptance of
  compatible spelling is an adjacent discrepancy, explicitly deferred.

Rust destination: replace the two unconditional wipe-state match arms in
`Estimate::record`; any necessary tag parsing belongs to the existing
`processor/moves/tags.rs` reserved-tag boundary. Keep integration minimal:
`estimate.rs` is already 411 lines and must not grow a separate parser.
The current Estimate/MotionState scaffold remains the temporary compatibility
shell around upstream GCodeProcessor, not a new Ares pipeline or fallback.
No new configuration, crate, producer detector, Native patch or adapter logic.

Included downstream effects are those caused by this state correction:
ordinary-versus-wipe move classification, existing volume/color/role accounting
and any timing/progress effects of the corrected classification. Do not edit
role-cache formulas, tool-change flush boundaries, geometry, export ordering
or independent timing formulas to make an oracle pass.

Deferred: BBL-side symmetric tag strictness; other reserved-tag dialects;
leading indentation before a semicolon; broader foreign-producer detection;
tower/flush tag parsing; producer-specific role parsing; unrelated motion
precision. These are not silently folded into this slice.

## Native RED from the frozen measured input

Copy the full frozen reproducer unchanged into a new fixture directory,
without modifying the role-filament fixture or other workers' inputs:

- Input `/tmp/worker6-role-cache-pre-dialect.gcode`, SHA256
  `d0d02feedc6cf6a144402818ff02f2db82c950757dd407599f4015b3adf618b5`.
- Native map `/tmp/worker6-role-cache-pre-dialect-map.json`, SHA256
  `43a518d832368c528ef8027912b5e41f2cfd7e37fa8be4dcfc1ba4ca0a00413f`.
- Lines 31-33: BBL-spaced WIPE_START, `G1 X120 E1`, BBL-spaced WIPE_END.
  The embedded printer is non-BBL, so Native counts the move as ordinary
  ExternalPerimeter extrusion; Ares incorrectly excludes it as wiping.

Observed Native ExternalPerimeter is 0.0107540787180275 meters /
0.10290679264068603 grams. Current Ares is 0.009754078673612502 meters /
0.09333769798278808 grams. These are evidence, not hand-computed expected
values: assert the **complete six-role Native map and every f64 pair bit**.
Expected first assertion difference is ExternalPerimeter length. Its excess
in Native corresponds to the one relative-E move, not an upstream UB.

Before committing RED, replay the exact frozen input through the existing
Native ELF using `nix shell nixpkgs#gdb -c gdb`, without rebuilding OrcaSlicer.
Use the already footer-validated map reader from the approved role-filament
slice; pin ELF SHA256
`59f9ea7e16165df6979a243de7d6acd49d8cf523ac5de6d3fec08d6e25c7896e`.
Verify the new map against the frozen map and record the Native dialect flag
as false (read-only debugger observation), input SHA, metadata and logs.
The existing process-mode map has no footer equality assertion because flush
and unequal-material cache scopes differ; retain that explicit distinction.
Do not turn rounded footer values into a substitute role oracle.

Replay through the existing whole-processor Estimate harness, optionally with
MoveRecorder. Freeze complete Ares statistics and Native result artifacts;
compare the full role map, not only the differing row. Also capture Native
`--dump-moves` to check the move at source line 32 is ordinary extrusion and
that the Ares move is initially Wipe. This is a processor-result RED, not a
fabricated slicing/G-code byte difference: the input G-code bytes are fixed.
Whole-result comparisons must identify any other preexisting mismatch rather
than weakening assertions. No new isolation test is written after GREEN.

Additional Native controls, captured before implementation:

| Input | Required signal |
| --- | --- |
| Existing corrected compatible fixture | Complete role map remains equal; recognized wipes still excluded |
| Compatible START followed by BBL-spaced END and subsequent XY+E move | Native ignores that END and remains wiping; isolates END behavior |
| Non-BBL compatible prefix with suffix text | Native prefix matching toggles state; no equality-only replacement |
| Matching BBL printer configuration and canonical BBL role/WIPE tags | Native/Ares valid BBL control remains equal, no BBL regression |

Use the existing raw Native executable and byte-exact retained inputs for every
control; no reconstructed Python/Rust accounting constants. New variants have
their own Native reference captures and SHA receipts. The source-derived
minimal sequence in the finding is not claimed as an already measured input.

## Sequence and acceptance

1. Approve this source-cited spec. Browser window currently allows only
   read-only analysis/docs; do not launch heavy work before RESUME.
2. Add and observe the frozen-reproducer Native RED, plus controls, in a
   separate test-only commit. Report the RED hash before production changes.
3. Port the scoped non-BBL wipe-state decision. Complete Native role maps and
   line-32 move classification must agree; compatible and BBL controls retain
   parity. No role/timing formula adjustment is permitted.
4. Run directed tests, fresh paired seeded controls and official controls
   (processor classification can affect timing/progress). Candidate failures
   must be a subset of base; list every status/G-code change and explain its
   relation to WIPE state. Preserve raw artifacts and oracle inconclusives.
5. Run workspace nextest, strict all-target/all-feature clippy, fmt and
   wasm32 checks. Request review and stop before push.

Unexpected G-code differences must be explained, not normalized away: genuine
corrected timing/M73/footer changes may occur only in non-BBL inputs containing
previously misrecognized tags. Controls without affected tags, and BBL inputs,
must remain invariant. Normal Native/Ares G-code comparisons normalize only
`generated by`; processor references preserve original input bytes throughout.
