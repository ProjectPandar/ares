# S-C0 touching-grid public RED

Approved SPEC37145ce12. Frozen authored input, copied byte-for-byte from
`/tmp/worker3-object-spacing-native-1tgobnaj/inputs/S-C0.3mf`.
SHA256: `9580fd4f17fad55829f561c13ee0ae336b447016c7c9682ce301f79afc9d0bc1`.

Single-region Classic two-wall 20x20x10 object, SupportCubic30%, width0.45,
multiline1, layer0.2, top/bottom3, anchors400%/20; no modifier/support/brim/skirt.
Authored geometry/part/plate packaging derives from repository fixture
`tests/parity/validation/modifier-wall-loops-classic.3mf`
(SHA25666e398caa9286359051e0af158bc6ca393bb022671679642f1aa251b851cb7cf).
Project settings derive from the prior B-S A45 runtime input
(SHA2566695378f361f06f2a77f1ab1d36927eb0d95bea1ed72782cf85ce2d3498cbef6),
with the parameters above authored by the preflight generator. Printer:
Creality Ender-3 0.4 nozzle / 0.20mm Standard @Creality Ender3 0.4 /
Creality Generic PLA. ZIP members are authored inputs, with fixed timestamps;
there is no generated G-code or frozen Native expected output.

Original generator, manifest and inputs are retained in the durable STOP
archive referenced by docs/architecture/support-cubic-touching-grid-spec.md.
This slice reuses the frozen bytes, not a regenerated or tuned variant.

The runtime test uses OrcaRunner::from_env (canonical ARES_ORCA_BIN required),
Native --slice 0, the public validated API (plate None, seed0) and real CLI.
It requires Native/API/CLI admission, strict whole-output comparison using
the existing generator-only comparator, and raw API==CLI bytes. CLI runs
first so its status/stderr remain available if the API panics. API panic is
an ordinary test failure; no exception is swallowed or treated as success.
Launch, admission, stderr and successful comparison artifacts are retained.

GREEN must reproduce release-build grid/visit semantics from the cited
source. Upstream asserts are compiled out under NDEBUG; merely deleting the
Ares assert does not establish equivalent trace/grid operands or traversal.
