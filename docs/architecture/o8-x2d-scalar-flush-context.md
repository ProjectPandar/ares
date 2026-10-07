# O8-X2D-SCALAR: unresolved original flush-context divergence

## Status and owner

**Open, pi-lint; not solved by X2D-FM.** Separate follow-up slice, pending
coordinator contract/spec approval. Removing its test from the defined-input
FM module does not waive the original divergence or close the reported issue.
Preserve the admitted test-only RED `65336cb594253ca0c4b54942719927cdf4c24bd8`
and all `case-LqoYic` artifacts. No ignored test or numeric fallback is added.

## Exact input and fresh observation

Repository-cube10 X2D input:
`/tmp/ares-x2d-flush-matrix/red/seeded/case-LqoYic/input.3mf`.
Original admission and input SHA remain in `red/admitted-red.json`.
This is a **test-constructed input, not a real GUI-saved 3MF**: the test exports
repository `tests/parity/cube10.stl` with the stock X2D machine/process/filament
profiles through Native CLI, binds two logical filaments, then rewrites
`Metadata/project_settings.config` (including the scalar nozzle-volume type and
four-entry matrix) and adds explicit end-G-code tool switches. It is not a
cropped customer GUI project. O8-X2D-SCALAR is scheduled after FM; genuine GUI
provenance, if subsequently established for another reproducer, raises priority.

Fresh source base `d789b0750` includes `458938aa4`; candidate test-only HEAD
`304bda1ebd9a1984f3cf9f01270424298c57f4fc` plus uncommitted FM preparation.
Public `slice_project` on the exact original bytes returns
`Flush volumes matrix do not match to the correct size!` after the any-vector
coverage rule preserves the original matrix. It does **not** succeed.

Two prescribed seeded Native diagnostic invocations both succeed. Their logs
show `unsupported NozzleVolumeType=30898` and `=6` respectively, between
`flush_volumes_matrix before computing: 0,140,140,0` and the reconstructed
`0,339,621,0,0,339,621,0`. Complete output differs only at the generator
clock line. This establishes repeatable observed output for these two runs,
**not deterministic or defined C++ behavior**: the invalid enum value changes.

Fresh evidence:
`/tmp/ares-x2d-flush-matrix/rebase-458/original-input/` contains input and
source hashes, production diff, fresh build log/rlib hash, public API adapter
result, actual Native outputs, stdout/stderr/debug logs and full two-run byte
difference inventory. Ares error versus Native success remains an explicit
unresolved behavioral difference.

## Owning upstream boundary and Rust destination

Revision `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `OrcaSlicer/src/OrcaSlicer.cpp:1708–1718,3022–3047`: current raw type vector
  versus new CLI types; new-vector filling does not resize the current vector.
- `:3340–3351,3457–3462`: reconstruction predicate and unchecked per-nozzle
  volume-type/dataset selection. The original type vector has one entry for
  two nozzles; Native reads entry1 out of bounds.
- `libslic3r/PrintConfig.cpp:9004–9060`: variant lookup used by dataset selection;
  no inferred first-id or zero-dataset replacement may be introduced.
- `libslic3r/GCode.cpp:5596–5614`: unchanged scale/size guard and exact error.

Rust destination is the existing raw project/CLI preparation context and
`ares-core/src/options/flush_matrix.rs`, not emitter geometry or a new pipeline.
This follow-up must first define an approved source-owned handling contract for
this undersized input. The any-vector nonrebuild rule alone does not solve parity.
Do not derive that contract from observed numeric values, emulate UB, relax the
matrix guard, or silently replace the original input with a complete vector.

## Included, deferred and exit

Included: original-input fresh-main E2E RED, precise current/new context and
source prerequisite tracing, and an explicitly reviewed resolution contract.
Deferred: generic generated-id materialization, timing, flush emission, geometry,
UI, arbitrary input canonicalization and broad UB policy.

Before production, get coordinator approval for the specific resolution contract
and retain a separately bound RED. Exit requires an accepted disposition of the
original same-input Native-success/Ares-error difference, complete byte evidence,
unchanged controls and ordinary Tier1/lint/workspace gates. An FM GREEN covering
only complete-vector inputs must cite this open follow-up rather than claim the
original reported input is fixed.
