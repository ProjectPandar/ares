# Runtime Native incident-pointer parity

Source-owned prerequisite: matching Boost1.86 `voronoi_diagram.hpp::_build:470-475`
→ `vendor/boostvoronoi/src/diagram/diagram_impl.rs::finish:136-144`.
Plan: `docs/architecture/painted-voronoi-incident-prerequisite-spec.md`.
This test does not qualify full Orca issue-detection/rotation parity or public
painted admission. It compares ONE actual rectangle geometry, repeated50 times
in each of two fresh seeded Native processes, against the workspace backend.

## Mandatory test / retained evidence

Run from repository root with the v2.5.0 seed-replay recipe's ELF and matching
separate DWARF, GDB Python/ptrace access, Python3 and a clean upstream checkout at
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
Set the shell variables below to your installation paths and one available CPU;
there are no machine-path or CPU defaults in the harness:

```sh
RUSTUP_TOOLCHAIN=1.99.0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 \
ARES_REQUIRE_ORCA=1 \
ARES_ORCA_BIN="$ORCA_WRAPPER" \
ARES_ORCA_NATIVE_BIN="$ORCA_NATIVE_ELF" \
ARES_ORCA_RESOURCES_DIR="$ORCA_RESOURCES" \
ARES_ORCA_DEBUG_DIR="$ORCA_DEBUG_DIRECTORY" \
ARES_ORCA_DEBUG_FILE="$ORCA_DEBUG_FILE" \
ARES_ORCA_NATIVE_SHA256="$ORCA_NATIVE_SHA256" \
ARES_ORCA_DEBUG_SHA256="$ORCA_DEBUG_SHA256" \
ORCA_REPLAY_CPU="$CPU" \
ARES_ORCA_SOURCE_DIR="$ORCA_BUILD_LIBSLIC3R" \
ARES_ORCA_UPSTREAM_DIR="$ORCA_SOURCE_CHECKOUT" \
ARES_GDB_BIN="$GDB_BIN" \
ARES_PARITY_ARTIFACT_ROOT="$ARTIFACT_ROOT" \
ARES_PYTHON_BIN="$PYTHON_BIN" \
taskset -c "$CPU" cargo nextest run -p ares-cli --test voronoi_incident_native \
  --test-threads 2
```

The top-level admission uses the existing `tests/support/native_environment.rs`
parity convention: absent optional Orca configuration reports `SKIPPED` and
returns before Python/native tools or artifact creation. `ARES_REQUIRE_ORCA=1`
makes absent configuration fail; configured invalid native/wrapper paths or
resources/debug/source directories fail. The same optional/required rule applies
to `ARES_GDB_BIN`, `ARES_ORCA_DEBUG_DIR` (GDB's debug-file-directory root),
`ARES_ORCA_SOURCE_DIR` (the build's `src/libslic3r`) and
`ARES_ORCA_UPSTREAM_DIR` (the clean pinned Git checkout root). Thus ordinary
Tier1 CI without the Orca environment does not
attempt Linux-only diagnostic tooling. Omitted comparisons are NOT Native parity
evidence. Inside the admitted Orca environment `ARES_PYTHON_BIN` is REQUIRED;
there is no `python3` PATH fallback. Missing/wrong Python/GDB/debug/source identity
still fails. No `ignore`, cached Native records or hardcoded expected anchors.
The ELF/debug hashes and ABI in native.py identify the exact qualified seeded
build, not an installation location or generated geometry. The observer launches
the configured native ELF directly under GDB, inheriting the caller's loader
environment; it does not silently substitute a local wrapper/binary/source.
Resources must match that build's resource discovery; the configured directory
is validated and recorded. Relocating/replacing the build requires
requalification, not guessing an offset for an unknown ELF. No application
data/registers/arguments modified
by observer (GDB uses ordinary code/finish breakpoints).

`prepare.py` in the existing paint-color fixture generates input at runtime.
Two actual MMU constructors (`MultiMaterialSegmentation.cpp:1665-1666`) execute
with default detection/repair dispatch (`Geometry/Voronoi.cpp:28-73`). The
observer captures effective return diagrams BEFORE graph mutations. A second
raw-byte decoder independently qualifies all100 arrays. Seed0 and50 completed
calls/process, repeated ordered input/diagram records, whole Native
output (generator header only), positive model activity and hashes are checked
BEFORE any Rust diagram is constructed. Python and its Rust parent must share
exactly one CPU; its actual ID is recorded and used for `ORCA_REPLAY_CPU` and
all observer thread-mask checks. Any singleton CPU chosen by `taskset` or the
nextest pin-runner is valid. All wrapper returns must be unmodified,
no-repair-needed/no-issue; no repair-active claim. This is seeded-only VD
observation; later G-code consumer qualification still requires both modes.

Rust serializes ALL ordered cell/source/raw-category/color/incident, exact f64
vertex bits/color/incident, and edge adjacency/classification/finite fields.
Its strict comparison includes cell anchors; no sorting, cycle rotation,
field-ignore, tolerance or output normalization. Raw category bits are read
from the existing backend Debug format because SourceCategory collapses initial/
reverse8/9; representation changes fail loudly, not default to a category.
The test compares all100 records and saves first differences BEFORE failing.

The persistent artifact directory is printed on success/failure. It contains
input/hash/member evidence, source/build/debug/observer provenance, each launch,
exit and Native PID/thread masks,100 raw+decoded arrays, actual G-code/activity,
qualification receipt, Rust test binary hash/PID, every Rust diagram and strict
comparison result. Native output is NEVER committed. A failing run is preserved,
not overwritten by a rerun. No dependency on historical `/tmp` scripts/dumps.

Before-code failure inventory is the SPEC section3; this RED harness
adds no production vendor/core changes. Both-mode baseline/consumer runs are
not part of this test's parity claim. Public painted XML remains fail-closed and cutting remains blocked.
