# SPEC-RR PROPOSAL: FanMover GCodeReader XYZ mode compatibility

## Status / exact base

**PROPOSAL only — no implementation/test code or Native run authorized yet.**
COORD requested one source-cited libslic3r/GCode parity slice,checked against
current main **acbb65b6961475c37b29466b306634c1694c8f13**.
New branch worker3/gcode-proposal-acbb65b69;no production changes.
Stock10 test-only f650049d0 remains separate/unmerged with sealed evidence.

Pinned upstream source:8500fcdccaa10b5099ac20d252af3a7c560046f1.
Candidate is **FanMover's private GCodeReader XYZ coordinate/distance contract
under G90/G91**,not a new motion planner,firmware parser or cooling scheduler.
A source-level mismatch is established;an end-to-end Native difference is
**not yet established**. Native-first qualification below is a hard prerequisite
before formal RED or any production implementation.

## Upstream owning boundary (file:line)

1. libslic3r/GCode.cpp:3727–3743 and3830–3841: existing per-layer FanMover
   filter,already ported tag handoff;no new pipeline path is proposed.
2. libslic3r/GCode/FanMover.cpp:22–32::process_gcode feeds real layer bytes to
   its GCodeReader callback;:291–304::_process_gcode_line reads dist_X/Y/Z
   for G0/G1 and forms existing mixed-precision segment time.
3. libslic3r/GCodeReader.hpp:129–135::parse_line invokes callback **before**
   update_coordinates;:43–45::GCodeLine::dist_X/Y/Z returns raw supplied axis
   value minus prior reader coordinate (or0 for absent axis).
4. libslic3r/GCodeReader.cpp:110–123::update_coordinates updates only exact
   uppercase G0/G1/G2/G3/G92 forms;for each present axis m_position=raw value.
   There is **no G90/G91 relative-XYZ state** in this reader's update contract.

This reader is a simplified upstream analysis helper,not the printer firmware.
Do not silently substitute physically preferable relative-XYZ interpretation
for its actual source semantics. This proposal ports the owning observable
FanMover helper behavior only;does not claim upstream's helper models every
valid firmware movement correctly.

## Verified current-main mismatch / not already ported

At exact acbb:
- gcode_emit/fan_mover.rs:79 stores relative_xyz;:116 initializes false.
- gcode_emit/fan_mover/motion.rs:11–15 changes it on G90/G91.
- motion.rs:22–42 uses relative axis values directly as deltas when true;
  :60–79 accumulates XYZ when true,instead of the cited raw-axis assignment.
- Existing FanMover.cpp source has no corresponding relative state.

Read actual acbb files,not old discovery binaries. git history of motion.rs on
acbb shows7212696b6 mixed-precision timing and95e230083 structural split;neither
removes the live G90/G91/relative_xyz branches. Main's merged tag handoff and
float-first abs are present and **not** this proposed change. Search current
fan tests/specs found no formal G91 reader contract witness. This is a specific
still-existing implementation difference,not a re-proposal of merged#86.

## Proposed Rust destination / included behavior

Exact destination:private
crates/ares-core/src/project_slice/gcode_emit/fan_mover/motion.rs and the
associated reader-state slot in fan_mover.rs. Existing FanMover/scaffold remains
a temporary compatibility shell around cited upstream GCode/FanMover+GCodeReader.

Included after independent RED/implementation approval:
- mirror upstream's lack of G90/G91 mode effect in this **private reader**;
- for the existing supported XYZ motion path,preserve raw-axis previous/current
  values and raw-minus-prior distance used by FanMover's existing time buffer;
- preserve callback/pre-update ordering,float coordinate storage and the already
  ported mixed-precision feedrate/distance time computation;
- do not change emitted custom G-code text or how actual printer/core/template
  motion state tracks G90/G91. No firmware-facing semantic reinterpretation.

The implementation shape is not pre-approved;first evidence must demonstrate
which private tracker operations are observable. Do not use a global reader or
writer policy override. Platform-neutral core boundary stays pure API/bytes,
portable to browser WASM,Windows,macOS,Linux,no filesystem/UI/Native/GDB in core.

