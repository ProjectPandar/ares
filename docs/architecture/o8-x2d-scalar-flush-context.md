# O8-X2D-SCALAR: unresolved original flush-context divergence

## Status and owner

**Open, worker1 (formerly pi-lint); spec/disposition review only.** Not solved
by X2D-FM or generated printer ids. Separate follow-up slice, pending
coordinator approval of the handling contract below. Removing its test from the defined-input
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

Historical source base `d789b0750` includes `458938aa4`; candidate test-only HEAD
`304bda1ebd9a1984f3cf9f01270424298c57f4fc` plus uncommitted FM preparation.
Public `slice_project` on the exact original bytes returns
`Flush volumes matrix do not match to the correct size!` after the dependency
coverage rule preserves the original matrix: this two-colour input needs
non-diagonal type/dataset calculations. It does **not** succeed.

Two prescribed seeded Native diagnostic invocations both succeed. Their logs
show `unsupported NozzleVolumeType=30898` and `=6` respectively, between
`flush_volumes_matrix before computing: 0,140,140,0` and the reconstructed
`0,339,621,0,0,339,621,0`. Complete output differs only at the generator
clock line. This establishes repeatable observed output for these two runs,
**not deterministic or defined C++ behavior**: the invalid enum value changes.

Historical captured evidence (not a new-main run):
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
this undersized input. Preserving cells with unresolved dataset dependencies
alone does not solve the original parity difference.
Do not derive that contract from observed numeric values, emulate UB, relax the
matrix guard, or silently replace the original input with a complete vector.

## Undefined-read boundary and proposed handling contract

This section is a **proposal for coordinator review**, not permission to alter
production or a declaration that the original parity difference is fixed.
Current integrated base is `cbee8353998283332cb838f2be3b41b826d8ef66`.

### What is undefined, and what is not

The preserved input SHA is
`b980548a6170e454a0b0701c115aecbae3441487c35b92fcd64e8c50b50595b2`:
`nozzle_diameter=[0.4,0.4]`, `nozzle_volume_type=[Standard]`,
`extruder_type=[Direct Drive,Bowden]`, ids[1,1,2,2], four variants,
two ordinary non-support colours, and a four-entry supplied flush matrix.
The generated-id map is complete; the newly integrated generic selector does
not repair the one-entry nozzle-volume vector or define an out-of-range read.

`OrcaSlicer.cpp:1708–1718` copies exactly the stored type count into the current
vector. At3022–3047 the new CLI vector has two Standard entries when no type
override exists; only an **empty** current vector is filled. A nonempty
one-entry current vector is not resized. Their inequality activates3340–3351's
flush reconstruction predicate even though a matrix is present.

The critical raw selector copy at3450–3458 is separate from that new CLI vector.
It retains the one serialized type (unless explicitly overridden). In3460–3462,
`eidx=1` calls `volume_types[1]` on that one-element std::vector: an unchecked
out-of-range read with C++ undefined behavior. `get_at` elsewhere cannot supply
a guarantee here. If the invalid enum makes9004–9060's lookup return -1,3463's
`nozzle_flush_dataset_full[index]` is another unchecked read; unlike the later
Print::apply invocation, this CLI call does **not** map -1 to0 first.
Do not import PrintConfig.cpp9677–9681's call-specific handling into this path.
Adjacent `extruders[eidx]` is also unchecked, but this exact original input
contains both extruder entries; it is not the missing prerequisite here.

A one-element vector is not globally illegal: O3 loading validation, option
serialization and other source `get_at` consumers remain valid and authoritative.
Only a calculator invocation needing the unavailable per-nozzle selector is
outside the defined upstream domain. Native's changing invalid enums30898/6,
coincidentally equal reconstructed values and seed/CPU pinning do not turn UB
into defined arithmetic or a reproducible contract. Return status0 is likewise
not evidence of a valid index. No new global scalar-option rejection is implied.

### Proposed disposition: preserve, never synthesize

Keep the already approved dependency rule at the shared byte/options boundary:

