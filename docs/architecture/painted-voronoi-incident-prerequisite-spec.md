# SPEC: standalone Boost1.86 Voronoi cell incident assignment

Source-cited standalone SPEC → runtime Native RED → minimal GREEN slice.
This slice extracts the incident prerequisite from the painted development
work. Exactly three commits: this SPEC, the runtime RED harness with pure input
fixtures, then production/literal GREEN. No painted production, public paint
test, roadmap or painted-document dependency. RED lands only with its GREEN;
full parity must have zero failures.

## 1. Source owner and Rust destination

Pinned Orca `8500fcdccaa10b5099ac20d252af3a7c560046f1` calls Boost through
`src/libslic3r/Geometry/Voronoi.cpp:28-73`; the observed real MMU construction is
`src/libslic3r/MultiMaterialSegmentation.cpp:1555-1556`. Matching seeded Native
uses Boost1.86. Owning implementation is
`boost/polygon/voronoi_diagram.hpp::_build:470-475`: iterate surviving edges in
ascending storage order, assign their cell incident pointer EVERY time and
assign each non-null vertex0 incident EVERY time. Last edge wins for both.
Cells without surviving edges remain null. This is an upstream rewrite slice,
not an independently designed Ares pipeline behavior.

Rust destination:
`vendor/boostvoronoi/src/diagram/diagram_impl.rs::Diagram::finish:136-144`.
Remove only the `cell_get_incident_edge_(cell).is_none()` conditional around
`cell_set_incident_edge_(cell, Some(edge_it))`. Preserve iterator order, lookup,
vertex branch and later linkage. `iterator_impl.rs:22-24` yields ascending
0..edges.len() after compaction (`finish:70-129`, Boost `_build:441-468`).
No new engine, API/crate/FFI, clearing pass, sorting, normalization, feature
switch or fallback. Pure Rust production remains portable to browser WASM,
Windows, macOS and Linux. Native observation/filesystem/tooling is CLI-test-only.
Existing Arachne/medial-axis adapters remain unchanged compatibility consumers.

Included: exact cell assignment port and minimal affected literal expectations.
Deferred: sweep predicates, compaction changes, source categories, general Orca
issue detection/rotation/repair equivalence, all painted producer/apply and
public painted admission. The no-repair rectangle does not qualify repaired VD.
Unmerged painted scaffolding is a temporary compatibility shell kept separate
and is not carried into this slice.

## 2. Earlier-write audit and invariant

Recheck at the RED and GREEN commits:

| Boundary | Required finding |
| --- | --- |
| Boost cell ctor `voronoi_diagram.hpp:41-47`; Rust `cell_impl.rs:29-36` | Both initialize null/None incidents. |
| Rust `diagram_cell_impl.rs:44-59` | Real/placeholder cells use `Cell::new`; no preset anchor. |
| Rust `builder.rs:215-237` | Sweep ends before `output.finish()`. |
| Rust `diagram_cell_impl.rs:111-114` | Sole cell setter; sole caller is target finish loop. |
| Rust `diagram_impl.rs:70-129`, `diagram_edge_impl.rs:148-155,277-331` | Removal/edge copying does not preset cell incidents. |
| Boost `_build:441-468`, `remove_edge:583-616` | Pre-loop removal does not set cell incidents. |
| Rust `diagram_impl.rs:197-261`; Boost `_build:496-552` | Later infinite-edge linkage reads incidents; leave unchanged and test consumers. |

Vertex writes are distinct. Unexpected earlier cell writes or remaining diagram
mismatch mean STOP/report, not an expanded backend repair.

## 3. Before-code failure inventory

- Wrong first/last winner/order: compare every complete ordered cell record.
- Earlier preset survives compaction: audit constructor/write/removal paths;
  do not add unnecessary reset passes.
- Vertex/classification/f64 regression: compare all cells, vertices and edges,
  exact float bits, adjacency, categories/colors and classification flags.
- Infinite-edge linkage/traversal regression: Arachne
  `arachne/trapezoidation/voronoi.rs:48`, medial axis
  `geometry/medial_axis/diagram.rs:72`, `annotate/propagate.rs:61` consume anchors;
  complete fixed151 consumer selection and both full-parity modes required.
- Missing/wrong source, ELF, debug, GDB/Python/ptrace or incomplete Native:
  infrastructure NON-GATE failure, not geometric RED or skip inside admission.
- Source-category accessor collapses8/9: preserve raw category bits through
  existing backend Debug representation; fail loudly on format drift.
- Fixture drift: preserve frozen archive/member hashes; no Native output dump
  or expected Native anchor array committed.
- Raw Boost bypasses effective wrapper: observe real MMU ctor returns with
  source-default detection/repair dispatch before graph mutations. Require
  actual unmodified/no-issue returns; do not pretend repair-active coverage.
- False RED/GREEN: Native completes before Rust construction, comparison mismatch
  must be real; never fabricate mismatch or treat infrastructure failure as RED.
- Broad consumer regression: preserve all full-parity tests/control refs; zero failures
  required, no exemptions, name filters, tolerance or comparator/golden repairs.

## 4. Standalone runtime RED harness and pure input dependency

The RED commit contains
`crates/ares-cli/tests/voronoi_incident_native.rs`, helper
`crates/ares-cli/tests/voronoi_incident/diagram.rs`,
`tests/parity/voronoi-incident/{README.md,native.py,observe.py,raw.py}`, plus
CLI dev dependency `boostvoronoi = { workspace = true }` and lock entry only.
No production change. All changed Rust files below400LOC.

