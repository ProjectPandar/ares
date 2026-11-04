# F1 Native closure and minimum-width preflight

Status: Native-only diagnostics; closure contract decided below.
SPEC 308583e3a defines the F1/F2 split. F1 production remains unchanged.

The preflight run retains all inputs,
commands/environments, producer hashes, exits, complete raw G-code and stderr.
`closure-width-audit.json` records public emitted-path diagnostics. Both legs
run sequentially with fresh cache/root; no full/workspace or probes.

Official and seeded each ran31 cases:29 exit0 with G-code,2 exit156 without
G-code. The two width diagnostic cases use Classic noise, thickness2 and
Extrusion/Combined modes. Both producers say:
`Flow::spacing() produced negative spacing. Did you set some extrusion width too small?`
They return-100 (Unix156). This follows the warning at PrintConfig.cpp:3591–3600
and Flow.cpp::rounded_rectangle_extrusion_spacing:200–210, not a new Ares clamp.
The two cases are removed from the repository parity matrix and retained only
as raw preflight inputs/artifacts outside Git. No Ares guard is added for them.

## Closure finding

The prism is a watertight single model part,4 layers,2 walls, explicit
seam_gap0, no support/skirt/brim/spiral/scarf. Disabled/none controls have8
continuous positive wall paths and all8 return to their starting XY point.
Classic displacement has the same eight closed path identities in both modes.
Ripple displacement/extrusion/combined each has4 unclosed external paths:
endpoint gaps approximately0.320306mm on layer0 and0.200202mm on layers1–3,
with4 inner paths closed. These exact diagnostic vectors match official and
seeded. Ripple modes retain the same geometry/width behavior as source223–291.

FuzzySkin.cpp::fuzzy_extrusion_line_ripple:271–291 samples strictly before each
segment end and replaces the junction list without endpoint closure repair.
The non-Ripple branch repairs original closed-input first/back positions and
widths at406–408. apply_fuzzy_skin:689–693 directly invokes the whole-line
fuzzifier; its Ripple return does not pass through the random branch's repair.
PerimeterGenerator.cpp:534–540 nevertheless asserts loop connectivity, including
first-point==last-point, when packaging a closed ExtrusionLoop. Current Rust
arachne/traverse.rs mirrors that closing debug assertion.

These are public release-producer endpoint observations, not a debugger dump
of private junctions. Together with the source branch difference, they identify
a release/debug contract conflict needing a decision before F1 production.
Do not append a synthetic closing segment (would change the strict Native
stream), retag a closed line as open, remove or conditionally suppress the
assertion without an explicit decision, or call a failed debug slice parity PASS.
Decision: Native release behavior is authoritative; the upstream
assertions at:536–540 are compiled out under NDEBUG, and the TODO at:534–535
explicitly admits disconnected paths in practice. Emit the same open Ripple
loop, without a synthetic closing segment or retagging. In the F1 production
commit delete the mirrored first-point==last-point debug assertion broken by
this legitimate input; also delete the adjacent-path joint assertion only if
independently demonstrated broken. Keep every debug assertion not broken.
That production commit message and the SPEC addendum must cite
PerimeterGenerator.cpp:534–540, NDEBUG and closure-width-audit.json. This is
distinct from the old Unsupported unit retirement.

## Diagnostic projection revision

The original wall-witness.json draft incorrectly split continuous extrusion
at feedrate-only G1 commands. It is retained as v1, not accepted closure evidence.
v2 keeps those commands inside a continuous path, splits at actual XY travel,
retraction/wipe, role and layer boundaries, and reports endpoint distances from
the raw emitted coordinates. Every raw Native SHA is unchanged; no Native run
was retried and no parity comparator was modified or used here.

Open-neck paths remain open as expected from their input geometry; their
endpoint gaps are not used as proof that a closed source wall stays open.
Zero/sub-threshold thickness and small-padding inputs are Native-admitted.
PrintObject.cpp::region_config_from_model_volume:3664–3709 applies overrides
then normalizes fuzzy type to None at3707–3708 when thickness<0.001 or point
distance<0.01. Zero/sub-threshold cases therefore are strict inactive controls;
the current helper's threshold reflects this upstream rule. The initial SPEC
interpretation of that threshold as an independent Ares shortcut was incorrect
and is corrected before production. Their global footer may still serialize
External; the actual per-region type is normalized before group/fuzzify.