## Explicitly deferred / STOP boundary

Defer all FanMover delay thresholds,split interpolation/rounding,kickstart,
fan flavor/PWM,custom-barrier policies,role tags,flush logic,E bookkeeping,
M82/M83,G92 edge cases,arcs,lowercase/unknown command parsing and parser grammar.
Defer processor physical-motion timing,placeholder evaluation,emitter template
state,validation/admission and all geometry/perimeter behavior.
Already merged per-layer tag/abs fixes stay frozen.

If control fails,Native rejects/varies,or first proven cause belongs to any of
those adjacent owners,STOP and report;do not repair neighboring behavior to
manufacture this slice. No output masks,empirical deltas or new Ares pipeline.

## GUI reachability and minimal Native-backed RED plan

GUI source:Tab.cpp:4629–4638 exposes editable Before layer change G-code;
Tab.cpp:4552–4556 exposes fan speed-up time and Only overhangs;
PrintConfig.cpp:3830–3845 defines fractional seconds. Plain G-code,not a hidden
key or placeholder-language feature. No conflict with main's recent expression
conversion patches;the candidate template contains no braces/expressions.

Input origin:existing tests/parity/fan-layer-tag/control.3mf SHA
17896c6fdf4d5d570c604843fafd049cbc1ac6cde86a21c07dd20ebfdd8a13b2.
Existing stored Standard-Mine Bambu profile is customized,not pristine factory;
geometry/modifier4/placement and unrelated config remain frozen.

Proposed two-input qualification (not generated/run at this stage):
- Both share existing fanmin/max20,OnlyOverhangs0 and a common GUI
  before_layer_change_gcode containing a balanced relative small move:
  `G91\nG1 X1 F6000\nG1 X-1 F6000\nG90\n`.
  This restores the physical X position and absolute-XYZ mode,contains no E or
  fan commands and changes no geometry. Preserve existing layer_change_gcode.
- Control fan_speedup_time=0;target=+3. Pair differs ONLY that scalar after
  common GUI template is fixed. No synthetic machine/process/filament presets.
- CPU12/jobs1/threads1,fresh evidence roots;official then seeded0. Per input:
  actual Native slice twice and complete normalized replay stability first.
  Native must accept project/template and produce complete G-code/footer.
- Only then run public validated API+actual CLI at exact acbb-derived tip;
  require success and raw API==CLI. Compare complete ordered Native bytes with
  only original generator/date normalization,including time/M73/E/F/custom
  commands/fans/start/end/config/footer. Native failures are not Ares RED.
- Expected hypothesis:disabled control fullPASS;enabled target fullFAIL from
  differing private reader deltas/buffer state. This expectation is not evidence.
  If the pair does not reproduce,record bounded null;no speculative code fix.
- If needed to attribute first difference,only bounded read-only observation of
  existing binaries,without inferior calls/state mutation/instrumentation;whole
  observer outputs must remain canonical. Compare actual coordinates/deltas at
  the same captured input line,not breakpoint-event counts as layer identity.

Only after Native-backed qualification:copy the two proven input bytes+SHA,
formal two direct unchanged native_stream_case::check calls `(false,|_,_|{})`;
no Native output goldens/private counter assertions/inversion/ignore/fallback.
Formal dual modes should show1PASS1genuineFAIL before any implementation.
Reuse existing shared checker/normalizer;preserve unexecuted-tail honesty when
target API equality panics. Exact directed/gate evidence then separate RR.

## Request

Approve bounded Native-first qualification of this ONE private-reader source
slice and,if proven,the docs-first/formalRED stages separately as COORD chooses.
Do **not** infer GREEN/code authorization from this proposal. No Native/Ares
producer run,fixture generation,core/test implementation or FULL has occurred.
This proposal checks acbb state and defines a falsifiable source-owned E2E plan;
it is not a claim of a new already-reproduced production defect.