The sole external fixture dependency is kept at its ORIGINAL paths, byte-for-byte:

1. `tests/parity/paint-color/manifest.json`: frozen archive/member hashes,
   settings, per-tool activity and both-material layer counts.
2. `tests/parity/paint-color/prepare.py`: deterministic standard-library input
   generator; native.py executes it, generating all three archives outside Git.
3. `tests/parity/paint-color/painted-leaves.3mf`: frozen input archive.

Actual generator disk input is existing main
`tests/parity/interface-shells/two-region-prism.3mf`, checked by template hash.
No other paint-color input file is read by native.py/prepare.py: unpainted and
painted-split are generated from template+manifest, not loaded from archives.
No copied/renamed fixtures. No `paint_color.rs`, painted production code,
encoding/geometry design records, painted README or docs/roadmap chain imported.
Manifest historical paths/metadata remain byte-identical provenance, not a
runtime dependency on the original location. Do not change frozen hashes.

Native admission follows existing `tests/support/native_environment.rs`:
missing OPTIONAL Orca config logs omission and returns; `ARES_REQUIRE_ORCA=1`
hard-fails missing config. Inside admission `ARES_PYTHON_BIN` is required, no
PATH fallback. Missing GDB/debug/source identity fails. No ignores/cfg parking.

For each target run, two fresh actual seeded Native processes (seed0)
execute50 MMU constructor returns each. Matching concrete Boost layouts/ABI,
ELF/debug SHA and build identity verified. Independently decode raw arrays;
require all100 finishes, exits0, pointer bounds, exact repeated input/diagrams,
whole Native G-code equality except existing generator-line policy and genuine
positive model activity. All this completes BEFORE constructing Rust diagrams.
One ordered rectangle repeats50times/process, not100unique geometries.
Compare all ordered cell/source/raw-category/color/incident, f64-bit vertices
and all edge adjacency/flags. No filtering, tolerance, anchor cycle rotation or
normalization. Save first differences and every Native/Rust record outside Git.

RED must run under both official/seeded admission environments and genuinely
fail after qualified Native completion at cell incident (not infrastructure).
The exact-DWARF observer intentionally uses the pinned SEEDED ELF in both target
runs; do not mislabel this as official effective-VD capture. Official and seeded
full-parity G-code consumers remain separate and mandatory. No committed Native arrays.

## 5. Measured literal RED, then production-only GREEN

The RED commit includes the measured raw Boost1.86 literal correction
in `crates/ares-core/src/geometry/tests/medial_axis/diagram.rs`: eight cell
incident expectations and eight directly measured incident cycles. The unchanged
first-wins backend must fail `task22o13_finite_multisite_topology_is_fully_literal`
with a real value mismatch before the vendor edit. Keep all four assertions,
all cell metadata, six vertices,26edge rows, cycle lengths and collection/comparison
code. Any further mismatch STOP/report. GREEN changes ONLY the section1 vendor
production delta, with no test or documentation changes.

### Raw Boost1.86 literal reproduction and external evidence

The raw diagnostic is separate from the effective-Orca runtime harness. It uses
`boost/polygon/voronoi.hpp`, `boost/polygon/segment_data.hpp` and
`construct_voronoi` with the SAME ordered test rectangle segments:
`(0,0)->(20,0)`, `(20,0)->(20,10)`, `(20,10)->(0,10)`, `(0,10)->(0,0)`.
Construct `std::vector<boost::polygon::segment_data<int>>` and
`boost::polygon::voronoi_diagram<double>`. For each cell in storage order, record
`incident_edge() - &vd.edges().front()`. Starting at that edge, repeatedly record
the same storage index and follow `next()` until returning to the start. Do not
sort, rotate, normalize or derive expected values from the Rust implementation.
The cell/edge traversal is the same raw Boost1.86 header implementation at
`voronoi_diagram.hpp:470-475`; this is not repaired/effective-Orca qualification.

The diagnostic records direct cell incidents and full incident cycles, built
with GCC15.3.0 against the Boost1.86 headers. Reconstruct it by the complete
input/API/traversal method above; never replace it with Rust-derived goldens.
No diagnostic C++ or Native/raw-Boost output is committed.

Freeze `test(arachne::) | test(geometry::tests::medial_axis)` fixed151 inventory
before production edit. At the GREEN commit run:
- cargo fmt workspace and vendor file rustfmt checks;
- strict cargo clippy core+CLI alltargets `-D warnings`;
- cargo check ares-wasm browser target;
- complete151 consumer tests with nextest, all pass;
- two fresh independent mandatory Native-first complete-diagram runs, all match;
- full official and seeded canonical `ares-cli::orca_parity`, no top-level name
  filter or removed controls, **zero failures**. Existing optional omission
  logs remain explicit omissions, not Native successes.

No repository launcher/seed/comparator changes.

Account for whole retained outputs/hashes and any byte changes without added
normalization or new parity credit. Explain Native stochastic diagnostic deltas
as diagnostics, not strict equivalence. No Native golden/literal repair other
than the measured raw-Boost test correction above.

RED does not land without GREEN. Painted development remains separate; painted
final gates and cutting stay deferred, not a known-failure exemption in this
standalone full-parity run.
