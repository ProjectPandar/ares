# ARD: reuse upstream mode-dependent template layer total

## Status and source

Status: accepted SPEC, docs then formal RED above frozen `117f4ffe3`;
RED precedes GREEN. Owning source: pinned OrcaSlicer
`8500fcdcc` `GCode.cpp::_do_export:2484–2528` (`m_layer_count`) and `:2890`
(`total_layer_count` placeholder), consumed by layer templates at `:4677–4684`.
Rust boundary: private `gcode_emit/orchestration.rs::emit`, existing
`EmitState.total_layer_count` field initializer only.

## Non-negotiable decisions

1. Reuse existing `tail_layer_count`; no second counter, shadow scheduling,
   public API, fallback, template special case or independently designed pipeline.
   The wrong existing scalar handoff is a temporary upstream compatibility shell.
2. Keep all layer geometry, ordering, numerators, file-start/header consumers,
   writer, processor, M73, templates, evaluator and `value.rs` unchanged.
3. Use Native-success stock A1 ByObject as the count-only target. Require
   stock A1 ByLayer and the original P1S ByLayer complete-output controls.
4. Use unchanged Native-first public API/CLI runner and complete ordered-byte
   generator-only comparator. Replay both Native witnesses before comparison;
   raw API/CLI equality and successful exits are mandatory even for RED.
5. Commit docs before tests. Later GREEN changes exactly one initializer and
   no tests, fixtures, helpers, comparator, runner or neighboring comments.
6. No full parity or integration in this phase.

## Deferred and consequences

P1S ByObject's Y/M73 compound residual and broader skirt control are separate
work; do not mask their differences here. Broader instance/tool/support/raft
runtime qualification and other count consumers are deferred. The one-field
handoff retains existing platform-neutral ownership without additional state
or performance cost. Linux evidence alone does not claim all-platform runtime
coverage. See [SPEC](sequential-template-layer-count-spec.md) for failure modes,
fixtures, verification and later GREEN acceptance.
