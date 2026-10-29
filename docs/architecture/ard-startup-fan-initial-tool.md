# ARD: consume initial filament identity for startup fan-off

Status: proposed, docs-only SPEC-RR on main957e963c3. No implementation authority.

## Evidence-led context

Two earlier discovery nulls do not justify speculative production work. The new
three-case GUI-stock-edit probe instead establishes four successful-output whole
differences (target API/CLI in both modes), eight whole control equalities and six
raw API==CLI equalities, after all twelve Native reference/replays globally first.
The complete target delta is the missing19-byte startup fan-off pair. See the
companion SPEC and immutable external discovery report/audit/localization.

## Non-negotiable rewrite boundary

1. Upstream source pin8500fcd, `GCode::_do_export`, `GCode.cpp:2822–2826`: consume the already-selected
   initial filament's `close_fan_the_first_x_layers` through existing indexed access
   when auxiliary capability is on. Writer1095–1155 owns existing fan-off bytes.
   Rust destination is machine.rs::append_limits and its existing startup caller.
2. The unconditional first-slot read is a temporary compatibility shell around
   that upstream consumer, not an Ares-owned policy. Replace only that read/handoff;
   do not redesign tool selection or introduce a registry/fallback/public API.
3. Reuse `InitialFilaments::initial()`, not default writer active0 before selection.
   Keep startup order, capability gate and existing serialization. Generic get_at,
   layer cooling, FanMover, additional-fan scheduling and PWM/flavor policy stay out.
4. The oracle remains whole ordered repository comparison plus raw API==CLI.
   Diagnostic deletion proving localization must NEVER become a test allowance,
   normalization rule or production output rewrite. No hardcoded Native goldens.
5. Future tests must precede any production change and exercise actual public
   validated API/CLI through existing native-stream seams. No unit/private helper
   substitute. Separate RED and GREEN authority remain required.
6. W2/W5/W6/W4 scopes and adjacent W3 fan-buffer work remain separate. Later minimal
   shared orchestration/registration changes require coordination, not ownership of
   their tool-ordering, timing, temperature, label or adhesion policies.

## Alternatives rejected / consequences

Do not add a generic fan state machine, scan positive extrusion to choose startup
identity, use logical slot0 as a fallback, change the tool ordering, or waive the
two omitted commands. Existing identity and serialization already suffice.
The discovery proves the narrow stock startup mismatch, not all fan/filament/
platform parity. Preserve every artifact and sampled-ELF coverage limitation;
CPU18 serial controls do not prove a complete process/thread census.
