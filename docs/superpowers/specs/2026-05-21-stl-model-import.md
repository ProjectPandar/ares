# STL Model Import Spec

## Goal
Advance Ares from format-label placeholder slicing to a real byte-level STL import stage that `ares-core` can use before later layer planning and G-code parity milestones.

## Context and OrcaSlicer evidence
- `OrcaSlicer/src/libslic3r/Format/STL.cpp` loads STL into `TriangleMesh` and then into `Model`, making STL import the first boundary before slicing.
- `OrcaSlicer/src/libslic3r/Format/STL.hpp` exposes `load_stl` as a model-import function rather than as a CLI operation.
- `OrcaSlicer/tests/libslic3r/test_stl.cpp` covers ASCII and binary STL reads, including LF/CRLF ASCII variants and nonstandard ASCII files.
- `OrcaSlicer/src/libslic3r/TriangleMesh.*` and `Model.*` show that imported triangles become core model data consumed by slicing stages.

## Scope
Milestone 2 implements STL byte import inside `ares-core` only. It does not add a new crate, does not perform file I/O in core, and does not attempt full 3MF ZIP/XML parsing. 3MF remains accepted by the existing command path as detected project bytes, but geometry extraction for 3MF is a later milestone because it requires archive/XML handling and broader project metadata decisions.

## Functional requirements
1. `ares-core` exposes a public model import API:
   ```rust
   pub fn load_model(input: impl AsRef<[u8]>) -> Result<Model, SliceError>
   ```
2. `Model` stores the detected `InputFormat` and a list of imported triangles.
3. `Triangle` stores three vertices as `Point3 { x, y, z }` using `f32` coordinates.
4. ASCII STL parsing supports LF and CRLF input with `vertex x y z` lines and creates one triangle per three vertices.
5. Binary STL parsing supports the standard 80-byte header, little-endian `u32` triangle count, 50-byte triangle records, and ignores the two-byte attribute count. Binary STL bytes do not need to start with `solid`; `load_model` must route standard binary STL records to the STL parser before rejecting unknown input.
6. `load_model` rejects empty input with `SliceError::EmptyInput`.
7. `load_model` rejects malformed STL bytes with a typed error containing a concise message.
8. `slice` calls `load_model` and includes deterministic model metadata in placeholder G-code:
   - `; input_format = stl`
   - `; triangle_count = N`
   - `; option_count = N`
9. Existing `SliceOptions` behavior stays unchanged and continues preserving arbitrary Orca option keys.
10. `ares-cli` continues to own filesystem and extension checks and continues to call only the core public API for slicing.

## Non-goals
- No real layer planning, extrusion, movement generation, support generation, profile typing, or Orca G-code parity in this milestone.
- No 3MF archive/XML model extraction in this milestone.
- No new workspace crates in this milestone.
- No native filesystem access in `ares-core`.

## Acceptance criteria
- Core tests cover LF ASCII STL import, CRLF ASCII STL import, binary STL import, malformed STL rejection, and `slice` output containing imported triangle counts.
- CLI tests cover STL output containing `triangle_count` metadata.
- M2 milestone docs describe the narrow STL import exit criteria and defer 3MF geometry extraction explicitly.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
