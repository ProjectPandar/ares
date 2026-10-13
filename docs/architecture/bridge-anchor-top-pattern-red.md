# C2-T admitted public E2E RED

Source base: merged `4c5812d18`; production untouched.
Approved SPEC `2f2cdfe41`; local top-pattern slice only.

Permanent public E2E: object Grid, model-part Grid, and a thin slab whose
geometry has no internal bridge candidates, each paired with rectilinear.
All public ZIP members except local model-settings metadata retain their
exported bytes/hashes. Slab geometry is an explicitly recorded positive Z
scale of the pinned binary cube10 STL; no internal surface injection.

Corrected run official CPU13 / seeded CPU14, jobs2/threads2, fresh cache/root:
each 3 tests run, 1 passed and 2 failed, exit 100. No-candidate pair passes
whole-output API/CLI parity. Each active case reaches the unchanged whole-output
PASS assertion with ARES_ERROR: unsupported project feature: top_surface_pattern.
Its CLI independently returns UnsupportedProjectFeature("top_surface_pattern").
All four rectilinear/no-candidate cases per mode pass whole Native comparisons.

Native admits both active metadata scopes, emits a complete footer, nonempty
internal bridge and top-role moves; Grid changes the top-role moves from its
rectilinear control. Live receipts join the exact input and raw Native output
SHA to the actual wrapper/ELF. Official wrapper is ASLR-off SHA1689a2ff.
Evidence: `/tmp/worker2-c2t-red-evidence.json`.

Initial no-candidate attempt misread binary STL as UTF-8; that setup failure
is preserved in original YEGS6h/UW35dH roots and excluded from RED. Correcting
the fixture's binary Z transform changed no assertion or production code.
Native preflight `/tmp/worker2-c2t-admission-summary.json` is retained separately.

Green must keep these expectations/comparators unchanged. A global Grid probe
also demonstrates Native configuration rejection versus Ares Unsupported.
The requested global-validation PLAN amendment is separate; no production
implementation starts until its scope is ruled on.
