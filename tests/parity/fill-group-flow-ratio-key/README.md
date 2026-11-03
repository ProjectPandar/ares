# SurfaceFillParams flow-ratio grouping inputs

SPEC: `docs/architecture/2026-10-20-fill-group-flow-ratio-key-spec.md`, approved
at `2995e6d60`. Orca source: `Fill.cpp::SurfaceFillParams::operator<` and
`group_fills`, pinned `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

Frozen input-only copies of the independently probed compatible-region
`overhang.stl` fixture and its existing parameter modifier. Ten ZIP members;
all eight non-settings members are unchanged from the original passing
fixture. Both global and modifier sparse density are 15%. No geometry,
placement, printer/filament profile, G-code, preview or golden was added.

| Input | Variation | SHA-256 |
|---|---|---|
| equal-density | no top override | `89a9b7babcf153df4377c809064d6036668c3e4667bc727302771d6630fe339b` |
| ratio-equal | local top ratio1 | `887cb0a5f4409b3e386be772394496ef461e54cbda421f290f8d41ad4600a500` |
| ratio11 | local top ratio1.1, target | `38fd0b9767a231f736e5a0a0b70fab3fdf1d6c411ff524c60f2ab63b14d1166c` |
| global-ratio11 | global top ratio1.1 | `73efbf93cd3043c2a7e98e23ed82d6fb806c2f590015e20475ad205fbbcefc7d` |
| local-speed45 | local top speed45 mm/s | `2f403750506a532562fa6d97adeb801587a648ab37037feef3e3fc574786ce4a` |

All five tests assert strict complete Native/public API/CLI equality, with
fresh Native repeats and API/CLI raw identity. Only generator/date identity
is normalized. Before the separately approved GREEN, ratio11 fails and the
four controls pass in both official and seeded modes. This is not an inverted
assertion or a recorded Native golden. The seam_gap sibling is excluded.

Initial discovery and reproducible provenance:
`${EVIDENCE_ROOT}/w3-fill-group-flow-ratio-spec-review/README.md`.
