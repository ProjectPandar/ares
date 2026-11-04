# SPEC proposal: ordinary zero-extrusion XY moves retain verbose role comments

## Exact base

Docs-only source-owned proposal on main
**9cfaf0d16f0f5c4bf5fb6bf6c62d1c54e0d94ecf**. No production or
test/fixture implementation is included.

## Actual Native/public difference,not a static hypothesis

Stock-origin Creality Ender-3 0.4nozzle /0.20mm Standard @Creality Ender3 0.4 /
Creality Generic PLA,original32-sided12×6mm cylinder,30×0.2mm layers,Classic
2walls/15%crosshatch/3top-bottom. Common GUI Verbose G-codeON;only pairkey
**Top surface flow ratio1(control)→0(target)**. Zero is the documentedGUI
lower bound,not malformed input or forced Native compatibility.

| proved external INPUT | SHA256 |
|---|---|
|control|ef1951bd9f85d84ebfaf569b5a0ac4cf729e22e3c0c0d788ea5ad71b54a6e632|
|target|90ca1740ea4fece67054f2fdeb4202ed31f7fd43b521f9976aacf9647550631b|

Both official and seeded0:Native2FIRST runtime0/stable,then validatedpublicAPI0
and actualCLI0;rawAPI==CLI. **Control complete orderedPASS,target real complete
orderedFAIL**. First normalized **byte131415,line3737**:
Native `G1 X114.426 Y112.701 ; infill` versus Ares `G1 X114.426 Y112.701`.
Native154886bytes versus API/CLI154625bytes. Crossmode normalizedNative/rawpublic
identities are exact. Allroles/time/M73/fan/E/F/wipe/start/end/config/footer
remain compared;only original generator/date normalization,no comment mask.

Full diagnostic inventory has29zero-EG1 lines missing only ` ; infill` in Ares,
lines3737through3824,261bytes total. No numerical/position/E-word discrepancy
is asserted;both omitE for zero flow. These diagnostic facts are not masks or
replacement comparator allowances. All29must close under whole equality.

## Owning upstream source boundary

Pinned OrcaSlicer **8500fcdccaa10b5099ac20d252af3a7c560046f1**:

- `src/libslic3r/GCode.cpp:7031–7036`,normal extrusion dispatch passes
  `tempDescription` to `m_writer.extrude_to_xy` even when the computed dE is0.
- `GCodeWriter.cpp:933–956`, `GCodeWriter::extrude_to_xy`:ifabs(dE)≤doubleepsilon,
  setforce_no_extrusion. Skip filamentextrude andemit_e whenforced,but
  **unconditionally call emit_comment(full_gcode_comment,comment)** afterward.
  E-word omission and role-comment emission are independent source decisions.
- `GCode.cpp:2038`sets existingwriterverboseflagfromgcode_comments. No new
  global-policy or caller-level flow semantics is needed to port this contract.

Rust destination **only**
`crates/ares-core/src/project_slice/gcode_emit/motion/extrusion.rs::linear_segment`,
zero-Ecomment handling at103–129. Existing branch
`extrusion.abs() <= f64::EPSILON` emits bare`G1 X… Y…\n`;role/bridge comment
construction occurs only in the nonzeroEbranch. This is an incomplete shell
around sourcewriterextrude_to_xy,not an Ares-owned non-printingmove convention
or a newpipeline feature.

Futureincluded port:keep existing role-description/bridge-suffix verbose
formatting available in both E/no-Ebranches. Move/reuse that exactlocalcomment
calculation asneeded;append it on zero-EG1 too. Do not duplicate a Top-surface
special-case or hardcode`infill`fromthewitness. No new module/helperframework,
configuration/API/globalwriterstate or fallback/admission rejection.

Nonnegotiable:preserve existing epsilon gate,Eomission,filament-coordinate/
usage bookkeeping,XY emission/state updates,flowratio arithmetic and selection,
feedrate/acceleration/motion planning. Never “fix” this by emitting E0,changing
Top flow=0 admission/removing moves,or masking comments/progress. Included
behavior is **ordinary emitter comment policy independent of E presence**.

## GUI stock-origin provenance and bounded evidence

Origin already official-exported INPUT
`tests/parity/fan-mixed-timing/control.3mf`,SHA
b8bc9d8ab8db4eeaa21da61e759c64f230ddf4da800997424faf10fc53cfd298.
Common VerboseON is GUI Tab.cpp2753/PrintConfig.cpp3965–3969.
Top surface flow ratio is Tab.cpp2404/PrintConfig.cpp1366–1375,coFloat,
**min0,max2**,default1. Paironlytop_solid_infill_flow_ratio changes. Allother
settings/geometry/placement/non-settingsZIPpayloads unchanged;no custom G-code,
syntheticpresetJSON/model substitution/new GUI-export transaction claim.
Both fan_speedup_time and fan_kickstart0,so **FanMover is not active** and own
frozenverbosefix is not a prerequisite. This is not its generated/split text.

Prior bounded probes:Gyroidoptimized,Crosshatchmultiline,
Archimedeansparse,correctenable_arc_fitting each control/targetwholePASSboth
modes. These are4nulls,not universal validation or separate SPECs. One earlier
misnamednon-GUI `arc_fitting` key caused only one addedfooterline in thecontrol;
its actualfailedcontrol/STOPtrace is retained/excluded,not relabelledparityRED.
CorrectGUIkeyenable_arc_fitting subsequentlynull. Acrossallretainedattempts:
42actualNativeSlice,21validatedAPI,21actualCLI,runtime0;one controllerassertFAIL
for invalid-keyfooter classification,not a hiddenproducererror. No trialpatch,
GDB/instrumentation/Native rebuild or numericalparameterhunt.

## Explicit exclusions

Avoid Type1/tower,fuzzy/startupfan/machine.rs,temperature,template/bed/
processor/excludename,skirt/brim/supportspeed and processor-word/FanMovertext.
No edits in those areas;this path is ordinary GCodeWriter XY emitter output,
not template expansion,FanMover regeneration,processor analysis or flowcalc.

**Same-file adjacency:** the brim flow work
changes the effective_flow call atline99 and adds effective_flow_at_emission
later in thisfile. This SPEC does NOT touch that calculation/call/helper or
its feature/state logic. Do not broaden patch into flowselection to makecommentsappear.

Defer nonzeroEpolicy,force-no-extrusionothercallers,arcs/XYZ/scarf,globalformatting,
flowratios/adhesion/supportspeed,all frozencontexts and unrelatedproducerdata.
Thelocalnonzero verbosecommentrepresentation is reused unchanged,not redesigned.

## Next staged gates

Freeze proved inputbytes/SHA,two ordinary unchanged shared
native_stream_case checks `(false,|_,_|{})`;Native2FIRST stable→validatedAPI/
actualCLI0/rawAPIeqCLI/unmaskedwhole equality. Reproduce dualcontrolPASS/
targetgenuineFAIL in test-only RED before production. NoNativeoutput
goldens/privateconstant-hunkassertions/ignore/inversion/commentmask/tolerance.
GREENmustcloseall29commentdifferenceswithoutflow/state
changes;unexpectedowner/residual→STOP.
