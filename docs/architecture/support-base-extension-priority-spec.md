# Shared support-base extension priority: bounded rewrite slice

## Authority and source boundary

COORD authorizes SPEC -> tests-only RED -> minimal GREEN -> REVIEW-REQUEST/HOLD, no FULL. Parent is frozen `ccc79c31430bc9c0d7556158e7a17c7f219eaa68`, not discovery maina6232a1df. Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

Actual default-tree route: TreeSupport3D -> SupportCommon.cpp::generate_support_toolpaths -> FillRectilinear.cpp:3610–3634::FillSupportBase::fill_surface -> FillBase.cpp:2247–2708::Fill::connect_base_support -> :2326 call of :1834–1948::base_support_extend_infill_lines. Destination is existing `crates/ares-core/src/fill/base_support/extend.rs`; no new Ares pipeline.

## Literal behavior and proof

At upstream1914–1915, `dist_y_prev < dist_y_next ? extend_prev_idx : extend_next_idx = -1;` means `cond ? prev : (next = -1)`, not assignment to the selected lvalue. Previous is never invalidated. Existing previous-first dispatch therefore selects valid previous even when next has greater vertical rise. Rust currently clears previous in that case, based on its misleading :1925 comment. That longest-side scaffold must be replaced with the literal upstream effect.

Sealed diagnosis: `/home/indexyz/ares-evidence/worker1/tree-support-connector-localization-a6232a1df/ROOT-CAUSE-AND-LITERAL-FIX-REQUEST.md`, SHA256b5f86720a87e87fdcdb7e4d5c419fa126c7151fb7c0582bb11b312094b307ce3; seal349b5805793e296535209ee506d52f6f2b1ffbedb6ad08e89e9ba7798ef9d66d. Equal input, graph, marking, extension-entry and exact predecision states. At call3/Z0.6, endpoint42 accepts previous41/rise2035351 and next54/rise2447021; Native selects41, Rust54. It is the only both-valid/previous-shorter decision among78. Extension is the first unequal phase, before costs/linking. Relevant Rust sources are identical on discovery main and frozenccc.

## Included and deferred behavior

Included: in the existing both-valid block, only invalidate next when previous rise is not less than next (integer `>=`); never invalidate previous. Remove previous binding mutability and correct the comment to cite :1915 and its actual parse; correct the affected module description. Preserve all probe return shapes and existing dispatch.

Deferred/unchanged: candidate validity, thresholds, contour walking, arc-fit, graph creation, marking, trimming, later arc consumption/costs/traversal, rotation, tree generation, G-code/writer behavior, new fixtures and broader parity. Do not repair upstream intention or add compatibility switches/helpers. Existing longest-side selection is an incompatible temporary rewrite shell, not a design to retain.

## Agreed E2E seam and fixture scope

Use `runner::native_stream_case` only, with full original Native replay and unchanged public API+CLI generator-only ordered-byte comparator; assert raw API==CLI. Frozen discovery projects (same K1/overhang input) become committed fixtures: tree(auto), stock support-disabled, normal(auto). SHA256/metadata manifest records original bytes and source settings. Target must have actual Native support toolpaths; controls distinguish support-disabled and ordinary automatic support.

In **each phase**, first complete Native double replay for all three cases in **both** official and seeded modes before any public API/CLI call. Then run three public tests per mode, each again using native_stream_case double replay before API/CLI. Preserve complete streams, comparisons, exact selected-test lists, exit codes, build identities and requested affinity receipts. No Native total-threadcount1 or exhaustive ELF/TID claim.

RED exit: both mode preflights stable; stock and normal controls whole-output pass; tree target API and CLI both slice successfully and raw-match each other but both genuinely fail unchanged full-output comparison at the known support-path divergence. Admission failure, instability or unrelated failure is STOP, not acceptable RED. Production source remains byte-identical to parent.

GREEN exit: apply only the included literal block/comment change after RED; same six case/mode outcomes all whole-output pass with raw API==CLI. Run rustfmt, clippy, workspace all-target check and browser-WASM core/vgcode/adapter check; use cargo nextest for bounded E2E. No FULL. Retain existing frozen reset/spiral stack and record that integration qualification is separately gated. If a second independent defect appears, STOP rather than widen.

## Delivery

Separate source-cited SPEC, tests-only RED and minimal production GREEN commits; sealed REVIEW-REQUEST with Native-first order, all phase artifacts, source/diff audit and residual platform limits. Then HOLD. Windows/macOS runtime tests are not claimed by Linux checks; the core change remains platform-neutral, with WASM compile checked.