1. Retain the original raw option shape, old/new preparation context and supplied
   matrix. No automatic Standard extension, `get_at` substitution at the
   unchecked selector, palette heuristic, retry or UB/allocator emulation.
2. Inspect actual calculator dependencies, not printer names or palette count.
   If an off-diagonal non-support cell needs a missing selector, decline
   reconstruction before selecting any dataset. Do not invent dataset0 or reuse
   the previous nozzle's selector. Independent source multiplier resizing stays
   independent. Dataset-free diagonals/support constants remain computable under
   the FM contract; this proposal must not regress that boundary.
3. Let the unchanged downstream GCode size/scaling/rounding authority consume
   the preserved matrix. For this original four-entry/two-nozzle case it returns
   the existing exact error `Flush volumes matrix do not match to the correct size!`.
   Do not bypass that guard, pad the matrix or return Native's observed numbers.
   A supplied valid-size matrix with unresolved selector context is preserved;
   it is not globally rejected solely for having one type entry.
4. Classify the original Native-success/Ares-error result as **source-UB input,
   unsupported automatic reconstruction**, not Native-output parity PASS.
   Preserve the admitted RED, both Native outputs, first UB boundary and Ares
   outcome. Only an explicit coordinator disposition can close tracking#56 as
   a documented unsupported-input contract; it must not claim numerical parity.
5. A caller can explicitly supply per-nozzle types (e.g. two Standard entries)
   through the existing configuration interface. That is a **different effective
   option contract**, with its own input/override identities and complete Native
   byte comparison. It is not an implicit repair of, or substitute evidence for,
   the original witness. Genuine GUI provenance for a separate project stays a
   separate investigation rather than changing this input's provenance.

This handling is a bounded safe-Rust disposition of an upstream UB precondition,
not an independently designed Ares slicing algorithm. Existing defined-input
FM and generated-id behavior must remain byte-identical. No new API field,
loader bypass, compatibility fallback, emitter patch or configurable UB mode.

### Spec-first verification and exit decision

After exact-hash spec approval, capture a fresh current-main same-input public
API outcome with input/source/build hashes and Native producer/resource identity.
One source/Native trace establishes actual vector sizes and call location; do
not retry until a desired garbage enum appears. Already preserved UB executions
remain diagnostic evidence and are not a valid expected-G-code oracle.

Before any necessary production edit, separately bind tests to the approved
public bytes/options seam: original-input **defined rejection** (not a parity
claim), an explicit fully specified selector control using Native full bytes,
and supplied-valid-matrix preservation. This must not delete/relax/ignore the
historical parity RED or reuse the same label to turn its verdict into PASS.
If current code already implements the approved handling, prefer no production
change; record its verified behavior and close only by explicit disposition.
Otherwise new test-only RED precedes the minimal source-precondition correction.

Failure inventory: pretending CLI[new] vector repairs raw[current] storage;
confusing `get_at` with operator[]; importing Print::apply's -1→0 policy into
CLI; extending vectors or padding matrices; breaking dataset-free constants;
bypassing O3 or the exact matrix-size error; hiding original UB under a modified
fixture; claiming seed pinning defines UB. Verify unchanged whole-output FM and
generic generated-id controls, and all affected lint/Tier1/workspace gates if
production changes. Complete Native byte parity remains mandatory for defined
inputs, and explicitly inapplicable to the OOB invocation itself.

## Included, deferred and exit

Included: original-input fresh-main E2E RED, precise current/new context and
source prerequisite tracing, and an explicitly reviewed resolution contract.
Deferred: changes to the now-integrated generic generated-id materialization,
timing, flush emission, geometry, UI, arbitrary input canonicalization and broad UB policy.

Before production, get coordinator approval for the specific resolution contract
and retain a separately bound RED. Exit requires an accepted disposition of the
original same-input Native-success/Ares-error difference, complete byte evidence,
unchanged controls and ordinary Tier1/lint/workspace gates. An FM GREEN covering
only complete-vector inputs must cite this open follow-up rather than claim the
original reported input is fixed.
