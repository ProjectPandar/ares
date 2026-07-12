# M852 OrcaSlicer source crate partition checkpoint Implementation Plan

Status: Complete

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Record a source-cited crate partition checkpoint for the Rust rewrite based on OrcaSlicer source/build boundaries.

**Architecture:** This is a documentation-only milestone. It confirms the current four-crate workspace and blocks speculative crate splits until a future source-cited milestone proves a concrete boundary. It must not create a crate, move Rust modules, change `Cargo.toml`, change `AGENTS.md`, or design an Ares-owned pipeline.

**Tech Stack:** Markdown architecture docs, roadmap, milestone/spec/plan docs, git diff verification.

---

## Rewrite gate mapping

Source boundary:

- `OrcaSlicer/src/CMakeLists.txt`
- `OrcaSlicer/src/libslic3r/CMakeLists.txt`
- `OrcaSlicer/src/libvgcode/CMakeLists.txt`
- `OrcaSlicer/src/slic3r/CMakeLists.txt`

This plan preserves the `libslic3r`/rendering-neutral `libvgcode` rewrite boundary and treats `slic3r` as UI/API-consumer evidence only.

## Files

- Create: `docs/architecture/ard-0022-orcaslicer-source-crate-partition-checkpoint.md`
- Modify: `docs/architecture/orcaslicer-source-structure.md`
- Create: `docs/milestones/m852-orcaslicer-source-crate-partition-checkpoint.md`
- Modify: `docs/roadmap.md`
- Create: `docs/superpowers/specs/2026-05-31-orcaslicer-source-crate-partition-checkpoint-m852.md`
- Create: `docs/superpowers/plans/2026-05-31-orcaslicer-source-crate-partition-checkpoint-m852.md`

Do not modify `Cargo.toml` or `AGENTS.md` in M852 because no new crate is created and the current `Workspace Crates` section already matches the accepted active workspace.

### Task 1: Record source/build evidence

- [x] **Step 1:** Add an ARD that cites the OrcaSlicer build/source boundaries and records the accepted four-crate partition.
- [x] **Step 2:** Update `orcaslicer-source-structure.md` with build-system evidence explaining why `libslic3r`, rendering-neutral `libvgcode`, and `slic3r` map to different Ares ownership boundaries.
- [x] **Step 3:** Add M852 to `docs/roadmap.md` and create its milestone file.

### Task 2: Review and verify documentation-only milestone

- [x] **Step 1:** Request independent planning review. The verdict must start with `APPROVE` before treating the plan as accepted.
- [x] **Step 2:** Run verification:

```bash
git diff --check
```

- [x] **Step 3:** Request independent implementation review. The verdict must start with `APPROVE` before commit.
- [x] **Step 4:** Commit and push the documentation milestone after approved review.

### Task 3: Completion status

- [x] **Step 1:** Mark this plan `Status: Complete` and switch all task checkboxes to `[x]` after the approved documentation commit. Keep the header text `Steps use checkbox (`- [ ]`) syntax for tracking.` unchanged.
- [x] **Step 2:** Commit and push the completion-status update separately.
