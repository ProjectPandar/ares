# AGENTS.md

This file provides guidance when working with code in this repository.

## Project Overview

Ares is a Rust implementation of the Orca Slicer. It provides G-code generator for 3D printers.

### Lint
Make sure pass the clippy and rustfmt checks via: `cargo clippy` and `cargo fmt` after edit files.

Split modules into multiple files when files grow too large. Start split when file exceed 400 line of file (LOC).

### Workspace Crates

Active workspace members:
- `crates/ares-core`: Platform-neutral core slicing API crate. It owns the Rust rewrite of `libslic3r` concepts, slicer data models, option handling, and the async byte-in/options to byte-output API. It must run on WASM, Windows, macOS, and Linux without direct file I/O, UI, OpenGL, or terminal behavior.
- `crates/ares-cli`: Command-line adapter crate. It owns argument parsing, filesystem input/output, and terminal-facing behavior while calling `ares-core` for slicing work.
- `crates/ares-vgcode`: Rendering-neutral Rust rewrite of `OrcaSlicer/src/libvgcode` data. It owns G-code input data, path vertices, layer/range/color data, and role vocabulary for UI/viewer consumers, but must not contain OpenGL, native viewer runtime, filesystem, terminal, or slicer logic.
- `crates/ares-wasm`: Browser WASM adapter crate. It owns wasm-bindgen byte-oriented bindings around `ares-core` and rendering-neutral data exposure, but must not contain filesystem access, terminal behavior, OpenGL/viewer runtime, or independent slicing pipeline logic.

Candidate crates are not workspace members until a milestone creates them:
- Optional geometry/config split crates: Create only if `ares-core` module size, compile boundaries, or reuse pressure makes extraction simpler than keeping modules inside core.

Do not add candidate crates to `Cargo.toml` or this active list before the milestone that creates them.

### Future Milestone Rewrite Gate

All future milestones must be planned as source-cited Rust rewrite slices of `OrcaSlicer/src/libslic3r` or `OrcaSlicer/src/libvgcode`, not as independently designed Ares pipeline features. Every milestone spec and plan must name the upstream file(s), class(es), function(s), or data structure(s) being ported, define the Rust destination boundary, list included and deferred upstream behavior, and explain how any existing Ares scaffold is being replaced, renamed, or used only as a temporary compatibility shell around an upstream concept. Milestones that add slicing, G-code, configuration, or viewer-data behavior must first identify the owning `libslic3r`/`libvgcode` source boundary; milestone text must continue to frame those changes as upstream rewrite slices, not as Ares-owned pipeline design. Designing an Ares-owned pipeline is not an acceptable milestone goal. Independent reviewers must reject milestone specs/plans that primarily grow an Ares-owned pipeline instead of moving toward `libslic3r`/`libvgcode` parity.

For future milestones, the rewrite boundary stays with `libslic3r` and `libvgcode` port slices only; do not invent a new Ares pipeline as the milestone goal. If a milestone spec or plan cannot cite an upstream file/class/function/data structure and the exact Ares destination boundary, it is incomplete and must be rewritten before implementation starts. Future milestone docs must also state which upstream behavior is included, which adjacent behavior is deferred, and that existing Ares scaffolding is only a temporary compatibility shell when it does not yet match the cited `libslic3r`/`libvgcode` source boundary.

This rewrite-gate requirement applies to the continuing `PrintConfig.hpp` and `PrintConfig.cpp` milestone chain as well as later `libslic3r` / `libvgcode` slices: keep milestones source-cited, upstream-boundary-first, and free of self-designed Ares pipeline goals.

## Document

### docs/architecture/*.md
将架构决策、审查技术方案文档放在 architecture 中. 在分析时编写 ARD 文档, 将不可协商的架构决策放在 ARD 文档中.

### docs/roadmap.md
编写路线图、排列优先级、撰写里程碑退出标准，将该内容存放在 roadmap.md 中.

## 项目测试
遵循 Test Driven Development 进行代码的编写，需要使用原 OrcaSlicer 来做 End to end (E2E) 测试。
使用 `cargo nextest run` 运行 Rust 测试；全量测试使用 `cargo nextest run --workspace`，不要用 `cargo test` 作为默认测试入口。Nextest 并行度由 `.config/nextest.toml` 配置为 `test-threads = "num-cpus"`。

## 项目要求
- 目前的 Tier 1 平台为: WASM (浏览器), Windows, macOS, Linux. 需要支持这四个平台. 尤其是 WASM, 需要能在浏览器环境下运行切片
- 项目方向是对 OrcaSlicer 的 `libslic3r` 和 `libvgcode` 做 source-cited Rust rewrite；后续 milestone 必须以明确的上游文件、类、函数或数据结构为边界推进，不允许把自研 Ares pipeline 当成主要设计目标。
- 逻辑和 UI 分离, 新的 Rust 重写应该将核心逻辑编写成 API, UI 应该调用 API. 目前 C++ 部分 UI 代码和逻辑耦合严重, 需要 UI 低耦合
- 性能优化, 参考 C++ 部分, 编写高效的代码, 不要使用简单但是更耗性能的方式实现代码
- 最大化利用 Rust 语言最新的 feature, 使用零成本抽象等优化
- No legacy fallback

## Avoid over-engineering. Only make changes that are directly requested or clearly necessary. Keep solutions simple and focused.
- Don't add features, refactor code, or make "improvements" beyond what was asked. A bug fix doesn't need surrounding code cleaned up. A simple feature doesn't need extra configurability. Don't add docstrings, comments, or type annotations to code you didn't change. Only add comments where the logic isn't self-evident.
- Don't add error handling, fallbacks, or validation for scenarios that can't happen. Trust internal code and framework guarantees. Only validate at system boundaries (user input, external APIs). Don't use feature flags or backwards-compatibility shims when you can just change the code.
- Don't create helpers, utilities, or abstractions for one-time operations. Don't design for hypothetical future requirements. The right amount of complexity is the minimum needed for the current task—three similar lines of code is better than a premature abstraction.
- Avoid backwards-compatibility hacks like renaming unused _vars, re-exporting types, adding // removed comments for removed code, etc. If you are certain that something is unused, you can delete it completely.


## DO NOT OVER-DEFEND

- Only add defensive checks (null/nil/None checks, type guards, boundary validation) at true system boundaries — public API entry points that accept external, untrusted input.
- Do not add defensive checks in internal/private functions, constructors called only by your own code, or test helpers.
- Do not add defensive copies unless the data is genuinely shared across trust boundaries.
- Omitting a defensive check is not a bug — it is a deliberate signal that the caller is trusted.

## USE MODERN LANGUAGE FEATURES

- Write idiomatic code for the language version specified by the project. Do not write code that targets an older version out of habit.
- Prefer language-level constructs that reduce boilerplate: pattern matching, destructuring, algebraic data types (sealed types, tagged unions, enums with data), data classes/records/structs, and built-in concurrency primitives.
- If the language provides exhaustiveness checking (e.g., sealed types + switch, match expressions, tagged unions), use it. Compiler-enforced completeness is better than a default/else branch that hides missing cases.
- Do not manually write what the language generates for free (toString, equality, hash, serialization).

## Code Rules

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

### 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

### 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

### 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

### 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.
